//! The packed 48-byte header record: one per game, holding everything a view or a filter
//! pass needs without touching the game text.
//!
//! `docs/header-record-format.md` is the format note this code implements. The field set was
//! studied from the reference suite's index entry and re-expressed here (see `NOTICE.md`).

use chess_core::date::Date;
use chess_core::matsig::MaterialSignature;
use chess_core::pgn::GameResult;
use std::fmt;

/// Bytes per record, and per game. Pinned by a test and by [`assert_layout`].
pub const HEADER_RECORD_SIZE: usize = 48;

/// The version byte at the head of an index file. A bump means rebuild, not translate; see the
/// format note.
pub const INDEX_FORMAT_VERSION: u8 = 1;

/// How a rating was obtained. A rating of zero is unknown whatever this says.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RatingKind {
    #[default]
    Unknown,
    Elo,
    FideOrUscf,
    Brcp,
    Uscf,
    Issr,
    Program,
}

impl RatingKind {
    const fn code(self) -> u32 {
        match self {
            RatingKind::Unknown => 0,
            RatingKind::Elo => 1,
            RatingKind::FideOrUscf => 2,
            RatingKind::Brcp => 3,
            RatingKind::Uscf => 4,
            RatingKind::Issr => 5,
            RatingKind::Program => 6,
        }
    }

    const fn from_code(code: u32) -> Option<Self> {
        Some(match code {
            0 => RatingKind::Unknown,
            1 => RatingKind::Elo,
            2 => RatingKind::FideOrUscf,
            3 => RatingKind::Brcp,
            4 => RatingKind::Uscf,
            5 => RatingKind::Issr,
            6 => RatingKind::Program,
            _ => return None,
        })
    }
}

/// Zero means no result yet, so a record nobody filled in reads as unfinished rather than as a
/// white win.
const fn result_code(result: GameResult) -> u32 {
    match result {
        GameResult::Unfinished => 0,
        GameResult::WhiteWins => 1,
        GameResult::Draw => 2,
        GameResult::BlackWins => 3,
    }
}

const fn result_from_code(code: u32) -> GameResult {
    match code {
        1 => GameResult::WhiteWins,
        2 => GameResult::Draw,
        3 => GameResult::BlackWins,
        _ => GameResult::Unfinished,
    }
}

/// The counts the record carries, saturating at 15 with 15 meaning "15 or more".
const COUNT_CEILING: u8 = 15;
/// The half-move count saturates at 1023, meaning "1023 or more".
const HALF_MOVE_CEILING: u16 = 1023;
/// Ratings live in 12 bits, so 4095 is the highest storeable rating.
pub const MAX_RATING: u16 = 4095;

/// One game's index entry. Every field is public; the record is a value with no invariants
/// beyond the widths the format gives it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct HeaderRecord {
    /// Byte offset of the game record in the data file.
    pub offset: u64,
    /// Bytes of the game record; zero means the game is not stored yet.
    pub length: u32,
    pub white: u32,
    pub black: u32,
    pub event: u32,
    pub round: u32,
    pub site: u32,
    pub white_rating: u16,
    pub black_rating: u16,
    pub white_rating_kind: RatingKind,
    pub black_rating_kind: RatingKind,
    pub date: Date,
    pub event_date: Date,
    pub result: GameResult,
    /// One bit per flag character, positioned as the format note lists them.
    pub flags: u32,
    /// Code table index; zero is unclassified.
    pub eco: u16,
    pub final_material: MaterialSignature,
    pub comment_count: u8,
    pub variation_count: u8,
    pub nag_count: u8,
    pub half_moves: u16,
}

impl HeaderRecord {
    /// Encode to the documented 48 bytes. Values wider than their field saturate rather than
    /// truncating, so a bad offset cannot quietly alias another game's record.
    pub fn encode(&self) -> [u8; HEADER_RECORD_SIZE] {
        let mut out = Bits::write();
        out.push(self.offset.min((1u64 << 48) - 1), 48);
        out.push(self.length.min((1 << 18) - 1) as u64, 18);
        out.push(self.white.min((1 << 28) - 1) as u64, 28);
        out.push(self.black.min((1 << 28) - 1) as u64, 28);
        out.push(self.event.min((1 << 28) - 1) as u64, 28);
        out.push(self.round.min((1 << 28) - 1) as u64, 28);
        out.push(self.site as u64, 32);
        out.push(self.white_rating.min(MAX_RATING) as u64, 12);
        out.push(self.black_rating.min(MAX_RATING) as u64, 12);
        out.push(self.white_rating_kind.code() as u64, 3);
        out.push(self.black_rating_kind.code() as u64, 3);
        out.push(u64::from(self.date.sort_key()), 20);
        out.push(u64::from(self.event_date.sort_key()), 20);
        out.push(result_code(self.result) as u64, 2);
        out.push(self.flags.min((1 << 22) - 1) as u64, 22);
        out.push(self.eco as u64, 16);
        out.push(self.final_material.raw() as u64, 24);
        out.push(self.comment_count.min(COUNT_CEILING) as u64, 4);
        out.push(self.variation_count.min(COUNT_CEILING) as u64, 4);
        out.push(self.nag_count.min(COUNT_CEILING) as u64, 4);
        out.push(self.half_moves.min(HALF_MOVE_CEILING) as u64, 10);
        out.bytes
    }

