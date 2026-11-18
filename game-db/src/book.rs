//! The app's own opening book, and the door in and out of the interchange format.
//!
//! A book is not a list of moves but a set of positions, and a position is named by its 8-byte key.
//! Importing therefore cannot be a loop over the records: a record says "in this position play this",
//! and whether that means anything depends on whether the position can be reached at all. So import
//! replays from the start position outwards, and a record whose position is never reached, or whose
//! move was never legal, is skipped and counted rather than dropped silently.

use chess_core::board::{rank_of, Board, Kind, Move, MoveError};
use chess_core::date::Date;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::book_binary::{self, Entry as InterchangeEntry, FormatError};

/// Shown wherever a move's result or average rating is asked for and the book does not have it. The
/// interchange format carries neither, so an imported move shows this rather than a misleading zero.
pub const UNKNOWN: &str = "—";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    Format(FormatError),
    /// Nothing in the book is the starting position, so nothing in it can be reached.
    NoStartPosition,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::Format(why) => write!(f, "{why}"),
            ImportError::NoStartPosition => write!(f, "book holds no record for the starting position"),
        }
    }
}

impl std::error::Error for ImportError {}

impl From<FormatError> for ImportError {
    fn from(why: FormatError) -> ImportError {
        ImportError::Format(why)
    }
}

/// What the book reports about one move out of one position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub mv: Move,
    /// How often the book says the move was played — the record's weight.
    pub plays: u32,
    /// Wins, draws and losses. `None` when the book did not say, which is the usual case.
    pub score: Option<Score>,
    /// The average rating of the players who played it, when the book knows.
    pub average_rating: Option<u16>,
    pub last_used: Option<Date>,
    /// The interchange format's spare field, kept so that importing and exporting lose nothing.
    pub reserved: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Score {
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
}

impl Offer {
    /// The score column: three numbers, or the unknown marker. Never `0-0-0` for a move nobody
    /// reported results for.
    pub fn score_text(&self) -> String {
        match self.score {
            Some(score) => format!("{}-{}-{}", score.wins, score.draws, score.losses),
            None => UNKNOWN.to_string(),
        }
    }

    pub fn rating_text(&self) -> String {
        self.average_rating.map_or_else(|| UNKNOWN.to_string(), |rating| rating.to_string())
    }
}

/// One position and the moves the book offers from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub key: u64,
    pub moves: Vec<Offer>,
}

/// A whole book, positions sorted by key so a lookup is one binary search.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Book {
    positions: Vec<Position>,
}

impl Book {
    pub fn new(positions: Vec<Position>) -> Book {
        let mut book = Book { positions };
        book.positions.sort_unstable_by_key(|position| position.key);
        book
    }

    pub fn find(&self, key: u64) -> Option<&Position> {
        let at = self.positions.partition_point(|position| position.key < key);
        self.positions.get(at).filter(|position| position.key == key)
    }

    pub fn positions(&self) -> &[Position] {
        &self.positions
    }

    pub fn move_count(&self) -> usize {
        self.positions.iter().map(|position| position.moves.len()).sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    /// The position was reached, but the move the record names is not legal there.
    Illegal(MoveError),
    /// The position is in no game the book can be replayed into, so the record means nothing.
    Unreachable,
}

/// What an import did, so a caller can say "1 234 moves in, 5 records skipped" and be believed.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub positions: usize,
    pub moves: usize,
    /// Why each record that was skipped was skipped, in the order the file held them.
    pub rejected: Vec<Rejection>,
}

/// The format has no code of its own for a promotion to a queen: a pawn reaching the last rank with
/// an empty promotion field means one, and only the position can say so.
fn queened(board: &Board, mv: Move) -> Move {
    let last_rank = board
        .piece_at(mv.from)
        .is_some_and(|p| p.kind == Kind::Pawn && (rank_of(mv.to) == 0 || rank_of(mv.to) == 7));
    if last_rank && mv.promotion.is_none() {
        Move { promotion: Some(Kind::Queen), ..mv }
    } else {
        mv
    }
}

