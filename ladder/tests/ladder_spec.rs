//! The ladder (task 3.5): the arithmetic of whether a level is real, and one game
//! against an engine that exists.

use chess_core::board::Board;
use ladder::{Contender, Level, MatchConfig, Mechanism, Pair, SEPARATION, pairs, report};
use move_select::Score;
use uci_host::{Info, InfoScore};

fn info(multipv: u32, depth: u32, score: i32, first: &str) -> Info {
    Info {
        depth,
        multipv,
        score: Some(InfoScore::Cp(score)),
        nodes: 100,
        time_ms: 1,
        pv: vec![first.into()],
    }
}

#[test]
fn a_pair_is_separated_only_at_fifty_five_percent() {
    let mut pair = Pair {
        lower: 4,
        higher: 5,
        wins: 11,
        draws: 0,
        losses: 9,
    };
    assert!(
        pair.is_separated(),
        "11/20 is exactly the bar: {:.3}",
        pair.score_rate()
    );

    pair.wins = 10;
    pair.losses = 10;
    assert!(
        !pair.is_separated(),
        "10/20 is not separated: {:.3}",
        pair.score_rate()
    );

    pair.wins = 9;
    pair.draws = 4;
    pair.losses = 7;
    assert!(
        pair.is_separated(),
        "draws count half: {:.3}",
        pair.score_rate()
    );

    let nothing = Pair::new(Level::MIN, Level::MAX);
    assert!(!nothing.is_separated(), "no games proves nothing");
    assert!(nothing.score_rate() < SEPARATION);
}

#[test]
fn the_report_is_what_the_engine_last_said_about_each_line() {
    let board = Board::start();
    let infos = [
        info(1, 1, 10, "e2e4"),
        info(2, 1, 5, "d2d4"),
        // Deeper: line 1's score moved, and a line the position refuses appears.
        info(1, 8, 34, "e2e4"),
        info(2, 8, 12, "d2d4"),
        // Both refused: one is three squares, one is a king with a pawn in front of it.
        info(3, 8, 0, "e2e5"),
        info(4, 8, -5, "e1e4"),
    ];

    let report = report(&infos, &board).unwrap();
    let lines = report.lines();
    assert_eq!(
        lines.len(),
        2,
        "a line the rules refuse is not a line: {lines:?}"
    );
    assert_eq!(
        report.best().score,
        Score::Cp(34),
        "the deepest report about a line, not the first one"
    );
    let second = lines
        .iter()
        .find(|line| line.score == Score::Cp(12))
        .expect("d2d4 at its deepest score");
    assert_eq!(second.score.loss_vs(&report.best().score), 22);
}

#[test]
fn a_position_with_nothing_playable_is_an_error_and_not_an_empty_report() {
    let board = Board::start();
    let error = report(&[info(1, 8, 0, "e9e9")], &board).unwrap_err();
    assert!(error.to_string().contains("none playable"), "{error}");
}

#[test]
fn adjacent_pairs_cover_the_ladder_once() {
    let pairs = pairs(1..=ladder::LEVELS);
    assert_eq!(pairs.len(), 19);
    assert_eq!(
        pairs.first().unwrap(),
        &(Level::MIN, Level::new(2).unwrap())
    );
    assert_eq!(
        pairs.last().unwrap(),
        &(Level::new(19).unwrap(), Level::MAX)
    );
    for (lower, higher) in &pairs {
        assert_eq!(higher.get(), lower.get() + 1, "adjacent, not sampled");
    }
}

/// The one test here that needs an engine. `SPITZ_ENGINES` is the contract the compliance
/// suite and the command both run on.
#[test]
fn a_real_engine_plays_a_game_the_rules_can_read() {
    let Some(program) = std::env::var_os("SPITZ_ENGINES")
        .iter()
        .flat_map(|paths| std::env::split_paths(paths))
        .find(|path| path.is_file())
    else {
        eprintln!("note: no engine configured (SPITZ_ENGINES); the ladder is not measured");
        return;
    };

    let config = MatchConfig {
        games: 1,
        depth: 5,
        max_plies: 30,
        ..MatchConfig::default()
    };
    let pair = ladder::play_pair(&program, Level::MIN, Level::MAX, &config, None).unwrap();
    assert_eq!(
        pair.games(),
        1,
        "the driver validates every move it plays, so finishing is the proof"
    );

    let weakest = Contender::start(&program, Level::MIN, None).unwrap();
    let strongest = Contender::start(&program, Level::MAX, None).unwrap();
    eprintln!(
        "note: {} — L{} {:?} ({} lines), L{} {:?} ({} lines)",
        weakest.id,
        Level::MIN.get(),
        weakest.plan.mechanism,
        weakest.plan.multipv,
        Level::MAX.get(),
        strongest.plan.mechanism,
        strongest.plan.multipv
    );
    assert_eq!(
        strongest.plan.multipv, 1,
        "the top level plays what the engine says, so it asks for one line"
    );
    assert!(
        weakest.plan.multipv > 1,
        "a low level samples, so it needs lines to sample between"
    );
    match (weakest.plan.mechanism, strongest.plan.mechanism) {
        (Mechanism::EloCapped { elo: low }, Mechanism::EloCapped { elo: high }) => assert!(
            high > low,
            "an engine that limits its Elo should be asked for more of it at the top: {low} vs {high}"
        ),
        (
            Mechanism::DepthAndNodes { depth: low, .. },
            Mechanism::DepthAndNodes { depth: high, .. },
        ) => assert!(high > low, "depth {low} vs {high}"),
        (weaker, stronger) => {
            panic!("one engine should not change mechanism mid-ladder: {weaker:?} vs {stronger:?}")
        }
    }
    let _ = weakest.engine.quit();
    let _ = strongest.engine.quit();
}
