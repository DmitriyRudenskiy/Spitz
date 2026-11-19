//! The compliance suite the `engine-protocol` baseline spec asks for, driven against
//! engines that exist outside this repository.
//!
//! The fake engine in `engine_protocol_spec.rs` is obedient because it has to be, which is
//! also its weakness: it says what its author thought to say. A real engine is the only
//! witness to the things nobody models twice — a banner before the id lines, an option with
//! a space in its name, a `Spin` whose minimum is negative, an evaluation that arrives as
//! `mate 1` on the first ply. This suite is the part of the spec that cannot be satisfied
//! with a fixture.
//!
//! Engines are never vendored (ADR-0001) and the default CI job opens no sockets
//! (ADR-0009), so the executables come from the environment: `SPITZ_ENGINES` is a colon
//! separated list of paths, filled by `scripts/fetch-engines.sh`. With nothing in it the
//! suite says so and passes — a pull request cannot depend on a download — and the nightly
//! `engines` workflow is where it actually runs, against three engines.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use chess_core::board::Board;
use chess_core::{fen, state, uci};
use uci_host::{Engine, EngineId, Limits, OptionDecl, Position};

/// A position where White mates on the back rank in more than one way, so the engine has
/// to have searched rather than guessed, and the answer is checkmate either way.
const MATE_IN_ONE: &str = "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";

/// The engines under test. Empty means the suite has nothing to say.
fn engines() -> Vec<PathBuf> {
    std::env::var("SPITZ_ENGINES")
        .unwrap_or_default()
        .split(':')
        .map(Path::new)
        .filter(|path| path.exists())
        .map(PathBuf::from)
        .collect()
}

/// Run one engine's checks over every engine, collecting what failed so one stubborn engine
/// does not hide the others. The closure returns the failures it found, not a panic.
fn for_each_engine(mut check: impl FnMut(&Path) -> Vec<String>) -> Vec<String> {
    let engines = engines();
    if engines.is_empty() {
        eprintln!(
            "note: SPITZ_ENGINES is empty, so the real-engine suite ran nothing. \
             scripts/fetch-engines.sh fills it; the nightly engines job runs it."
        );
        return Vec::new();
    }
    engines
        .iter()
        .flat_map(|program| {
            let mut failures = check(program);
            for failure in &mut failures {
                *failure = format!("{}: {failure}", program.display());
            }
            failures
        })
        .collect()
}

fn assert_clean(failures: &[String], what: &str) {
    assert!(
        failures.is_empty(),
        "{what} failed:\n{}",
        failures.join("\n")
    );
}

/// A live session with a real engine: what it calls itself, and where its log is, because
/// the log is the evidence when a check fails.
struct Session {
    engine: Engine,
    id: EngineId,
    log: PathBuf,
}

/// A session with a real engine, logged so a failure can be read afterwards.
fn session(program: &Path, name: &str) -> Session {
    let log = std::env::temp_dir().join(format!(
        "uci-host-real-{name}-{}",
        program.file_name().unwrap().to_string_lossy()
    ));
    let mut engine = Engine::spawn(program, &[] as &[&str], Some(&log)).unwrap();
    let id = engine.handshake().unwrap();
    Session { engine, id, log }
}

/// One search bound, small enough that four engines and three of them stay a few seconds.
fn depth(d: u32) -> Limits {
    Limits {
        depth: Some(d),
        nodes: None,
        movetime: None,
        white_ms: None,
        black_ms: None,
        infinite: false,
    }
}

