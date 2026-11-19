//! The layer that decides what the opponent actually plays.
//!
//! Spitz ships no engine. A third-party engine reports ranked lines and scores, and this
//! crate turns that report into a move that matches the configured level — by choosing
//! among the reported lines rather than by trusting the engine's first one. Two knobs do
//! the work: how often the layer looks away from the best line at all, and how far down
//! the report it wanders when it does. Both come from the level, both are integers, and
//! every decision is drawn from a seeded generator, so a game can be replayed move for
//! move from the seed its metadata carries.
//!
//! The level tables are a starting curve, not a measurement. What "level 8" means is
//! settled by the ladder harness, per engine; until that harness has run, the numbers
//! here are only a monotone shape for it to calibrate against.

pub mod options;
pub mod rng;
pub mod score;

pub use options::{EngineOptions, Mechanism, Plan, Setting};
pub use rng::{Rng, SeedStream};
pub use score::Score;

use chess_core::board::Move;

/// How many levels there are. The spec's floor is twenty; the ladder is what makes them
/// twenty distinct things.
pub const LEVELS: u8 = 20;

/// How far the layer looks away from the best line, in basis points of one draw per
/// decision. Zero at the top level, where the opponent plays the engine's first line.
const EPSILON_BP: [u64; LEVELS as usize] = [
    7500, 6878, 6277, 5697, 5138, 4601, 4087, 3595, 3128, 2686, 2269, 1879, 1518, 1186, 886, 620,
    391, 205, 67, 0,
];

/// How much of a score gap still counts as "close enough to play", in centipawns. The
/// weight of a line is `temperature / (temperature + loss)`, so a line one temperature
/// away from the best is worth half as much as the best and a line ten is nearly dead.
const TEMPERATURE_CP: [i64; LEVELS as usize] = [
    1200, 1114, 1030, 948, 868, 789, 714, 640, 569, 500, 435, 372, 312, 255, 202, 153, 109, 70, 39,
    20,
];

/// The Elo this layer asks a limiting engine for. A curve through the human range, with
/// most of its levels spent below 2000 because that is where the opponent will be played.
const ELO: [i32; LEVELS as usize] = [
    800, 812, 842, 887, 945, 1017, 1101, 1198, 1306, 1425, 1556, 1697, 1849, 2012, 2185, 2368,
    2561, 2765, 2977, 3200,
];

/// Lines to ask for. A wide sample at the weak end, one line at the strong end where
/// nothing is going to be sampled anyway.
const MULTIPV: [u32; LEVELS as usize] =
    [5, 4, 4, 4, 4, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1];

/// Search depth cap, for engines without Elo limiting.
const DEPTH: [u32; LEVELS as usize] = [
    1, 2, 4, 5, 7, 8, 10, 11, 13, 14, 16, 17, 19, 20, 22, 23, 25, 26, 28, 30,
];

/// Node cap, doubling per level. The top level's budget is half a billion nodes, which a
/// strong engine spends in seconds; the point is the ordering, not the ceiling.
const NODES: [u64; LEVELS as usize] = [
    1_000,
    2_000,
    4_000,
    8_000,
    16_000,
    32_000,
    64_000,
    128_000,
    256_000,
    512_000,
    1_024_000,
    2_048_000,
    4_096_000,
    8_192_000,
    16_384_000,
    32_768_000,
    65_536_000,
    131_072_000,
    262_144_000,
    524_288_000,
];

/// The numerator of the weight, so the division keeps resolution in integers.
const WEIGHT_SCALE: i64 = 1_000_000;

/// A strength level, one through [`LEVELS`]. Higher is stronger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Level(u8);

impl Level {
    pub const MIN: Level = Level(1);
    pub const MAX: Level = Level(LEVELS);

    /// The level a player chose, if they chose one.
    pub const fn new(level: i32) -> Option<Level> {
        if level >= 1 && level <= LEVELS as i32 {
            Some(Level(level as u8))
        } else {
            None
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    const fn index(self) -> usize {
        (self.0 - 1) as usize
    }

    /// The chance, in basis points, that this level looks away from the best line.
    pub const fn epsilon_bp(self) -> u64 {
        EPSILON_BP[self.index()]
    }

    /// How wide a score gap this level still considers playable, in centipawns.
    pub const fn temperature_cp(self) -> i64 {
        TEMPERATURE_CP[self.index()]
    }

    pub const fn elo(self) -> i32 {
        ELO[self.index()]
    }

    pub const fn multipv_width(self) -> u32 {
        MULTIPV[self.index()]
    }

    pub const fn depth(self) -> u32 {
        DEPTH[self.index()]
    }

    pub const fn nodes(self) -> u64 {
        NODES[self.index()]
    }
}

/// One line of the engine's report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub mv: Move,
    pub score: Score,
}

/// What the engine reported for this position: ranked lines, best first, in the order the
/// engine gave them. The layer never re-ranks the report; it chooses among it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineReport {
    lines: Vec<Line>,
}

