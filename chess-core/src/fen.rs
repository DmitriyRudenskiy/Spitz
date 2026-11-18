//! FEN — the one-line form a position is exchanged in, and the form a game document uses for a
//! position that is not the start position.
//!
//! The board keeps the en-passant square only when a capture is actually possible, which is what
//! the interchange key counts (see `docs/position-key-format.md`). A FEN naming a square nobody can
//! capture on is accepted and normalised to `-`, so parsing then formatting is stable from the
//! second pass on rather than the first.

use crate::board::{
    Board, CASTLE_BLACK_KING, CASTLE_BLACK_QUEEN, CASTLE_WHITE_KING, CASTLE_WHITE_QUEEN, Colour,
    Kind, Piece, Square, file_of, rank_of,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FenError {
    /// Fewer than the four fields a position needs.
    TooFewFields,
    /// More than the six fields a position has.
    TooManyFields,
    /// The piece placement is not eight ranks, or says more than eight files, or holds a character
    /// that is not a piece or a run of empty squares.
    Placement,
    /// Active colour is neither `w` nor `b`.
    ActiveColour,
    /// A castling field that is not `-` or a subset of `KQkq` without repeats.
    Castling,
    /// An en-passant field that is not `-` or a square on the rank the side to move would capture on.
    EnPassant,
    /// A clock field that is not a number, or larger than the rules can reach.
    Clock,
}

impl std::fmt::Display for FenError {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FenError::TooFewFields => write!(out, "a FEN needs placement, colour, castling and en passant"),
            FenError::TooManyFields => write!(out, "a FEN has six fields at most"),
            FenError::Placement => write!(out, "piece placement is eight ranks of at most eight files"),
            FenError::ActiveColour => write!(out, "active colour is `w` or `b`"),
            FenError::Castling => write!(out, "castling is `-` or a subset of `KQkq`"),
            FenError::EnPassant => write!(out, "en passant is `-` or a square on the capturable rank"),
            FenError::Clock => write!(out, "clock fields are numbers within reach of the rules"),
        }
    }
}

impl std::error::Error for FenError {}

/// Read a position. The halfmove and fullmove fields are kept but do not take part in the
/// position's key.
pub fn parse(text: &str) -> Result<Board, FenError> {
    let mut fields = text.split_whitespace();
    let mut next = |what| fields.next().ok_or(what);
    let placement = next(FenError::TooFewFields)?;
    let active = next(FenError::TooFewFields)?;
    let castling = next(FenError::TooFewFields)?;
    let ep = next(FenError::TooFewFields)?;
    let halfmove = match fields.next() {
        None => 0,
        Some(field) => clock(field)?,
    };
    let fullmove = match fields.next() {
        None => 1,
        Some(field) => moves(field)?,
    };
    if fields.next().is_some() {
        return Err(FenError::TooManyFields);
    }

    let squares = placement_of(placement)?;
    let turn = match active {
        "w" => Colour::White,
        "b" => Colour::Black,
        _ => return Err(FenError::ActiveColour),
    };
    let rights = rights_of(castling)?;
    Ok(Board::from_parts(squares, turn, rights, ep_of(ep, turn)?, halfmove, fullmove))
}

/// Write a position: the six fields, in the order a FEN has them.
pub fn format(board: &Board) -> String {
    let mut text = String::with_capacity(84);
    for rank in (0..8).rev() {
        let mut empty = 0;
        for file in 0..8 {
            match board.piece_at(rank * 8 + file) {
                None => empty += 1,
                Some(piece) => {
                    if empty > 0 {
                        text.push(char::from(b'0' + empty));
                        empty = 0;
                    }
                    text.push(char_of(piece));
                }
            }
        }
        if empty > 0 {
            text.push(char::from(b'0' + empty));
        }
        if rank > 0 {
            text.push('/');
        }
    }
    text.push_str(if board.turn() == Colour::White { " w " } else { " b " });
    let mut rights = String::new();
    for (right, letter) in [
        (CASTLE_WHITE_KING, 'K'),
        (CASTLE_WHITE_QUEEN, 'Q'),
        (CASTLE_BLACK_KING, 'k'),
        (CASTLE_BLACK_QUEEN, 'q'),
    ] {
        if board.castling() & right != 0 {
            rights.push(letter);
        }
    }
    text.push_str(if rights.is_empty() { "-" } else { &rights });
    match board.ep_field() {
        None => text.push_str(" -"),
        Some(at) => text.push_str(&format!(" {}", square_name(at))),
    }
    text.push_str(&format!(" {} {}", board.halfmove(), board.fullmove()));
    text
}

