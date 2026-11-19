//! The move-selection spec, scenario by scenario, against the `move-selection` baseline
//! spec. No engine is involved: the layer's whole contract is what it does with a report,
//! so a report is all a test has to build.

use chess_core::board::Move;
use move_select::{EngineOptions, EngineReport, Level, Line, Reason, Score, SeedStream, select};

/// Squares are a1=0, so a UCI string is the shortest way to say one.
fn mv(uci: &str) -> Move {
    let b = uci.as_bytes();
    Move::simple(
        (b[0] - b'a') + (b[1] - b'1') * 8,
        (b[2] - b'a') + (b[3] - b'1') * 8,
    )
}

fn report(lines: &[(&str, i32)]) -> EngineReport {
    EngineReport::new(
        lines
            .iter()
            .map(|(uci, cp)| Line {
                mv: mv(uci),
                score: Score::Cp(*cp),
            })
            .collect(),
    )
    .expect("a test report is never empty")
}

/// Four lines of decreasing quality: equal-ish, slightly worse, a mistake, a blunder.
const FOUR_LINES: &[(&str, i32)] = &[("e2e4", 40), ("d2d4", 30), ("g1f3", -60), ("b1c3", -900)];

/// How often the layer plays the engine's first line, and how far down the report its
/// choices wander on average.
fn how_often(level: Level, draws: u64) -> (f64, f64) {
    let report = report(FOUR_LINES);
    let mut best = 0u64;
    let mut rank_sum = 0u64;
    for seed in 0..draws {
        let chosen = select(&report, level, seed, "engine");
        best += u64::from(chosen.rank == 0);
        rank_sum += chosen.rank as u64;
    }
    (best as f64 / draws as f64, rank_sum as f64 / draws as f64)
}

/// How often the weakest line of the report gets played.
fn blunder_rate(level: Level, draws: u64) -> f64 {
    let report = report(FOUR_LINES);
    let worst = report.lines().len() - 1;
    let hits = (0..draws)
        .filter(|seed| select(&report, level, *seed, "engine").rank == worst)
        .count();
    hits as f64 / draws as f64
}

