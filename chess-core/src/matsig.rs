//! Material and home-pawn signatures: the compact position descriptors the library filters
//! and the opening-book lookup use.
//!
//! Layout studied from `src/matsig.h` of the reference suite; re-expressed here in Rust with
//! no source copied (see `NOTICE.md`). Twelve bits per colour, kings absent because every
//! legal position has one and a bit for it would only cost space.

use std::fmt;

/// Which army. Local to this module so the signatures can exist before the board does; the
/// board's own colour type converts to it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    White,
    Black,
}

/// Piece types that a signature counts. Kings are not counted.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
}

const PAWN: usize = 0;
const KNIGHT: usize = 1;
const BISHOP: usize = 2;
const ROOK: usize = 3;
const QUEEN: usize = 4;

/// Widest field: pawns get four bits, every other kind two.
const MAX_PAWNS: u8 = 15;
const MAX_PIECES: u8 = 3;

/// The twelve-bits-per-colour piece-count signature. Bit layout, low to high: black pawns (4),
/// black knights, bishops, rooks, queens (2 each), then the same five fields for white.
#[derive(Clone, Copy, PartialEq, Eq, Default, Hash, Debug)]
pub struct MaterialSignature(u32);

/// One bit per file per colour: set when that colour still has a pawn on its home rank there.
/// Bit 7..0 are black's a..h, bit 15..8 white's, so a colour flip is a byte swap.
#[derive(Clone, Copy, PartialEq, Eq, Default, Hash, Debug)]
pub struct HomePawnSignature(u16);

impl MaterialSignature {
    /// No pieces at all.
    pub const EMPTY: Self = Self(0);

    /// The standard opening position: 8/2/2/2/1 per colour.
    pub const START: Self = Self(0x006A_86A8);

    /// The same signature with the colours exchanged.
    pub const fn flip(self) -> Self {
        Self((self.0 >> 12) | ((self.0 & 0xFFF) << 12))
    }

    /// The stored count of one kind for one colour, saturating at the field width.
    pub const fn count(self, side: Side, kind: PieceKind) -> u8 {
        let shift = 12 * side_half(side) + field_shift(kind);
        ((self.0 >> shift) & field_mask(kind)) as u8
    }
}

impl HomePawnSignature {
    /// No pawn on either home rank.
    pub const EMPTY_RANKS: Self = Self(0);

    /// All sixteen pawns on their home ranks.
    pub const START: Self = Self(0xFFFF);

    /// The same signature with the colours exchanged: the per-file bits mirrored across the
    /// two colours.
    pub const fn flip(self) -> Self {
        Self(self.0.swap_bytes())
    }

    /// Whether `side` has a pawn on its home rank on `file` (0 = a).
    pub const fn has_pawn(self, side: Side, file: u8) -> bool {
        (self.0 >> (8 * side_half(side) + file as u32)) & 1 == 1
    }
}

/// Pack per-colour piece counts into a signature. Counts above what a field holds saturate,
/// and the second element reports that the signature is inexact rather than silently wrong.
///
/// `counts` is ordered [`PieceKind::Pawn`], Knight, Bishop, Rook, Queen.
pub fn material_signature(white: [u8; 5], black: [u8; 5]) -> (MaterialSignature, bool) {
    let mut packed = 0u32;
    let mut inexact = false;
    for (side, counts) in [(Side::White, white), (Side::Black, black)] {
        for (kind, count) in [
            (PieceKind::Pawn, counts[PAWN]),
            (PieceKind::Knight, counts[KNIGHT]),
            (PieceKind::Bishop, counts[BISHOP]),
            (PieceKind::Rook, counts[ROOK]),
            (PieceKind::Queen, counts[QUEEN]),
        ] {
            let cap = match kind {
                PieceKind::Pawn => MAX_PAWNS,
                _ => MAX_PIECES,
            };
            inexact |= count > cap;
            packed |= (count.min(cap) as u32) << (12 * side_half(side) + field_shift(kind));
        }
    }
    (MaterialSignature(packed), inexact)
}

/// Build a home-pawn signature from the set of files that still hold a home-rank pawn.
/// Bit `f` of each colour's byte is file `a + f`.
pub const fn home_pawn_signature(white_files: u8, black_files: u8) -> HomePawnSignature {
    HomePawnSignature((black_files as u16) | ((white_files as u16) << 8))
}

/// Black occupies the low half of both signatures, white the high half.
const fn side_half(side: Side) -> u32 {
    match side {
        Side::Black => 0,
        Side::White => 1,
    }
}