    /// Decode 48 bytes written by [`Self::encode`].
    pub fn decode(bytes: &[u8]) -> Result<Self, RecordError> {
        if bytes.len() < HEADER_RECORD_SIZE {
            return Err(RecordError::TooShort { len: bytes.len() });
        }
        let mut in_ = Bits::from_bytes(bytes);
        let record = Self {
            offset: in_.take(48),
            length: in_.take(18) as u32,
            white: in_.take(28) as u32,
            black: in_.take(28) as u32,
            event: in_.take(28) as u32,
            round: in_.take(28) as u32,
            site: in_.take(32) as u32,
            white_rating: in_.take(12) as u16,
            black_rating: in_.take(12) as u16,
            white_rating_kind: RatingKind::from_code(in_.take(3) as u32)
                .ok_or(RecordError::UnknownRatingKind)?,
            black_rating_kind: RatingKind::from_code(in_.take(3) as u32)
                .ok_or(RecordError::UnknownRatingKind)?,
            date: Date::from_sort_key(in_.take(20) as u32),
            event_date: Date::from_sort_key(in_.take(20) as u32),
            result: result_from_code(in_.take(2) as u32),
            flags: in_.take(22) as u32,
            eco: in_.take(16) as u16,
            final_material: MaterialSignature::from_raw(in_.take(24) as u32),
            comment_count: in_.take(4) as u8,
            variation_count: in_.take(4) as u8,
            nag_count: in_.take(4) as u8,
            half_moves: in_.take(10) as u16,
        };
        if in_.take(18) != 0 {
            return Err(RecordError::ReservedBitsSet);
        }
        Ok(record)
    }
}

/// The record's own size, so callers never hard-code the constant.
pub const fn record_size() -> usize {
    HEADER_RECORD_SIZE
}

/// Panics if the layout drifts from the documented 48 bytes. Called from a test and usable at
/// start-up by a library that is about to memory-map a file.
pub fn assert_layout() {
    assert_eq!(HEADER_RECORD_SIZE, 48);
    assert_eq!(HeaderRecord::default().encode().len(), HEADER_RECORD_SIZE);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordError {
    TooShort { len: usize },
    UnknownRatingKind,
    /// A record written by a different version, or corrupt: the spare bits are never set.
    ReservedBitsSet,
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordError::TooShort { len } => {
                write!(f, "record is {len} bytes, needs {HEADER_RECORD_SIZE}")
            }
            RecordError::UnknownRatingKind => write!(f, "unknown rating kind"),
            RecordError::ReservedBitsSet => write!(f, "reserved bits set: wrong format version"),
        }
    }
}

/// Writes fields LSB-first into the record buffer.
struct Bits {
    bytes: [u8; HEADER_RECORD_SIZE],
    at: usize,
}

impl Bits {
    fn write() -> Self {
        Self { bytes: [0; HEADER_RECORD_SIZE], at: 0 }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut buf = [0u8; HEADER_RECORD_SIZE];
        buf.copy_from_slice(&bytes[..HEADER_RECORD_SIZE]);
        Self { bytes: buf, at: 0 }
    }

    fn push(&mut self, value: u64, bits: u32) {
        let mut value = value;
        let mut left = bits;
        while left > 0 {
            let free = 8 - (self.at % 8);
            let take = free.min(left as usize);
            let mask = ((1u16 << take) - 1) as u8;
            self.bytes[self.at / 8] |= ((value as u8) & mask) << (self.at % 8);
            value >>= take;
            self.at += take;
            left -= take as u32;
        }
    }

