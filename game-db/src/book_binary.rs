//! The interchange format: a book as one flat array of 16-byte big-endian records.
//!
//! ```text
//! key      8 bytes  the position's 8-byte key, big-endian
//! move     2 bytes  from (6 bits) | to (6 bits) | promotion (2 bits)
//! weight   2 bytes  how much the book likes the move
//! reserved 4 bytes  the format's spare field, carried through untouched
//! ```
//!
//! The array is sorted by key, so a position's moves are found by binary search and are contiguous.
//! The whole file is the format — there is no header, and a length that is not a multiple of sixteen
//! is the corruption that callers must be able to report.
//!
//! Two of the format's conventions are worth knowing before reading a move: castling is written as
//! the king taking its own rook, and promotion to a queen shares its code with no promotion.

use chess_core::board::{Kind, Move, Square};

pub const ENTRY_BYTES: usize = 16;

/// A book that is not a whole number of records is corrupt, and saying so is the whole error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatError {
    Trailing { bytes: usize, trailing: usize },
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FormatError::Trailing { bytes, trailing } = self;
        write!(f, "book length {bytes} is not a whole number of {ENTRY_BYTES}-byte records, {trailing} over")
    }
}

impl std::error::Error for FormatError {}

/// One record, exactly as the format holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub key: u64,
    pub mv: Move,
    pub weight: u16,
    /// The format's spare field. The native store has nowhere to put it, so the exporter takes it
    /// from here and the importer writes it back.
    pub reserved: u32,
}

impl Entry {
    pub fn new(key: u64, mv: Move, weight: u16) -> Entry {
        Entry { key, mv, weight, reserved: 0 }
    }

    fn bytes(self) -> [u8; ENTRY_BYTES] {
        let mut out = [0u8; ENTRY_BYTES];
        out[..8].copy_from_slice(&self.key.to_be_bytes());
        out[8..10].copy_from_slice(&encode_move(self.mv).to_be_bytes());
        out[10..12].copy_from_slice(&self.weight.to_be_bytes());
        out[12..].copy_from_slice(&self.reserved.to_be_bytes());
        out
    }
}

/// The two-byte move: six bits of from-square, six of to-square, and two saying what a pawn became.
/// The promotion field counts knight, bishop and rook; promotion to a queen shares the code with "no
/// promotion", and is told apart by the pawn landing on the last rank. That is the format's own
/// convention, read in `crate::book::import` where the position is known.
/// Castling, which the format writes as the king taking its own rook: e1h1, e1a1, e8h8, e8a8.
/// No other move is shaped like this, so the two squares say which it is.
const CASTLES: [(Square, Square, Square); 4] = [
    (4, 7, 6),    // e1 takes h1, that is castling kingside
    (4, 0, 2),    // e1 takes a1
    (60, 63, 62), // e8 takes h8
    (60, 56, 58), // e8 takes a8
];

pub fn encode_move(mv: Move) -> u16 {
    let mv = match CASTLES.iter().find(|(king, _, to)| *king == mv.from && *to == mv.to) {
        Some((_, rook, _)) => Move { to: *rook, ..mv },
        None => mv,
    };
    let promotion = match mv.promotion {
        Some(Kind::Knight) => 1,
        Some(Kind::Bishop) => 2,
        Some(Kind::Rook) => 3,
        // Promotion to a queen is the one thing the format cannot say; see `decode_move`.
        _ => 0,
    };
    (promotion << 12) | ((mv.from as u16) << 6) | mv.to as u16
}

pub fn decode_move(packed: u16) -> Move {
    let promotion = match packed >> 12 {
        1 => Some(Kind::Knight),
        2 => Some(Kind::Bishop),
        3 => Some(Kind::Rook),
        _ => None, // none, or a queen — see the note above
    };
    let from = ((packed >> 6) & 0x3f) as Square;
    let to = (packed & 0x3f) as Square;
    match CASTLES.iter().find(|(king, rook, _)| *king == from && *rook == to) {
        Some((king, _, castle)) => Move { from: *king, to: *castle, promotion },
        None => Move { from, to, promotion },
    }
}

/// Read a whole book. The entries come back in the file's order.
pub fn read(bytes: &[u8]) -> Result<Vec<Entry>, FormatError> {
    if !bytes.len().is_multiple_of(ENTRY_BYTES) {
        return Err(FormatError::Trailing { bytes: bytes.len(), trailing: bytes.len() % ENTRY_BYTES });
    }
    Ok(bytes
        .chunks(ENTRY_BYTES)
        .map(|record| Entry {
            key: u64::from_be_bytes(record[..8].try_into().unwrap()),
            mv: decode_move(u16::from_be_bytes(record[8..10].try_into().unwrap())),
            weight: u16::from_be_bytes(record[10..12].try_into().unwrap()),
            reserved: u32::from_be_bytes(record[12..16].try_into().unwrap()),
        })
        .collect())
}

/// Write a whole book, sorted by key as the format requires.
pub fn write(entries: &[Entry]) -> Vec<u8> {
    let mut sorted: Vec<Entry> = entries.to_vec();
    sorted.sort_unstable_by_key(|entry| entry.key);
    let mut out = Vec::with_capacity(sorted.len() * ENTRY_BYTES);
    for entry in sorted {
        out.extend_from_slice(&entry.bytes());
    }
    out
}