fn placement_of(placement: &str) -> Result<[Option<Piece>; 64], FenError> {
    let mut squares = [None; 64];
    let ranks: Vec<&str> = placement.split('/').collect();
    if ranks.len() != 8 {
        return Err(FenError::Placement);
    }
    for (from_top, row) in ranks.iter().enumerate() {
        let rank = 7 - from_top as u8;
        let mut file = 0;
        for letter in row.chars() {
            if let '1'..='8' = letter {
                file += letter as u8 - b'0';
            } else {
                let colour = if letter.is_uppercase() { Colour::White } else { Colour::Black };
                let kind = kind_of(letter.to_ascii_lowercase()).ok_or(FenError::Placement)?;
                squares[usize::from(rank * 8 + file)] = Some(Piece { colour, kind });
                file += 1;
            }
            if file > 8 {
                return Err(FenError::Placement);
            }
        }
        if file != 8 {
            return Err(FenError::Placement);
        }
    }
    Ok(squares)
}

fn rights_of(castling: &str) -> Result<u8, FenError> {
    if castling == "-" {
        return Ok(0);
    }
    let mut rights = 0;
    for letter in castling.chars() {
        let right = match letter {
            'K' => CASTLE_WHITE_KING,
            'Q' => CASTLE_WHITE_QUEEN,
            'k' => CASTLE_BLACK_KING,
            'q' => CASTLE_BLACK_QUEEN,
            _ => return Err(FenError::Castling),
        };
        if rights & right != 0 {
            return Err(FenError::Castling);
        }
        rights |= right;
    }
    Ok(rights)
}

/// The square the field names. A FEN writes the square a double push jumped, whether or not a pawn
/// stands where it could take, and reading it back has to agree with the string it came from.
/// Whether it is a target at all is the board's question: [`Board::en_passant`].
fn ep_of(field: &str, turn: Colour) -> Result<Option<Square>, FenError> {
    if field == "-" {
        return Ok(None);
    }
    if field.len() != 2 {
        return Err(FenError::EnPassant);
    }
    let bytes = field.as_bytes();
    if !bytes[0].is_ascii_lowercase() || !bytes[1].is_ascii_digit() {
        return Err(FenError::EnPassant);
    }
    let (file, rank) = (bytes[0] - b'a', bytes[1] - b'1');
    // White captures on rank 6, black on rank 3, and only the side to move can capture.
    let expected = if turn == Colour::White { 5 } else { 2 };
    if file > 7 || rank != expected {
        return Err(FenError::EnPassant);
    }
    Ok(Some(rank * 8 + file))
}

fn clock(field: &str) -> Result<u16, FenError> {
    let value: u16 = field.parse().map_err(|_| FenError::Clock)?;
    // The rule resets at 100 halfmoves; anything past that never reaches the board.
    if value > 100 {
        return Err(FenError::Clock);
    }
    Ok(value)
}

fn moves(field: &str) -> Result<u16, FenError> {
    let value: u16 = field.parse().map_err(|_| FenError::Clock)?;
    // Numbering starts at 1, so a game that has not started yet is not a position.
    if value == 0 {
        return Err(FenError::Clock);
    }
    Ok(value)
}

fn kind_of(letter: char) -> Option<Kind> {
    match letter {
        'p' => Some(Kind::Pawn),
        'n' => Some(Kind::Knight),
        'b' => Some(Kind::Bishop),
        'r' => Some(Kind::Rook),
        'q' => Some(Kind::Queen),
        'k' => Some(Kind::King),
        _ => None,
    }
}

fn char_of(piece: Piece) -> char {
    let letter = match piece.kind {
        Kind::Pawn => 'p',
        Kind::Knight => 'n',
        Kind::Bishop => 'b',
        Kind::Rook => 'r',
        Kind::Queen => 'q',
        Kind::King => 'k',
    };
    if piece.colour == Colour::White { letter.to_ascii_uppercase() } else { letter }
}

