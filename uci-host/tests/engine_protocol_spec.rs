//! The engine-protocol spec, scenario by scenario, against the `engine-protocol` baseline
//! spec.
//!
//! The engine under test is a few dozen lines of awk written into a temporary directory by
//! each test: a real child process speaking real UCI over real pipes, generated rather than
//! vendored, and obedient enough that "did the host survive this" has one answer. Its
//! misbehaviour is switched on by a generated shell wrapper, because the interesting half of
//! hosting is what happens when an engine is unwell.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use uci_host::{Engine, EngineId, Health, HostError, Limits, OptionDecl, Position, Registry};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const FAKE_ENGINE: &str = r#"#!/usr/bin/awk -f
BEGIN {
  crash_after = ENVIRON["FAKE_CRASH_AFTER"] + 0
  noisy = ENVIRON["FAKE_NOISY"] + 0
  slow = ENVIRON["FAKE_SLOW"] + 0
  gosc = 0
  searching = 0
  print "Fake 1.0 - a test engine with opinions"
  fflush()
}
{
  cmd = $0
  if (cmd == "uci") {
    print "id name Fake 1.0"
    print "id author Spitz tests"
    print "option name MultiPV type spin default 1 min 1 max 8"
    print "option name UCI_Elo type spin default 1000 min 800 max 3200"
    print "option name UCI_Chess960 type check default false"
    print "option name Style type combo default solid var solid var aggressive var positional"
    print "option name Debug File Path type string default <empty>"
    print "option name Reset type button"
    print "uciok"
    fflush()
  } else if (cmd == "isready") {
    if (slow > 0) { system("sleep " slow); slow = 0 }
    print "readyok"
    fflush()
  } else if (cmd == "ucinewgame") {
    print "info string forgetting the last game"
    fflush()
  } else if (cmd ~ /^setoption/) {
    print "info string set " $3 " to " $NF
    fflush()
  } else if (cmd ~ /^go/) {
    gosc = gosc + 1
    if (crash_after > 0 && gosc >= crash_after) { exit 1 }
    if (noisy) {
      print "info string thinking about nothing"
      print "this is not uci at all"
      print "info depth 3 score"
    }
    print "info depth 1 seldepth 1 score cp 34 nodes 120 time 1 pv e2e4"
    print "info depth 1 seldepth 1 score cp 12 nodes 120 time 1 multipv 2 pv d2d4"
    if (cmd ~ /infinite/) {
      searching = 1
    } else {
      print "info depth 2 seldepth 3 score cp 36 nodes 940 time 4 pv e2e4 e7e5"
      print "bestmove e2e4 ponder e7e5"
    }
    fflush()
  } else if (cmd == "stop") {
    if (searching) {
      searching = 0
      print "bestmove e2e4 ponder e7e5"
      fflush()
    }
  } else if (cmd == "quit") {
    exit 0
  }
}
"#;

/// A search with one bound, which is all a test needs the engine to obey.
fn movetime(ms: u64) -> Limits {
    Limits {
        depth: None,
        nodes: None,
        movetime: Some(ms),
        white_ms: None,
        black_ms: None,
        infinite: false,
    }
}