const fn field_shift(kind: PieceKind) -> u32 {
    match kind {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 4,
        PieceKind::Bishop => 6,
        PieceKind::Rook => 8,
        PieceKind::Queen => 10,
    }
}

const fn field_mask(kind: PieceKind) -> u32 {
    match kind {
        PieceKind::Pawn => 0xF,
        _ => 0x3,
    }
}

/// The change one move makes to the two signatures. A quiet move makes neither, so it is
/// `None`; a capture is `Material` with `-1`, a promotion is a `-1` and a `+1`.
#[derive(Clone, Copy, Debug)]
pub enum Delta {
    /// `count` is `+1` for a piece arriving on the board, `-1` for one leaving it.
    Material {
        side: Side,
        kind: PieceKind,
        count: i8,
    },
    /// A pawn arriving on or leaving its home-rank square.
    HomePawn {
        side: Side,
        file: u8,
        present: bool,
    },
}

/// The signatures, updated one move at a time instead of recomputed from the position.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct SignatureTracker {
    counts: [[u8; 5]; 2],
    home_pawns: HomePawnSignature,
    inexact: bool,
}

impl SignatureTracker {
    pub const fn new() -> Self {
        Self {
            counts: [[0; 5]; 2],
            home_pawns: HomePawnSignature::EMPTY_RANKS,
            inexact: false,
        }
    }

    /// Apply one move. Counts never wrap: they stop at the field width, and `inexact` records
    /// that they did.
    pub fn apply(&mut self, delta: Delta) {
        match delta {
            Delta::Material { side, kind, count } => {
                let field = counts_field(side, kind);
                let cap = match kind {
                    PieceKind::Pawn => MAX_PAWNS,
                    _ => MAX_PIECES,
                };
                let current = self.counts[field.0][field.1];
                let next = if count < 0 {
                    current.saturating_sub(1)
                } else {
                    if current >= cap {
                        self.inexact = true;
                    }
                    current.saturating_add(1).min(cap)
                };
                self.counts[field.0][field.1] = next;
            }
            Delta::HomePawn { side, file, present } => {
                let bit = 1u16 << (8 * side_half(side) + file.min(7) as u32);
                self.home_pawns.0 = if present {
                    self.home_pawns.0 | bit
                } else {
                    self.home_pawns.0 & !bit
                };
            }
        }
    }

    pub fn signature(&self) -> (MaterialSignature, bool) {
        (
            MaterialSignature(
                material_signature(self.counts[0], self.counts[1]).0 .0,
            ),
            self.inexact,
        )
    }

    pub const fn home_pawns(&self) -> HomePawnSignature {
        self.home_pawns
    }

    /// The standard opening position, the state a game's first position has.
    pub fn at_start() -> Self {
        let mut tracker = Self::new();
        for side in [Side::White, Side::Black] {
            for (kind, count) in [
                (PieceKind::Pawn, 8),
                (PieceKind::Knight, 2),
                (PieceKind::Bishop, 2),
                (PieceKind::Rook, 2),
                (PieceKind::Queen, 1),
            ] {
                for _ in 0..count {
                    tracker.apply(Delta::Material { side, kind, count: 1 });
                }
            }
            for file in 0..8 {
                tracker.apply(Delta::HomePawn {
                    side,
                    file,
                    present: true,
                });
            }
        }
        tracker
    }
}

const fn counts_field(side: Side, kind: PieceKind) -> (usize, usize) {
    let side_index = match side {
        Side::White => 0,
        Side::Black => 1,
    };
    let kind_index = match kind {
        PieceKind::Pawn => PAWN,
        PieceKind::Knight => KNIGHT,
        PieceKind::Bishop => BISHOP,
        PieceKind::Rook => ROOK,
        PieceKind::Queen => QUEEN,
    };
    (side_index, kind_index)
}

impl fmt::Display for MaterialSignature {
    /// The printable form used in logs and in the cross-table's debug output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:06X}", self.0 & 0xFF_FFFF)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const START_COUNTS: [u8; 5] = [8, 2, 2, 2, 1];

    #[test]
    fn standard_start_packs_the_expected_bits() {
        let (sig, inexact) = material_signature(START_COUNTS, START_COUNTS);
        assert_eq!(sig, MaterialSignature::START);
        assert!(!inexact);
        assert_eq!(sig.count(Side::White, PieceKind::Pawn), 8);
        assert_eq!(sig.count(Side::Black, PieceKind::Queen), 1);
        assert_eq!(sig.count(Side::White, PieceKind::Rook), 2);
    }