fn square_name(at: Square) -> String {
    let mut name = String::with_capacity(2);
    name.push(char::from(b'a' + file_of(at)));
    name.push(char::from(b'1' + rank_of(at)));
    name
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Move;

    fn sq(name: &str) -> Square {
        let bytes = name.as_bytes();
        (bytes[1] - b'1') * 8 + (bytes[0] - b'a')
    }

    const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    #[test]
    fn the_start_position_writes_the_fen_everyone_expects() {
        assert_eq!(format(&Board::start()), START);
    }

    #[test]
    fn a_fen_reads_the_position_the_start_position_is() {
        assert_eq!(parse(START).unwrap().key(), Board::start().key());
    }

    #[test]
    fn every_field_survives_a_round_trip() {
        for text in [
            START,
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 12 44",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 b - - 0 1",
            "4k3/8/8/8/8/8/8/4K3 b - - 99 65000",
        ] {
            let board = parse(text).unwrap();
            assert_eq!(format(&board), text, "{text}");
        }
    }

    #[test]
    fn the_clocks_are_kept_and_the_move_number_follows_black() {
        let board = parse("4k3/8/8/8/8/8/8/4K3 w - - 7 18").unwrap();
        assert_eq!((board.halfmove(), board.fullmove()), (7, 18));
        let mut after = board;
        after.apply(Move::simple(4, 12)).unwrap();
        assert_eq!((after.halfmove(), after.fullmove()), (8, 18));
        after.apply(Move::simple(60, 52)).unwrap();
        assert_eq!((after.halfmove(), after.fullmove()), (9, 19));
    }

    #[test]
    fn a_pawn_move_or_a_capture_resets_the_halfmove_clock() {
        let mut board = parse("4k3/8/8/8/8/8/4P3/4K3 w - - 40 90").unwrap();
        board.apply(Move::simple(12, 20)).unwrap();
        assert_eq!(board.halfmove(), 0);
        board.apply(Move::simple(60, 52)).unwrap();
        assert_eq!(board.halfmove(), 1);
    }

    #[test]
    fn an_en_passant_square_nobody_can_capture_on_is_written_but_not_believed() {
        let written = "4k3/8/8/3p4/8/8/8/4K3 w - d6 0 1";
        let board = parse(written).unwrap();
        assert_eq!(board.ep_field(), Some(sq("d6")), "the string survives");
        assert_eq!(board.en_passant(), None, "no pawn stands where it could take");
        assert_eq!(format(&board), written, "byte for byte");
        // A pawn on e5 makes it a position, not just a note about the last move.
        let with = parse("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap();
        assert_eq!(with.en_passant(), Some(sq("d6")));
        assert_ne!(with.key(), board.key(), "a capturable square is part of the position");
        let uncapturable = parse("4k3/8/8/3p4/8/8/8/4K3 w - - 0 1").unwrap();
        assert_eq!(uncapturable.key(), board.key(), "one nobody can take is not part of it");
    }

    #[test]
    fn a_castling_right_without_a_rook_still_reads() {
        // Whether the rook survived is `apply`'s business; the position says what it says.
        let board = parse("4k3/8/8/8/8/8/8/4K3 w KQkq - 0 1").unwrap();
        assert_eq!(board.castling(), 0b1111);
    }

    #[test]
    fn malformed_positions_are_refused_by_what_is_wrong_with_them() {
        use FenError::*;
        for (text, expected) in [
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP", TooFewFields),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w", TooFewFields),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 extra", TooManyFields),
            ("rnbqkbnr/ppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", Placement),
            ("rnbqkbnr/ppppppppp/8/8/8/8/PPPPPPPP w KQkq - 0 1", Placement),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBXR w KQkq - 0 1", Placement),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1", ActiveColour),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KKkq - 0 1", Castling),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e5 0 1", EnPassant),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq i6 0 1", EnPassant),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - x 1", Clock),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 101 1", Clock),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0", Clock),
        ] {
            assert_eq!(parse(text).unwrap_err(), expected, "{text}");
        }
    }

    #[test]
    fn a_missing_clock_reads_the_default_a_short_fen_means() {
        let board = parse("4k3/8/8/8/8/8/8/4K3 w - -").unwrap();
        assert_eq!((board.halfmove(), board.fullmove()), (0, 1));
    }
}