fn executable(path: &Path) {
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// A directory of its own per test, with the engine script inside it.
struct Harness {
    dir: PathBuf,
    engine: PathBuf,
}

impl Harness {
    fn new(name: &str) -> Harness {
        let dir = std::env::temp_dir().join(format!("uci-host-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("a temp directory for the engine under test");
        let engine = dir.join("fake-engine");
        fs::write(&engine, FAKE_ENGINE).expect("the engine script is written");
        executable(&engine);
        Harness { dir, engine }
    }

    fn log(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{name}.log"))
    }

    fn read_log(&self, name: &str) -> String {
        fs::read_to_string(self.log(name)).unwrap_or_default()
    }

    fn engine(&self) -> Engine {
        Engine::spawn(&self.engine, &[] as &[&str], Some(self.log("session"))).unwrap()
    }

    fn handshaked(&self) -> Engine {
        let mut engine = self.engine();
        engine.handshake().unwrap();
        engine
    }

    /// The engine, started with its environment set — a wrapper rather than `set_var`,
    /// because tests run in parallel and the environment is one shared table.
    fn awkward(&self, name: &str, vars: &[(&str, &str)]) -> Engine {
        let wrapper = self.dir.join(name);
        let mut script = String::from("#!/bin/sh\n");
        for (key, value) in vars {
            script.push_str(&format!("export {key}={value}\n"));
        }
        script.push_str(&format!("exec '{}' \"$@\"\n", self.engine.display()));
        fs::write(&wrapper, script).unwrap();
        executable(&wrapper);
        Engine::spawn(&wrapper, &[] as &[&str], Some(self.log("session"))).unwrap()
    }

    fn clean(self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn the_handshake_names_the_engine_and_what_it_can_be_told() {
    let harness = Harness::new("handshake");
    let mut engine = harness.engine();

    let id = engine.handshake().unwrap();
    assert_eq!(
        id,
        EngineId {
            name: "Fake 1.0".into(),
            author: Some("Spitz tests".into()),
        },
        "the session shows who it is talking to"
    );

    let declared = engine.declared();
    assert_eq!(
        declared
            .iter()
            .map(|decl| decl.name())
            .collect::<Vec<_>>()
            .join(","),
        "MultiPV,UCI_Elo,UCI_Chess960,Style,Debug File Path,Reset"
    );
    assert!(declared.contains(&OptionDecl::Spin {
        name: "MultiPV".into(),
        min: 1,
        max: 8,
        default: 1,
    }));
    assert!(declared.contains(&OptionDecl::Check {
        name: "UCI_Chess960".into(),
        default: false,
    }));
    assert!(
        declared.iter().any(
            |decl| matches!(decl, OptionDecl::Combo { name, variants, .. }
                if name == "Style" && variants.len() == 3)
        ),
        "a combo declares its variants, or the UI has nothing to offer"
    );
    assert!(
        declared
            .iter()
            .any(|decl| matches!(decl, OptionDecl::Str { name, .. } if name == "Debug File Path"))
    );
    assert!(
        declared
            .iter()
            .any(|decl| matches!(decl, OptionDecl::Button { name } if name == "Reset"))
    );

    engine.is_ready().unwrap();
    engine.quit().unwrap();
    harness.clean();
}

#[test]
fn readiness_is_answered_only_when_the_engine_is_ready() {
    let harness = Harness::new("ready");
    let mut engine = harness.handshaked();
    engine.is_ready_within(Duration::from_secs(5)).unwrap();
    engine.is_ready_within(Duration::from_secs(5)).unwrap();

    let log = harness.read_log("session");
    let asked = log.matches("> isready").count();
    let answered = log.matches("< readyok").count();
    assert_eq!(asked, 2);
    assert_eq!(
        asked, answered,
        "every readiness question gets exactly one answer, in order"
    );
    assert!(log.find("> isready").unwrap() < log.find("< readyok").unwrap());
    engine.quit().unwrap();
    harness.clean();
}

#[test]
fn an_option_round_trips_and_a_bad_one_never_leaves() {
    let harness = Harness::new("options");
    let mut engine = harness.handshaked();

    engine.set_option("MultiPV", "4").unwrap();
    engine.set_option("Style", "aggressive").unwrap();
    engine.set_option("UCI_Chess960", "true").unwrap();
    engine.set_option("Debug File Path", "/tmp/debug").unwrap();
    assert_eq!(engine.option_value("MultiPV"), Some("4"));
    assert_eq!(engine.option_value("Style"), Some("aggressive"));
    engine.set_option("MultiPV", "6").unwrap();
    assert_eq!(
        engine.option_value("MultiPV"),
        Some("6"),
        "the last value asked for is the state shown"
    );

    let log = harness.read_log("session");
    assert!(log.contains("> setoption name MultiPV value 4"));
    assert!(log.contains("> setoption name Style value aggressive"));

    // Rejected here, so the engine never gets the chance to react badly to it.
    for (name, value) in [
        ("UCI_Elo", "999999"),
        ("Skill Level", "8"),
        ("Style", "chaotic"),
        ("Reset", "now"),
        ("MultiPV", "not a number"),
    ] {
        let error = engine.set_option(name, value).unwrap_err();
        assert!(
            matches!(error, HostError::BadOption { .. }),
            "{name}={value}"
        );
        assert!(
            !harness
                .read_log("session")
                .contains(&format!("> setoption name {name} value {value}")),
            "a rejected option reached the engine: {name}={value}"
        );
    }

    // A button carries no value, so it cannot go through set_option — and pressing one is
    // still a command the engine has to receive.
    engine.press_option("Reset").unwrap();
    assert!(
        harness
            .read_log("session")
            .contains("> setoption name Reset")
    );
    for not_a_button in ["MultiPV", "Nope"] {
        assert!(
            matches!(
                engine.press_option(not_a_button),
                Err(HostError::BadOption { .. })
            ),
            "{not_a_button} is not a button, and pressing it should say so"
        );
    }
    engine.quit().unwrap();
    harness.clean();
}

#[test]
fn a_search_reports_every_line_the_engine_found() {
    let harness = Harness::new("search");
    let mut engine = harness.handshaked();
    engine.new_game().unwrap();
    engine
        .set_position(Position::StartPos {
            moves: &["e2e4".into(), "e7e5".into()],
        })
        .unwrap();

    let outcome = engine
        .search(&Limits {
            depth: Some(2),
            ..Default::default()
        })
        .unwrap();

    assert_eq!(outcome.infos.len(), 3);
    assert_eq!(outcome.bestmove.as_deref(), Some("e2e4"));
    assert_eq!(outcome.ponder.as_deref(), Some("e7e5"));

    let first = &outcome.infos[0];
    assert_eq!(first.depth, 1);
    assert_eq!(first.multipv, 1);
    assert_eq!(first.score, Some(uci_host::InfoScore::Cp(34)));
    assert_eq!(first.pv, vec!["e2e4".to_string()]);
    assert_eq!(first.nodes, 120);

    let second = &outcome.infos[1];
    assert_eq!(second.multipv, 2, "the second line of a MultiPV report");
    assert_eq!(second.score, Some(uci_host::InfoScore::Cp(12)));
    assert_eq!(
        outcome.infos[2].score,
        Some(uci_host::InfoScore::Cp(36)),
        "the deeper score arrives as the search goes"
    );

    let log = harness.read_log("session");
    assert!(log.contains("> position startpos moves e2e4 e7e5"));
    assert!(log.contains("> go depth 2"));
    engine.quit().unwrap();
    harness.clean();
}

#[test]
fn a_stop_belongs_to_one_engine_and_not_to_the_others() {
    let harness = Harness::new("stop");
    let mut engines: Vec<Engine> = (0..4).map(|_| harness.handshaked()).collect();
    let infinite = Limits {
        infinite: true,
        ..Default::default()
    };
    for engine in engines.iter_mut() {
        engine.start_search(&infinite).unwrap();
    }

    // Stop two of them. The other two are still searching and must stay that way.
    let mut stopped = Vec::new();
    for index in [1, 3] {
        let started = Instant::now();
        engines[index].stop().unwrap();
        let outcome = engines[index]
            .await_bestmove(Some(Duration::from_secs(5)))
            .unwrap();
        assert!(outcome.bestmove.is_some(), "engine {index} stopped silent");
        stopped.push((index, started.elapsed()));
    }
    for index in [0, 2] {
        assert!(
            engines[index]
                .await_bestmove(Some(Duration::from_millis(150)))
                .is_err(),
            "engine {index} was stopped by someone else's stop request"
        );
    }
    for (index, took) in stopped {
        assert!(
            took < Duration::from_secs(1),
            "engine {index} took {took:?} to answer its own stop"
        );
    }

    // The ones left searching finish when finally told to, which is the point: their
    // queues were never poisoned by the engines that stopped.
    for index in [0, 2] {
        engines[index].stop().unwrap();
        let outcome = engines[index]
            .await_bestmove(Some(Duration::from_secs(5)))
            .unwrap();
        assert!(outcome.bestmove.is_some());
    }
    harness.clean();
}

#[test]
fn nonsense_from_the_engine_is_logged_and_the_search_finishes() {
    let harness = Harness::new("nonsense");
    let mut engine = harness.awkward("noisy", &[("FAKE_NOISY", "1")]);
    engine.handshake().unwrap();
    let outcome = engine
        .search(&Limits {
            depth: Some(1),
            ..Default::default()
        })
        .unwrap();

    assert_eq!(outcome.bestmove.as_deref(), Some("e2e4"));
    assert_eq!(
        outcome.infos.len(),
        3,
        "the noise is not a line of the report"
    );

    let log = harness.read_log("session");
    assert!(
        log.contains("< this is not uci at all"),
        "an unrecognized line has to be diagnosable: {log}"
    );
    assert!(
        log.contains("< info depth 3 score"),
        "a truncated line has to be diagnosable too"
    );
    engine.quit().unwrap();
    harness.clean();
}

#[test]
fn a_crashed_engine_comes_back_with_its_game_intact() {
    let harness = Harness::new("crash");
    // The fake engine dies on the second `go` it is given, and a restarted process starts
    // its counter again from zero.
    let mut engine = harness.awkward("crasher", &[("FAKE_CRASH_AFTER", "2")]);
    engine.handshake().unwrap();
    engine.set_option("UCI_Elo", "1500").unwrap();
    engine.set_option("UCI_Chess960", "false").unwrap();
    let moves = vec!["e2e4".to_string()];
    engine
        .set_position(Position::StartPos { moves: &moves })
        .unwrap();
    engine.search(&movetime(50)).unwrap();

    // The engine dies mid-request. The caller asked for a move and gets one: the manager
    // restarts the engine and asks again, without the caller being told it died.
    let outcome = engine.search(&movetime(50)).unwrap();
    assert_eq!(outcome.bestmove.as_deref(), Some("e2e4"));

    let log = harness.read_log("session");
    let replayed = log
        .rsplit("! started")
        .next()
        .expect("a restart is logged like any other start")
        .to_string();
    assert!(
        replayed.contains("ucinewgame"),
        "a restart begins a new game: {replayed}"
    );
    assert!(
        replayed.contains("setoption name UCI_Elo value 1500")
            && replayed.contains("setoption name UCI_Chess960 value false"),
        "a restart replays the options the player set: {replayed}"
    );
    assert!(
        replayed.contains("position startpos moves e2e4"),
        "a restart replays the position the player was in: {replayed}"
    );
    assert!(
        replayed.contains("go movetime 50"),
        "the request that died with the engine is asked for again: {replayed}"
    );
    harness.clean();
}

#[test]
fn an_engine_that_crashes_every_time_is_reported_not_restarted_forever() {
    let harness = Harness::new("always-crash");
    // This engine dies on the first `go` it ever sees, so every search ends the process
    // that started it. Recovery is one attempt, then the caller is told.
    let mut engine = harness.awkward("crasher", &[("FAKE_CRASH_AFTER", "1")]);
    engine.handshake().unwrap();
    let error = engine
        .search(&movetime(50))
        .expect_err("a second death is a result, not a loop");
    assert!(
        matches!(error, HostError::Exited),
        "the caller is told the engine died: {error:?}"
    );
    let log = harness.read_log("session");
    assert_eq!(
        log.lines()
            .filter(|line| line.starts_with("! started"))
            .count(),
        2,
        "one restart for one request, and no more:\n{log}"
    );
    harness.clean();
}

#[test]
fn a_missing_executable_fails_a_health_check_without_taking_the_app_with_it() {
    let harness = Harness::new("missing");
    let absent = harness.dir.join("not-an-engine");
    let error = uci_host::health_check(&absent, &[] as &[&str], None::<&Path>).unwrap_err();
    assert!(matches!(error, HostError::Io(_)), "got {error:?}");

    let mut registry = Registry::default();
    assert!(
        registry
            .add(&absent, &[] as &[&str], None::<&Path>)
            .is_err()
    );
    assert!(registry.list().is_empty(), "a broken engine is not offered");
    harness.clean();
}

#[test]
fn the_registry_shows_an_engine_and_then_hides_it() {
    let harness = Harness::new("registry");
    let mut registry = Registry::default();

    let registered = registry
        .add(
            &harness.engine,
            &[] as &[&str],
            Some(harness.log("register")),
        )
        .unwrap();
    assert_eq!(registered.id.name, "Fake 1.0");
    assert_eq!(registry.list().len(), 1);
    assert_eq!(registry.list()[0].program, harness.engine);

    assert!(registry.remove(&harness.engine));
    assert!(registry.list().is_empty());
    assert!(!registry.remove(&harness.engine), "it was already removed");
    harness.clean();
}

#[test]
fn the_session_log_records_both_directions() {
    let harness = Harness::new("log");
    let mut engine = harness.handshaked();
    engine.is_ready().unwrap();
    engine.quit().unwrap();

    let log = harness.read_log("session");
    assert!(log.contains("> uci"));
    assert!(log.contains("< uciok"));
    assert!(log.contains("> isready"));
    assert!(log.contains("< readyok"));
    assert!(log.contains("> quit"));
    assert!(log.contains("! started"));
    harness.clean();
}

#[test]
fn an_engine_that_never_answers_is_reported_and_not_waited_on() {
    let harness = Harness::new("silent");
    let mut engine = harness.awkward("slow", &[("FAKE_SLOW", "10")]);
    engine.handshake().unwrap();
    let started = Instant::now();
    // The engine sleeps ten seconds before it says readyok; the caller's patience is
    // shorter, and it is the caller's patience that decides when the app moves on.
    let error = engine
        .is_ready_within(Duration::from_millis(300))
        .unwrap_err();
    assert!(matches!(error, HostError::Unresponsive), "got {error:?}");
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(
        engine.health_within(Duration::from_millis(300)),
        Health::NotResponding,
        "running but silent is its own answer, not a crash"
    );
    harness.clean();
}