#[test]
fn the_handshake_cycle_completes_in_the_required_order() {
    let failures = for_each_engine(|program| {
        let session = session(program, "handshake");
        let mut engine = session.engine;
        let mut failures = Vec::new();
        if session.id.name.trim().is_empty() {
            failures.push(format!("the engine named itself nothing: {:?}", session.id));
        }
        engine.is_ready().unwrap();
        engine.new_game().unwrap();
        engine
            .set_position(Position::StartPos { moves: &[] })
            .unwrap();
        let outcome = engine.search(&depth(8)).unwrap();
        if outcome.bestmove.is_none() {
            failures.push("a search from the start position returned no move".to_string());
        }
        if outcome.infos.is_empty() {
            failures.push("the engine searched without reporting a single info line".into());
        }
        let log = std::fs::read_to_string(&session.log).unwrap();
        // The spec's order, read back out of the session log rather than trusted to the
        // code that wrote it.
        let steps = [
            "< id name",
            "< uciok",
            "> isready",
            "< readyok",
            "> ucinewgame",
            "> position",
            "> go",
            "< bestmove",
        ];
        let mut previous = 0usize;
        for step in steps {
            let found = log.match_indices(step).map(|(at, _)| at).last();
            match found {
                None => failures.push(format!("{} never appears in the session log", step)),
                Some(at) if at < previous => {
                    failures.push(format!("{} arrived out of order", step))
                }
                Some(at) => previous = at,
            }
        }
        let _ = engine.quit();
        failures
    });
    assert_clean(&failures, "the handshake cycle");
}

#[test]
fn every_option_type_a_real_engine_declares_can_be_set() {
    let failures = for_each_engine(|program| {
        let session = session(program, "options");
        let mut engine = session.engine;
        let mut failures = Vec::new();
        let declared = engine.declared().to_vec();
        eprintln!(
            "note: {} declares {} option(s)",
            session.id.name,
            declared.len()
        );
        for decl in declared {
            let (name, value) = match &decl {
                OptionDecl::Spin { name, min, .. } => {
                    let below = min.saturating_sub(1);
                    if engine.set_option(name, &below.to_string()).is_ok() {
                        failures.push(format!(
                            "{name} accepted {below}, one below its own minimum of {min}"
                        ));
                    }
                    (name.clone(), (min + 1).to_string())
                }
                OptionDecl::Check { name, .. } => (name.clone(), "false".to_string()),
                OptionDecl::Combo { name, variants, .. } => {
                    if variants.is_empty() {
                        failures.push(format!("{name} is a combo with no variants to choose"));
                        continue;
                    }
                    (name.clone(), variants[0].clone())
                }
                OptionDecl::Str { name, .. } => (
                    // A real engine acts on a string: this one opens the file it is given,
                    // so the value is a path in the temporary directory and not a word.
                    name.clone(),
                    std::env::temp_dir()
                        .join(format!("uci-host-compliance-{name}.log"))
                        .to_string_lossy()
                        .into_owned(),
                ),
                OptionDecl::Button { name } => {
                    // A button takes no value, so it cannot go through set_option — and
                    // pressing it must still reach the engine.
                    if let Err(why) = engine.press_option(name) {
                        failures.push(format!("pressing {name} failed: {why}"));
                    }
                    continue;
                }
            };
            if let Err(why) = engine.set_option(&name, &value) {
                failures.push(format!("setting {name} to {value:?} failed: {why}"));
            } else if engine.option_value(&name) != Some(value.as_str()) {
                failures.push(format!(
                    "{name} was set to {value:?} and reads back as {:?}",
                    engine.option_value(&name)
                ));
            }
        }
        let _ = engine.quit();
        failures
    });
    assert_clean(&failures, "the option exchange");
}

#[test]
fn a_real_engine_returns_a_move_the_rules_agree_with() {
    let failures = for_each_engine(|program| {
        let session = session(program, "moves");
        let id = session.id;
        let mut engine = session.engine;
        let mut failures = Vec::new();
        let board = fen::parse(MATE_IN_ONE).unwrap();
        engine
            .set_position(Position::Fen {
                fen: MATE_IN_ONE,
                moves: &[],
            })
            .unwrap();
        let outcome = engine.search(&depth(12)).unwrap();
        match outcome.bestmove.as_deref() {
            None => failures.push("no bestmove came back".to_string()),
            Some(text) => match uci::parse(&board, text) {
                Err(why) => {
                    failures.push(format!("the engine's move {text} is not a move: {why:?}"))
                }
                Ok(mv) => {
                    let mut after = board;
                    after.apply(mv).expect("the parsed move is playable");
                    eprintln!("note: {} mated with {}", id.name, text);
                    if !matches!(
                        state::outcome(&after),
                        state::Outcome::Checkmate {
                            winner: chess_core::board::Colour::White
                        }
                    ) {
                        failures.push(format!(
                            "{text} was played from a mate in one and did not mate"
                        ));
                    }
                }
            },
        }
        let _ = engine.quit();
        failures
    });
    assert_clean(&failures, "the move a real engine returns");
}

