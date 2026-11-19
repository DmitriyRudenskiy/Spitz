//! The seeded random source, owned by this crate on purpose.
//!
//! A ladder regression match replayed next year has to reproduce move for move, so the
//! generator is fifteen lines we control rather than a dependency whose algorithm may
//! be retuned in a point release. SplitMix64: small, fast, and the whole state is one
//! number, which is all a game's metadata needs to store to replay it.

/// SplitMix64. State is the whole generator, so a seed is a complete save game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

/// One generator per game, handing out a fresh seed per decision, so a game replays by
/// replaying the stream rather than by remembering twenty seeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedStream {
    rng: Rng,
}

const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

impl Rng {
    /// Any seed is a valid start, including zero — the first output mixes it anyway.
    pub const fn seeded(seed: u64) -> Self {
        Rng { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A draw from `0..n` with no bias. The rejection is nearly never taken.
    pub fn below(&mut self, n: u64) -> u64 {
        debug_assert!(n > 0, "an empty range has nothing to draw");
        let limit = u64::MAX - (u64::MAX % n);
        loop {
            let x = self.next_u64();
            if x < limit {
                return x % n;
            }
        }
    }

    /// One in ten thousand of the whole span, which is how the level tables express a
    /// probability: basis points, integers, no float in the deciding path.
    pub fn basis_points(&mut self) -> u64 {
        self.below(10_000)
    }
}

impl SeedStream {
    pub const fn seeded(seed: u64) -> Self {
        SeedStream {
            rng: Rng::seeded(seed),
        }
    }

    pub fn next_seed(&mut self) -> u64 {
        self.rng.next_u64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_seed_replays_the_same_stream() {
        let mut a = Rng::seeded(7);
        let mut b = Rng::seeded(7);
        let mut c = Rng::seeded(8);
        let first = a.next_u64();
        assert_eq!(first, b.next_u64());
        assert_ne!(first, c.next_u64());
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn a_draw_stays_inside_its_range() {
        let mut rng = Rng::seeded(1);
        for n in 1..100u64 {
            for _ in 0..200 {
                assert!(rng.below(n) < n);
            }
        }
    }
}
