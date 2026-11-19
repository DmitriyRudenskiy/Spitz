//! What an engine says a line is worth.
//!
//! Scores arrive from the engine in the point of view of the side to move, and this layer
//! keeps it that way: nothing here flips colours, because every comparison it makes is
//! between lines of one position.

/// Material in centipawns, or a forced mate in `n` — negative when the side to move is
/// the one being mated, which is what an engine reports for a losing line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Score {
    Cp(i32),
    Mate(i16),
}

/// A mate is not "a very large centipawn score", it is a different kind of claim, so it
/// is pinned far outside anything an engine will ever report as material. The number is
/// only a floor under the ordering and the loss weighting; no test should read it.
const MATE: i64 = 1_000_000;

impl Score {
    /// A single number to order by, so a mate and a material lead are comparable and a
    /// closer mate beats a slower one in both directions.
    fn strength(&self) -> i64 {
        match self {
            Score::Cp(v) => i64::from(*v),
            Score::Mate(n) if *n > 0 => MATE + 100 - i64::from(*n),
            Score::Mate(n) if *n < 0 => -MATE - 100 + i64::from(n.abs()),
            Score::Mate(_) => -MATE,
        }
    }

    /// How much worse this line is than `best`, in centipawns, for the sampling weights.
    /// Zero for a line that is not worse, so a mis-ordered report cannot invent a loss.
    pub fn loss_vs(&self, best: &Score) -> i64 {
        (best.strength() - self.strength()).max(0)
    }

    /// Whether this is the forced-mate kind of score, which the UI shows as "mate in 4"
    /// rather than as a number of pawns.
    pub fn is_mate(&self) -> bool {
        matches!(self, Score::Mate(_))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.strength().cmp(&other.strength())
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