#[test]
fn a_fixed_seed_replays_the_same_moves_in_the_same_order() {
    let report = report(FOUR_LINES);
    let run = |seed| {
        let mut stream = SeedStream::seeded(seed);
        let level = Level::new(6).unwrap();
        (0..200)
            .map(|_| {
                let chosen = select(&report, level, stream.next_seed(), "engine");
                (chosen.mv.unwrap(), chosen.reason, chosen.rank)
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(run(12345), run(12345));
    assert_ne!(run(12345), run(54321));
}

#[test]
fn sampling_widens_as_strength_drops() {
    let (best_at_one, wander_at_one) = how_often(Level::MIN, 20_000);
    let (best_at_nineteen, wander_at_nineteen) = how_often(Level::new(19).unwrap(), 20_000);
    assert!(
        best_at_nineteen > 0.99,
        "the near-top level barely looks away: {best_at_nineteen}"
    );
    assert!(
        best_at_one < 0.6,
        "the weakest level should play the engine's line about half the time: {best_at_one}"
    );
    assert!(
        wander_at_one > wander_at_nineteen * 10.0,
        "the weak level wanders to rank {wander_at_one} on average, the strong one {wander_at_nineteen}"
    );

    let (best_at_twenty, _) = how_often(Level::MAX, 2_000);
    assert_eq!(
        best_at_twenty, 1.0,
        "the top level plays the engine's first line"
    );
}

#[test]
fn the_error_rate_rises_monotonically_as_the_level_drops() {
    let rates: Vec<f64> = (1..=20)
        .map(|level| 1.0 - how_often(Level::new(level).unwrap(), 10_000).0)
        .collect();
    for (index, pair) in rates.windows(2).enumerate() {
        assert!(
            pair[0] > pair[1],
            "level {} errs {:.3} of the time, the level above errs {:.3}",
            20 - index,
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn a_sampled_line_is_a_near_line_and_not_a_random_one() {
    let report = report(FOUR_LINES);
    let mut ranks = [0u64; 4];
    for seed in 0..200_000 {
        ranks[select(&report, Level::MIN, seed, "engine").rank] += 1;
    }
    for pair in ranks.windows(2) {
        assert!(
            pair[0] > pair[1],
            "a weaker line should be picked less often: {ranks:?}"
        );
    }
    assert!(
        ranks[0] > ranks[3] * 2,
        "the report is chosen by quality, not uniformly: {ranks:?}"
    );
    // The shape, not the absolute rate, is the promise: a stronger level plays the same
    // blunder far less often than a weak one.
    assert!(
        blunder_rate(Level::new(15).unwrap(), 100_000) * 8.0 < blunder_rate(Level::MIN, 100_000),
        "the same blunder should thin out as the level rises"
    );
}

#[test]
fn a_level_becomes_the_options_the_engine_declared() {
    let engine = EngineOptions {
        multipv: true,
        uci_elo: Some(1000..=2500),
        uci_limit_strength: true,
        depth: true,
        nodes: true,
    };
    let plan = engine.plan_for(Level::new(10).unwrap());
    assert_eq!(
        plan.settings,
        vec![
            move_select::Setting {
                name: "MultiPV",
                value: "3".into()
            },
            move_select::Setting {
                name: "UCI_LimitStrength",
                value: "true".into()
            },
            move_select::Setting {
                name: "UCI_Elo",
                value: "1425".into()
            },
        ],
        "the level asks for Elo, not for a search budget it would double-cap"
    );
    assert_eq!(plan.multipv, 3);

    // An engine that refuses 3200 is asked for the most it accepts, and the mechanism
    // reports what was actually asked rather than what the level wanted.
    let top = engine.plan_for(Level::MAX);
    assert_eq!(
        top.mechanism,
        move_select::Mechanism::EloCapped { elo: 2500 }
    );
    let weak = engine.plan_for(Level::MIN);
    assert_eq!(
        weak.mechanism,
        move_select::Mechanism::EloCapped { elo: 1000 }
    );
}

#[test]
fn an_engine_without_elo_limiting_degrades_to_caps_and_says_so() {
    let engine = EngineOptions {
        multipv: true,
        uci_elo: None,
        uci_limit_strength: false,
        depth: true,
        nodes: true,
    };
    let plan = engine.plan_for(Level::new(12).unwrap());
    assert_eq!(
        plan.mechanism,
        move_select::Mechanism::DepthAndNodes {
            depth: 17,
            nodes: 2_048_000
        }
    );
    assert!(
        !plan
            .settings
            .iter()
            .any(|setting| setting.name == "UCI_Elo"),
        "an option the engine never declared must not be sent to it"
    );

    let nothing = EngineOptions::default().plan_for(Level::MIN);
    assert_eq!(nothing.mechanism, move_select::Mechanism::SamplingOnly);
    assert_eq!(
        nothing.multipv, 1,
        "without MultiPV there is one line and nothing to sample"
    );
    assert!(nothing.settings.is_empty());
}

#[test]
fn every_decision_records_why_and_against_what() {
    let report = report(FOUR_LINES);
    let seed = (0..10_000)
        .find(|seed| {
            select(&report, Level::MIN, *seed, "stockfish-16").reason == Reason::SampledLine
        })
        .expect("the weakest level samples within ten thousand draws");
    let sampled = select(&report, Level::MIN, seed, "stockfish-16");
    assert_ne!(sampled.rank, 0);
    assert_eq!(sampled.mv, Some(report.lines()[sampled.rank].mv));
    assert_eq!(
        sampled.engine_best,
        Some(report.best().mv),
        "the engine's own first line is recorded beside the layer's choice"
    );
    assert_eq!(sampled.seed, seed);
    assert_eq!(sampled.level, Level::MIN);
    assert_eq!(sampled.engine_id, "stockfish-16");
    assert_eq!(sampled.book_set, None);

    let capped = select(&report, Level::MAX, 1, "stockfish-16");
    assert_eq!(capped.reason, Reason::LevelCap);
    assert_eq!(capped.rank, 0);
    assert_eq!(capped.mv, Some(report.best().mv));
}

#[test]
fn a_book_move_records_the_book_set_and_no_engine_line() {
    let book = move_select::Selection::from_book(mv("d7d5"), "eco-2024", 7, "stockfish-16");
    assert_eq!(book.reason, Reason::Book);
    assert_eq!(book.mv, Some(mv("d7d5")));
    assert_eq!(book.book_set.as_deref(), Some("eco-2024"));
    assert_eq!(
        book.engine_best, None,
        "the book answered before the engine was asked"
    );
}

#[test]
fn a_resignation_and_a_draw_offer_are_decisions_without_moves() {
    let report = report(FOUR_LINES);
    let resigned = move_select::Selection::resign(&report, 3, Level::MIN, "engine");
    assert_eq!(resigned.reason, Reason::Resign);
    assert_eq!(resigned.mv, None);
    assert_eq!(resigned.engine_best, Some(report.best().mv));

    let offered = move_select::Selection::draw_offer(&report, 4, Level::MIN, "engine");
    assert_eq!(offered.reason, Reason::DrawOffer);
    assert_eq!(offered.mv, None);
}

#[test]
fn a_mate_outranks_any_amount_of_material() {
    let mate_in_one = Score::Mate(1);
    let mate_in_three = Score::Mate(3);
    let mated_in_one = Score::Mate(-1);
    let mated_in_five = Score::Mate(-5);
    let won = Score::Cp(9_000);
    let lost = Score::Cp(-9_000);

    assert!(mate_in_one > mate_in_three);
    assert!(mate_in_three > won);
    assert!(won > lost);
    assert!(lost > mated_in_five);
    assert!(mated_in_five > mated_in_one, "surviving longer is better");
    assert_eq!(mate_in_one.loss_vs(&mate_in_one), 0);
    assert_eq!(lost.loss_vs(&won), 18_000);
    assert!(
        won.loss_vs(&mate_in_one) > 900_000,
        "missing a forced mate is not a matter of a few pawns"
    );
    assert!(mated_in_one.loss_vs(&mate_in_one) > 2_000_000);
}

#[test]
fn levels_run_one_to_twenty_and_the_curves_are_monotone() {
    assert_eq!(Level::new(0), None);
    assert_eq!(Level::new(21), None);
    assert_eq!(Level::new(1), Some(Level::MIN));
    assert_eq!(Level::new(20), Some(Level::MAX));
    assert_eq!(Level::MIN.get(), 1);
    assert_eq!(Level::MAX.get(), 20);

    for level in 2..=20 {
        let lower = Level::new(level - 1).unwrap();
        let higher = Level::new(level).unwrap();
        assert!(higher.elo() >= lower.elo());
        assert!(higher.depth() >= lower.depth());
        assert!(higher.nodes() > lower.nodes());
        assert!(higher.epsilon_bp() <= lower.epsilon_bp());
        assert!(higher.temperature_cp() <= lower.temperature_cp());
        assert!(higher.multipv_width() <= lower.multipv_width());
    }
    assert_eq!(Level::MAX.epsilon_bp(), 0);
    assert!(Level::MIN.epsilon_bp() > 5_000);
}

#[test]
fn an_empty_report_is_not_an_answer() {
    assert!(EngineReport::new(Vec::new()).is_none());
}