    fn take(&mut self, bits: u32) -> u64 {
        let mut value = 0u64;
        let mut left = bits;
        let mut shift = 0;
        while left > 0 {
            let free = 8 - (self.at % 8);
            let take = free.min(left as usize);
            let mask = ((1u16 << take) - 1) as u8;
            value |= (((self.bytes[self.at / 8] >> (self.at % 8)) & mask) as u64) << shift;
            shift += take;
            self.at += take;
            left -= take as u32;
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(seed: &mut u64) -> HeaderRecord {
        let mut next = |mask: u64| {
            *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (*seed >> 33) & mask
        };
        HeaderRecord {
            offset: next((1u64 << 48) - 1),
            length: (next((1 << 18) - 1)) as u32,
            white: (next((1 << 28) - 1)) as u32,
            black: (next((1 << 28) - 1)) as u32,
            event: (next((1 << 28) - 1)) as u32,
            round: (next((1 << 28) - 1)) as u32,
            site: (next(u32::MAX as u64)) as u32,
            white_rating: next(MAX_RATING as u64) as u16,
            black_rating: next(MAX_RATING as u64) as u16,
            white_rating_kind: RatingKind::from_code((next(u64::MAX) % 7) as u32).unwrap(),
            black_rating_kind: RatingKind::from_code((next(u64::MAX) % 7) as u32).unwrap(),
            date: Date::from_sort_key(next((1 << 20) - 1) as u32),
            event_date: Date::from_sort_key(next((1 << 20) - 1) as u32),
            result: result_from_code(next(3) as u32),
            flags: next((1 << 22) - 1) as u32,
            eco: next(u16::MAX as u64) as u16,
            final_material: MaterialSignature::from_raw(next((1 << 24) - 1) as u32),
            comment_count: next(15) as u8,
            variation_count: next(15) as u8,
            nag_count: next(15) as u8,
            half_moves: next(1023) as u16,
        }
    }

    #[test]
    fn a_record_is_exactly_48_bytes() {
        assert_layout();
        assert_eq!(record_size(), 48);
        assert_eq!(HeaderRecord::default().encode(), [0; 48]);
    }

    #[test]
    fn every_field_survives_a_round_trip() {
        let mut seed = 0x9E3779B97F4A7C15;
        for _ in 0..100_000 {
            let record = record(&mut seed);
            let bytes = record.encode();
            assert_eq!(HeaderRecord::decode(&bytes), Ok(record));
        }
    }

    #[test]
    fn encoding_is_canonical_so_equal_games_hold_equal_bytes() {
        let mut seed = 0x1234_5678;
        for _ in 0..10_000 {
            let bytes = record(&mut seed).encode();
            assert_eq!(HeaderRecord::decode(&bytes).unwrap().encode(), bytes);
        }
    }

    #[test]
    fn wide_values_saturate_instead_of_aliasing() {
        let record = HeaderRecord {
            offset: u64::MAX,
            length: u32::MAX,
            white: u32::MAX,
            white_rating: u16::MAX,
            comment_count: 200,
            half_moves: u16::MAX,
            ..Default::default()
        };
        let decoded = HeaderRecord::decode(&record.encode()).unwrap();
        assert_eq!(decoded.offset, (1u64 << 48) - 1);
        assert_eq!(decoded.length, (1 << 18) - 1);
        assert_eq!(decoded.white, (1 << 28) - 1);
        assert_eq!(decoded.white_rating, MAX_RATING);
        assert_eq!(decoded.comment_count, COUNT_CEILING);
        assert_eq!(decoded.half_moves, HALF_MOVE_CEILING);
    }

    #[test]
    fn a_short_or_foreign_buffer_is_refused() {
        assert_eq!(HeaderRecord::decode(&[0; 47]), Err(RecordError::TooShort { len: 47 }));
        let mut bytes = HeaderRecord::default().encode();
        bytes[47] = 0xFF;
        assert_eq!(HeaderRecord::decode(&bytes), Err(RecordError::ReservedBitsSet));
    }

    #[test]
    fn dates_and_the_material_signature_keep_their_meaning() {
        let record = HeaderRecord {
            date: Date::parse("1999.03.??").unwrap(),
            event_date: Date::parse("2011").unwrap(),
            final_material: MaterialSignature::START,
            ..Default::default()
        };
        let decoded = HeaderRecord::decode(&record.encode()).unwrap();
        assert_eq!(decoded.date, Date::parse("1999.03").unwrap());
        assert_eq!(decoded.date.day(), None);
        assert_eq!(decoded.event_date, Date::parse("2011").unwrap());
        assert_eq!(decoded.final_material, MaterialSignature::START);
    }
}
