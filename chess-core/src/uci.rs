//! UCI long algebraic — the notation an engine speaks: `e2e4`, `e7e8q`, `e1g1`.
//!
//! One square after another, with a promotion piece tacked on the end. It is unambiguous without
//! knowing anything about the position, which is why it is what goes over a pipe; the price is that
//! it does not read like chess, so it is not what a human ever writes down.

use crate::board::{Board, Kind, Move, square_from_name, square_name};
use crate::move_gen::legal_moves;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UciError {
    /// Not four characters, or four that are not two squares.
    Malformed,
    /// A fifth character that is not a piece a pawn can become.
    BadPromotion,
    /// Well formed, and refused by the position.
    Illegal,
}

/// The move as a UCI engine writes it. A castling move is the king's two squares, which is what a
/// [`Move`] already carries.
pub fn format(mv: Move) -> String {
    let mut text = square_name(mv.from);
    text.push_str(&square_name(mv.to));
    if let Some(kind) = mv.promotion {
        text.push(letter(kind));
    }
    text
}

/// Read a UCI move against the position it is played in, because only the position can say whether
/// `e7e8q` is a promotion, a nonsense move, or the one move that mates.
pub fn parse(board: &Board, text: &str) -> Result<Move, UciError> {
    let text = text.trim();
    let (squares, promoted) = match text.len() {
        4 => (text, None),
        5 => (&text[..4], Some(&text[4..5])),
        _ => return Err(UciError::Malformed),
    };
    let (from, to) = squares.split_at(2);
    let from = square_from_name(from).ok_or(UciError::Malformed)?;
    let to = square_from_name(to).ok_or(UciError::Malformed)?;
    let promotion = match promoted {
        None => None,
        Some(letter) => Some(kind_of(letter).ok_or(UciError::BadPromotion)?),
    };
    let mv = Move {
        from,
        to,
        promotion,
    };
    if legal_moves(board).contains(&mv) {
        Ok(mv)
    } else {
        Err(UciError::Illegal)
    }
}

/// The letter a promotion piece is written with. Lowercase, as engines write it.
pub const fn letter(kind: Kind) -> char {
    match kind {
        Kind::Knight => 'n',
        Kind::Bishop => 'b',
        Kind::Rook => 'r',
        Kind::Queen => 'q',
        _ => ' ',
    }
}

/// The piece a letter names, refusing a pawn and a king, neither of which a pawn becomes.
pub fn kind_of(letter: &str) -> Option<Kind> {
    match letter.to_ascii_lowercase().as_str() {
        "n" => Some(Kind::Knight),
        "b" => Some(Kind::Bishop),
        "r" => Some(Kind::Rook),
        "q" => Some(Kind::Queen),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Square;
    use crate::fen::parse as from_fen;

    fn sq(name: &str) -> Square {
        square_from_name(name).unwrap()
    }

    #[test]
    fn a_move_is_two_squares_and_nothing_else() {
        assert_eq!(format(Move::simple(sq("e2"), sq("e4"))), "e2e4");
        assert_eq!(
            format(Move::simple(sq("e1"), sq("g1"))),
            "e1g1",
            "castling is the king's hop"
        );
    }

    #[test]
    fn a_promotion_names_the_piece_it_becomes() {
        let board = from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        for (kind, letter) in [
            (Kind::Queen, "a7a8q"),
            (Kind::Rook, "a7a8r"),
            (Kind::Bishop, "a7a8b"),
            (Kind::Knight, "a7a8n"),
        ] {
            let mv = Move {
                from: sq("a7"),
                to: sq("a8"),
                promotion: Some(kind),
            };
            assert_eq!(format(mv), letter);
            assert_eq!(parse(&board, letter).unwrap(), mv);
        }
    }

    #[test]
    fn a_move_reads_back_against_the_position() {
        let board =
            from_fen("r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5Q2/PPPP1PPP/RNB1K1NR w KQkq - 0 1").unwrap();
        let mv = parse(&board, "f3f7").unwrap();
        assert_eq!(format(mv), "f3f7");
    }

    #[test]
    fn nonsense_is_refused_for_what_is_wrong_with_it() {
        let board = Board::start();
        assert_eq!(parse(&board, "e2"), Err(UciError::Malformed));
        assert_eq!(parse(&board, "e2e4x"), Err(UciError::BadPromotion));
        assert_eq!(parse(&board, "i2i4"), Err(UciError::Malformed));
        assert_eq!(
            parse(&board, "e2e5"),
            Err(UciError::Illegal),
            "the pawn does not travel three"
        );
        assert_eq!(
            parse(&board, "e7e5"),
            Err(UciError::Illegal),
            "and it is not black's turn"
        );
    }

    #[test]
    fn every_legal_move_of_the_start_position_round_trips() {
        let board = Board::start();
        for mv in legal_moves(&board) {
            let text = format(mv);
            assert_eq!(parse(&board, &text).unwrap(), mv, "{text}");
        }
    }
}