/// Read the interchange format and replay it into the native store.
pub fn import(bytes: &[u8]) -> Result<(Book, Report), ImportError> {
    let entries = book_binary::read(bytes)?;
    // The file is sorted by key; a corrupt or hand-made one might not be, so sort a copy and keep
    // track of where each record came from for the report.
    let mut order: Vec<usize> = (0..entries.len()).collect();
    order.sort_unstable_by_key(|&at| entries[at].key);
    let sorted: Vec<InterchangeEntry> = order.iter().map(|&at| entries[at]).collect();
    let offsets: Vec<usize> = order;

    let start = Board::start();
    if book_binary::find_range(&sorted, start.key()).is_none() {
        return Err(ImportError::NoStartPosition);
    }

    let mut reached: HashMap<u64, Vec<Offer>> = HashMap::new();
    let mut illegal: HashSet<usize> = HashSet::new();
    let mut rejected = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(start);
    reached.insert(start.key(), Vec::new());
    while let Some(board) = queue.pop_front() {
        let Some(range) = book_binary::find_range(&sorted, board.key()) else { continue };
        let mut offers = Vec::new();
        for at in range {
            let entry = sorted[at];
            let mut next = board;
            match next.apply(queened(&board, entry.mv)) {
                Ok(()) => {
                    offers.push(Offer {
                        mv: entry.mv,
                        plays: entry.weight as u32,
                        score: None,
                        average_rating: None,
                        last_used: None,
                        reserved: entry.reserved,
                    });
                    if !reached.contains_key(&next.key()) {
                        reached.insert(next.key(), Vec::new());
                        queue.push_back(next);
                    }
                }
                Err(why) => {
                    illegal.insert(offsets[at]);
                    rejected.push((offsets[at], Rejection::Illegal(why)));
                }
            }
        }
        reached.insert(board.key(), offers);
    }

    let reached_keys: HashSet<u64> = reached.keys().copied().collect();
    for at in 0..sorted.len() {
        if !reached_keys.contains(&sorted[at].key) && !illegal.contains(&offsets[at]) {
            rejected.push((offsets[at], Rejection::Unreachable));
        }
    }
    rejected.sort_unstable_by_key(|(offset, _)| *offset);

    let positions: Vec<Position> = reached
        .into_iter()
        .filter(|(_, offers)| !offers.is_empty())
        .map(|(key, moves)| Position { key, moves })
        .collect();
    let report = Report {
        positions: positions.len(),
        moves: positions.iter().map(|position| position.moves.len()).sum(),
        rejected: rejected.into_iter().map(|(_, why)| why).collect(),
    };
    Ok((Book::new(positions), report))
}