impl EngineReport {
    /// A report with nothing in it is not a report — an engine that returned no lines has
    /// not answered the question, and pretending otherwise would invent a move.
    pub fn new(lines: Vec<Line>) -> Option<EngineReport> {
        if lines.is_empty() {
            None
        } else {
            Some(EngineReport { lines })
        }
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn best(&self) -> Line {
        self.lines[0]
    }
}

/// Why the opponent did what it did, in the player's terms. Recorded with every decision
/// so a game can be explained after the fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// The book answered before the engine was asked.
    Book,
    /// The engine's first line, which is what the level's caps ask for.
    LevelCap,
    /// A line below the best, drawn from the level's sampling width.
    SampledLine,
    /// A move forced by the handicap the game is played under.
    HandicapConstraint,
    /// The opponent gave up. No move.
    Resign,
    /// The opponent offered a draw. No move.
    DrawOffer,
}

/// The opponent's decision, with everything a replay or an explanation needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// The move to play, or `None` when the decision was a resignation or a draw offer.
    pub mv: Option<Move>,
    pub reason: Reason,
    /// Which line of the report was chosen, zero for the engine's own first choice.
    pub rank: usize,
    /// The engine's first-ranked line, recorded separately so the layer's choice can be
    /// compared against it later. `None` when no engine was asked, which is the book.
    pub engine_best: Option<Move>,
    pub seed: u64,
    pub level: Level,
    pub engine_id: String,
    /// The book set's content hash, when the book answered.
    pub book_set: Option<String>,
}

impl Selection {
    /// The book answered, so no engine was asked and no line was sampled.
    pub fn from_book(
        mv: Move,
        book_set: impl Into<String>,
        seed: u64,
        engine_id: &str,
    ) -> Selection {
        Selection {
            mv: Some(mv),
            reason: Reason::Book,
            rank: 0,
            engine_best: None,
            seed,
            level: Level::MIN,
            engine_id: engine_id.to_string(),
            book_set: Some(book_set.into()),
        }
    }

    /// The opponent gave up. The report is kept so the record still shows what the engine
    /// thought at the moment of the decision.
    pub fn resign(report: &EngineReport, seed: u64, level: Level, engine_id: &str) -> Selection {
        Selection::record(report, None, Reason::Resign, 0, seed, level, engine_id)
    }

    /// The opponent offered a draw.
    pub fn draw_offer(
        report: &EngineReport,
        seed: u64,
        level: Level,
        engine_id: &str,
    ) -> Selection {
        Selection::record(report, None, Reason::DrawOffer, 0, seed, level, engine_id)
    }

    fn record(
        report: &EngineReport,
        mv: Option<Move>,
        reason: Reason,
        rank: usize,
        seed: u64,
        level: Level,
        engine_id: &str,
    ) -> Selection {
        Selection {
            mv,
            reason,
            rank,
            engine_best: Some(report.best().mv),
            seed,
            level,
            engine_id: engine_id.to_string(),
            book_set: None,
        }
    }
}

/// Choose the opponent's move from what the engine reported.
///
/// The draw is one coin flip and, when it lands on "look away", one weighted pick over
/// the report — so the cost is a handful of integer operations, nowhere near the fifty
/// milliseconds the budget allows, and it never touches a clock or a thread.
pub fn select(report: &EngineReport, level: Level, seed: u64, engine_id: &str) -> Selection {
    let mut rng = Rng::seeded(seed);
    // The coin is only flipped when there is a choice to make: a top level, or a report
    // with one line in it, has nowhere to look.
    let looks_away =
        level.epsilon_bp() > 0 && report.lines.len() > 1 && rng.basis_points() < level.epsilon_bp();
    let rank = if looks_away {
        weighted_rank(report, level, &mut rng)
    } else {
        0
    };
    let reason = if rank == 0 {
        Reason::LevelCap
    } else {
        Reason::SampledLine
    };
    Selection::record(
        report,
        Some(report.lines[rank].mv),
        reason,
        rank,
        seed,
        level,
        engine_id,
    )
}

/// Draw a line from the report with weight `temperature / (temperature + loss)`: the best
/// line is the most likely, a near-equal line nearly as much, and a blunder effectively
/// dead. That shape is the difference between a weak opponent and a random one.
fn weighted_rank(report: &EngineReport, level: Level, rng: &mut Rng) -> usize {
    let temperature = level.temperature_cp().max(1);
    let best = report.best().score;
    let mut weights = Vec::with_capacity(report.lines().len());
    let mut total: i64 = 0;
    for line in report.lines() {
        let loss = line.score.loss_vs(&best);
        let weight = WEIGHT_SCALE * temperature / (temperature + loss);
        total += weight;
        weights.push(weight);
    }
    // The best line always carries a full weight, so the total is never zero.
    let mut draw = rng.below(total as u64) as i64;
    for (rank, weight) in weights.iter().enumerate() {
        if draw < *weight {
            return rank;
        }
        draw -= *weight;
    }
    weights.len() - 1
}