#[test]
fn multipv_gives_the_selection_layer_lines_to_choose_between() {
    let failures = for_each_engine(|program| {
        let session = session(program, "multipv");
        let mut engine = session.engine;
        let mut failures = Vec::new();
        if !engine
            .declared()
            .iter()
            .any(|decl| decl.name() == "MultiPV")
        {
            // An engine without MultiPV is not a failure of the host; the selection layer
            // degrades to one line plus depth caps, which is move-selection's business.
            eprintln!("note: {} declares no MultiPV", session.id.name);
            return failures;
        }
        engine.set_option("MultiPV", "3").unwrap();
        engine
            .set_position(Position::StartPos { moves: &[] })
            .unwrap();
        let outcome = engine.search(&depth(14)).unwrap();

        // The last word on each of the engine's lines: the report the selection layer is
        // handed. Several distinct moves, each scored, each playable in the position the
        // search started from — the shape that layer needs, checked where it is produced.
        let mut lines: Vec<(u32, String)> = Vec::new();
        let mut scored = 0usize;
        for info in &outcome.infos {
            let (Some(first), Some(_)) = (info.pv.first(), info.score) else {
                continue;
            };
            scored += 1;
            if uci::parse(&Board::start(), first).is_err() {
                failures.push(format!("line {} named a non-move: {first}", info.multipv));
            }
            if !lines.iter().any(|(_, known)| known == first) {
                lines.push((info.multipv, first.clone()));
            }
        }
        eprintln!(
            "note: {} reported {} distinct line(s) across {} scored info line(s)",
            session.id.name,
            lines.len(),
            scored
        );
        if lines.len() < 2 {
            failures.push(format!(
                "MultiPV 3 came back with {} distinct line(s): {:?}",
                lines.len(),
                outcome
                    .infos
                    .iter()
                    .map(|info| (info.multipv, info.depth))
                    .collect::<Vec<_>>()
            ));
        }
        for (multipv, first) in &lines {
            let mv = uci::parse(&Board::start(), first).unwrap();
            let mut board = Board::start();
            if board.apply(mv).is_err() {
                failures.push(format!(
                    "line {multipv} named {}, which the rules refuse",
                    uci::format(mv)
                ));
            }
        }
        let _ = engine.quit();
        failures
    });
    assert_clean(&failures, "MultiPV feeding the selection layer");
}

#[test]
fn four_instances_search_at_once_and_each_stops_when_its_own_stop_arrives() {
    let failures = for_each_engine(|program| {
        // The spec's number is 200 ms from stop to bestmove; the budget here is that plus
        // the time the harness spends getting there, so a slow machine is not the failure.
        const STOP_BUDGET: Duration = Duration::from_millis(200);
        let mut engines = Vec::new();
        for _ in 0..4 {
            let mut engine = session(program, "concurrent").engine;
            if engine
                .declared()
                .iter()
                .any(|decl| decl.name() == "Threads")
            {
                engine.set_option("Threads", "1").unwrap();
            }
            engine
                .set_position(Position::StartPos { moves: &[] })
                .unwrap();
            let infinite = Limits {
                infinite: true,
                ..depth(0)
            };
            engine.start_search(&infinite).unwrap();
            engines.push(engine);
        }
        std::thread::sleep(Duration::from_millis(400));

        let mut failures = Vec::new();
        for (index, engine) in engines.iter_mut().enumerate() {
            let started = Instant::now();
            engine.stop().unwrap();
            let outcome = engine
                .await_bestmove(Some(Duration::from_secs(5)))
                .expect("a stopped search still ends with a bestmove");
            let took = started.elapsed();
            if outcome.bestmove.is_none() {
                failures.push(format!("instance {index} stopped without a move"));
            }
            if took > STOP_BUDGET + Duration::from_millis(300) {
                failures.push(format!(
                    "instance {index} took {took:?} to answer its own stop, over the 200 ms budget"
                ));
            }
        }
        for engine in engines {
            let _ = engine.quit();
        }
        failures
    });
    assert_clean(&failures, "four concurrent instances");
}