/// The records whose key matches, as a range into a slice sorted by key. A position's moves are
/// contiguous, so this is the whole lookup: one binary search, then a walk.
pub fn find_range(entries: &[Entry], key: u64) -> Option<std::ops::Range<usize>> {
    let first = entries.partition_point(|entry| entry.key < key);
    if entries.get(first)?.key != key {
        return None;
    }
    Some(first..entries.partition_point(|entry| entry.key <= key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::board::Board;

    fn record(key: u64, from: u8, to: u8, promotion: u16, weight: u16, reserved: u32) -> [u8; ENTRY_BYTES] {
        let mut out = [0u8; ENTRY_BYTES];
        out[..8].copy_from_slice(&key.to_be_bytes());
        let packed = (promotion << 12) | ((from as u16) << 6) | to as u16;
        out[8..10].copy_from_slice(&packed.to_be_bytes());
        out[10..12].copy_from_slice(&weight.to_be_bytes());
        out[12..].copy_from_slice(&reserved.to_be_bytes());
        out
    }

    #[test]
    fn a_record_is_sixteen_bytes_big_endian() {
        let entry = Entry {
            key: 0x0123_4567_89ab_cdef,
            mv: Move::simple(12, 20),
            weight: 0x0102,
            reserved: 0x0a0b_0c0d,
        };
        assert_eq!(
            entry.bytes(),
            [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x03, 0x14, 0x01, 0x02, 0x0a, 0x0b, 0x0c, 0x0d]
        );
        assert_eq!(read(&entry.bytes()).unwrap(), vec![entry]);
    }

    #[test]
    fn a_move_survives_its_two_bytes() {
        for (from, to, promotion) in [
            (12u8, 20u8, None),
            (0, 7, None),
            (63, 56, None),
            (48, 56, Some(Kind::Knight)),
            (48, 56, Some(Kind::Bishop)),
            (48, 56, Some(Kind::Rook)),
        ] {
            let mv = Move { from, to, promotion };
            assert_eq!(decode_move(encode_move(mv)), mv, "{mv:?}");
        }
    }

    #[test]
    fn the_starting_moves_decode_to_the_moves_they_are() {
        // The book's own encoding of the first two opening moves, from the reference reader.
        let bytes = [
            record(0, 12, 28, 0, 100, 0), // e2-e4
            record(0, 11, 19, 0, 90, 0),  // d2-d4
            record(0, 48, 56, 1, 80, 0),  // a7-a8 knight
        ];
        let book = read(&bytes.concat()).unwrap();
        assert_eq!(book[0].mv, Move::simple(12, 28));
        assert_eq!(book[1].mv, Move::simple(11, 19));
        assert_eq!(book[2].mv, Move { from: 48, to: 56, promotion: Some(Kind::Knight) });
    }

    #[test]
    fn a_file_cut_mid_record_is_reported_not_guessed() {
        let two = [record(1, 0, 0, 0, 0, 0), record(2, 0, 0, 0, 0, 0)].concat();
        for len in [1, 8, 15, 17, 31] {
            assert!(
                matches!(read(&two[..len]), Err(FormatError::Trailing { .. })),
                "{len} bytes is not a whole number of records"
            );
        }
        assert_eq!(read(&two[..16]).unwrap().len(), 1, "sixteen bytes is one record");
        assert_eq!(read(&two).unwrap().len(), 2, "and thirty-two is two");
    }

    #[test]
    fn a_position_is_found_by_one_binary_search_and_its_moves_are_contiguous() {
        let mut entries = Vec::new();
        for key in [7u64, 7, 7, 9, 9, 11] {
            entries.push(Entry::new(key, Move::simple(0, 1), 1));
        }
        assert_eq!(find_range(&entries, 7), Some(0..3));
        assert_eq!(find_range(&entries, 9), Some(3..5));
        assert_eq!(find_range(&entries, 11), Some(5..6));
        assert_eq!(find_range(&entries, 8), None);
        assert_eq!(find_range(&entries, 6), None);
        assert_eq!(find_range(&entries, 12), None);
        assert_eq!(find_range(&[], 1), None);
    }

    #[test]
    fn a_written_book_is_sorted_and_reads_back() {
        let entries = [
            Entry::new(5, Move::simple(1, 2), 3),
            Entry::new(1, Move::simple(3, 4), 4),
            Entry::new(5, Move::simple(5, 6), 5),
        ];
        let bytes = write(&entries);
        assert_eq!(bytes.len(), 3 * ENTRY_BYTES);
        let keys: Vec<u64> = read(&bytes).unwrap().iter().map(|e| e.key).collect();
        assert_eq!(keys, vec![1, 5, 5]);
        assert_eq!(find_range(&read(&bytes).unwrap(), 5), Some(1..3));
    }

    #[test]
    fn castling_is_written_as_the_king_taking_its_own_rook() {
        for (castle, rook) in [(Move::simple(4, 6), 7u8), (Move::simple(4, 2), 0), (Move::simple(60, 62), 63), (Move::simple(60, 58), 56)] {
            let packed = encode_move(castle);
            assert_eq!(packed & 0x3f, rook as u16, "the format names the rook, not the file");
            assert_eq!(decode_move(packed), castle, "and it reads back as the castle");
        }
        // A king move that is not a castle keeps its own squares.
        assert_eq!(decode_move(encode_move(Move::simple(4, 12))), Move::simple(4, 12));
    }

    #[test]
    fn the_starting_position_is_where_a_book_starts() {
        assert_eq!(Board::start().key(), 0x463b96181691fc9c);
    }
}