    #[test]
    fn flipping_colours_swaps_the_two_halves() {
        let mut white = [0u8; 5];
        let mut black = [0u8; 5];
        white[QUEEN] = 1;
        white[ROOK] = 2;
        black[BISHOP] = 2;
        let (sig, _) = material_signature(white, black);
        let flipped = sig.flip();
        assert_eq!(flipped.count(Side::Black, PieceKind::Queen), 1);
        assert_eq!(flipped.count(Side::Black, PieceKind::Rook), 2);
        assert_eq!(flipped.count(Side::White, PieceKind::Bishop), 2);
        assert_eq!(flipped.flip(), sig);

        let home = home_pawn_signature(0b0000_0011, 0b1000_0000);
        assert!(home.has_pawn(Side::White, 0));
        assert!(home.has_pawn(Side::Black, 7));
        let flipped = home.flip();
        assert!(flipped.has_pawn(Side::Black, 0));
        assert!(flipped.has_pawn(Side::Black, 1));
        assert!(flipped.has_pawn(Side::White, 7));
        assert!(!flipped.has_pawn(Side::White, 0));
        assert_eq!(flipped.flip(), home);
    }

    #[test]
    fn over_cap_counts_saturate_and_are_marked_inexact() {
        let (sig, inexact) = material_signature([8, 9, 0, 0, 0], [0; 5]);
        assert!(inexact);
        assert_eq!(sig.count(Side::White, PieceKind::Knight), MAX_PIECES);
        let (pawns, inexact) = material_signature([16, 0, 0, 0, 0], [0; 5]);
        assert!(inexact);
        assert_eq!(pawns.count(Side::White, PieceKind::Pawn), MAX_PAWNS);
    }

    #[test]
    fn incremental_updates_match_a_fresh_computation() {
        // A deterministic pseudo-random move stream: every position is reached by applying one
        // delta to the tracker and is also computed from scratch from an independent count set.
        let mut tracker = SignatureTracker::at_start();
        let mut white = START_COUNTS;
        let mut black = START_COUNTS;
        let mut white_files = 0xFFu8;
        let mut black_files = 0xFFu8;
        let mut state = 0x2545_F491u64;
        let mut expected_inexact = false;
        let start = std::time::Instant::now();
        for _ in 0..1_000_000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let roll = (state >> 59) as u8;
            let side = if roll & 1 == 0 { Side::White } else { Side::Black };
            let file = ((state >> 40) as u8) & 7;
            let delta = match roll >> 1 {
                0 | 1 => Delta::HomePawn {
                    side,
                    file,
                    present: (state >> 33) & 1 == 0,
                },
                _ => {
                    let kind = match (state >> 24) as u8 % 5 {
                        0 => PieceKind::Pawn,
                        1 => PieceKind::Knight,
                        2 => PieceKind::Bishop,
                        3 => PieceKind::Rook,
                        _ => PieceKind::Queen,
                    };
                    let adding = (state >> 17) & 1 == 0;
                    let counts = match side {
                        Side::White => &mut white,
                        Side::Black => &mut black,
                    };
                    let index = counts_field(side, kind).1;
                    let cap = match kind {
                        PieceKind::Pawn => MAX_PAWNS,
                        _ => MAX_PIECES,
                    };
                    if adding {
                        // Once a field saturates the true count is lost for good, so the
                        // expectation latches exactly like the tracker does.
                        expected_inexact |= counts[index] >= cap;
                        if counts[index] < cap {
                            counts[index] += 1;
                        }
                    } else {
                        counts[index] = counts[index].saturating_sub(1);
                    }
                    Delta::Material {
                        side,
                        kind,
                        count: if adding { 1 } else { -1 },
                    }
                }
            };
            if let Delta::HomePawn { side, file, present } = delta {
                let files = match side {
                    Side::White => &mut white_files,
                    Side::Black => &mut black_files,
                };
                if present {
                    *files |= 1 << file;
                } else {
                    *files &= !(1 << file);
                }
            }
            tracker.apply(delta);
            let expected = material_signature(white, black).0;
            assert_eq!(tracker.signature(), (expected, expected_inexact));
            assert_eq!(tracker.home_pawns(), home_pawn_signature(white_files, black_files));
        }
        // The budget the library filter is designed around: 100 ns a position.
        if !cfg!(debug_assertions) {
            let elapsed = start.elapsed();
            assert!(elapsed.as_millis() < 100, "1M updates took {elapsed:?}");
        }
    }

    #[test]
    fn start_position_signatures() {
        let tracker = SignatureTracker::at_start();
        assert_eq!(tracker.signature(), (MaterialSignature::START, false));
        assert_eq!(tracker.home_pawns(), HomePawnSignature::START);
    }
}