/// Write the native store back out as the interchange format. The weight is the play count, and the
/// spare field goes back as it came in.
pub fn export(book: &Book) -> Vec<u8> {
    let entries: Vec<InterchangeEntry> = book
        .positions
        .iter()
        .flat_map(|position| {
            position.moves.iter().map(move |offer| InterchangeEntry {
                key: position.key,
                mv: offer.mv,
                weight: offer.plays.try_into().unwrap_or(u16::MAX),
                reserved: offer.reserved,
            })
        })
        .collect();
    book_binary::write(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::board::Kind;

    /// A one-move book, built by playing rather than by hand, so its keys are the real thing.
    fn book_of(lines: &[&str]) -> Vec<u8> {
        let mut board = Board::start();
        let mut entries = Vec::new();
        for line in lines {
            let (from, to) = line.split_once("-").unwrap();
            let mv = Move::simple(square(from), square(to));
            entries.push(InterchangeEntry::new(board.key(), mv, 10));
            board.apply(mv).unwrap();
        }
        book_binary::write(&entries)
    }

    fn square(name: &str) -> u8 {
        (name.as_bytes()[1] - b'1') * 8 + (name.as_bytes()[0] - b'a')
    }

    #[test]
    fn a_book_is_imported_by_replaying_it_from_the_start() {
        let bytes = book_of(&["e2-e4", "e7-e5", "g1-f3"]);
        let (book, report) = import(&bytes).unwrap();
        assert_eq!(report.positions, 3, "the start and the two positions after it");
        assert_eq!(report.moves, 3);
        assert!(report.rejected.is_empty());
        let start = book.find(Board::start().key()).expect("the start position");
        assert_eq!(start.moves.len(), 1);
        assert_eq!(start.moves[0].mv, Move::simple(square("e2"), square("e4")));
        assert_eq!(start.moves[0].plays, 10);
    }

    #[test]
    fn a_record_whose_position_is_never_reached_is_skipped_and_counted() {
        // A position from the middle of some other game, which nothing here can reach.
        let mut stranger = Board::start();
        for (from, to) in [("d2", "d4"), ("d7", "d5"), ("c2", "c4"), ("d5", "c4")] {
            stranger.apply(Move::simple(square(from), square(to))).unwrap();
        }
        let mut entries = vec![InterchangeEntry::new(Board::start().key(), Move::simple(square("e2"), square("e4")), 5)];
        entries.push(InterchangeEntry::new(stranger.key(), Move::simple(square("g1"), square("f3")), 7));
        let (book, report) = import(&book_binary::write(&entries)).unwrap();
        assert_eq!(report.moves, 1, "only the reachable move came in");
        assert_eq!(report.rejected, vec![Rejection::Unreachable]);
        assert!(book.find(stranger.key()).is_none());
    }

    #[test]
    fn a_record_naming_an_illegal_move_is_skipped_and_counted() {
        let entries = [
            InterchangeEntry::new(Board::start().key(), Move::simple(square("e2"), square("e4")), 5),
            InterchangeEntry::new(Board::start().key(), Move::simple(square("e2"), square("e5")), 6),
        ];
        let (book, report) = import(&book_binary::write(&entries)).unwrap();
        assert_eq!(report.moves, 1);
        assert_eq!(report.rejected, vec![Rejection::Illegal(MoveError::BadGeometry)]);
        assert_eq!(book.move_count(), 1);
    }

    #[test]
    fn a_book_without_the_starting_position_is_refused() {
        let bytes = book_binary::write(&[InterchangeEntry::new(1234, Move::simple(0, 1), 1)]);
        assert!(matches!(import(&bytes), Err(ImportError::NoStartPosition)));
    }

    #[test]
    fn a_file_cut_mid_record_says_so() {
        let mut bytes = book_of(&["e2-e4"]);
        bytes.truncate(bytes.len() - 1);
        assert!(matches!(import(&bytes), Err(ImportError::Format(FormatError::Trailing { .. }))));
    }

    #[test]
    fn a_round_trip_keeps_every_move_and_its_spare_field() {
        let bytes = book_of(&["e2-e4", "c7-c5", "g1-f3", "d7-d6"]);
        let (book, _) = import(&bytes).unwrap();
        let written = export(&book);
        let (again, _) = import(&written).unwrap();
        assert_eq!(again, book);

        // And the spare field, which the native store has no use for, still survives.
        let mut entries = vec![InterchangeEntry::new(Board::start().key(), Move::simple(square("e2"), square("e4")), 5)];
        entries[0].reserved = 0xdead_beef;
        let (book, _) = import(&book_binary::write(&entries)).unwrap();
        let written = export(&book);
        assert_eq!(book_binary::read(&written).unwrap()[0].reserved, 0xdead_beef);
    }

    #[test]
    fn a_promotion_survives_the_two_byte_move() {
        // A promotion to a queen shares its code with "no promotion", so it is the position that
        // says which it was; the others carry their own code.
        let bytes = book_of(&["a2-a4", "b7-b5", "a4-b5", "a7-a6", "b5-b6", "a6-a5", "b6-a7"]);
        let (book, report) = import(&bytes).unwrap();
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);
        let promoted = book
            .find(Board::start().key())
            .map(|_| ())
            .map(|_| {
                book.positions()
                    .iter()
                    .flat_map(|position| position.moves.iter())
                    .find(|offer| offer.mv.to == square("a7") && offer.mv.from == square("b6"))
            });
        assert_eq!(promoted.and_then(|offer| offer.mv.promotion), Some(Kind::Queen), "the way the format says it");
        let knight = Move { from: square("b6"), to: square("a7"), promotion: Some(Kind::Knight) };
        assert_eq!(book_binary::decode_move(book_binary::encode_move(knight)), knight);
    }

    #[test]
    fn a_move_without_results_shows_the_unknown_marker_not_zero() {
        let bytes = book_of(&["e2-e4"]);
        let (book, _) = import(&bytes).unwrap();
        let offer = &book.find(Board::start().key()).unwrap().moves[0];
        assert_eq!(offer.score, None);
        assert_eq!(offer.score_text(), "—");
        assert_eq!(offer.rating_text(), "—");
        let with_results = Offer {
            mv: offer.mv,
            plays: offer.plays,
            score: Some(Score { wins: 3, draws: 2, losses: 1 }),
            average_rating: Some(2450),
            last_used: None,
            reserved: 0,
        };
        assert_eq!(with_results.score_text(), "3-2-1");
        assert_eq!(with_results.rating_text(), "2450");
    }
}
