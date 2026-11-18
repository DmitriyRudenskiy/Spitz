//! SAN — the notation a score sheet is written in: `Nbd2`, `R1e2`, `exd8=Q+`, `O-O-O#`.
//!
//! Both directions are here, and both are the boring kind of hard. Writing it means working out
//! the least that tells two pieces apart; reading it means accepting what a human actually types,
//! which includes the disambiguation left out, the capture mark left out, and the promotion written
//! with or without an equals sign.

use crate::board::{
    Board, Colour, Kind, Move, Square, file_of, rank_of, square_from_name, square_name,
};
use crate::move_gen::legal_moves;
use crate::state::{Outcome, is_check, outcome};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SanError {
    /// Not SAN: an empty string, a target square that is not one, a piece letter that is not one.
    Malformed,
    /// The score sheet names two pieces that can both do it, and does not say which.
    Ambiguous,
    /// Well formed, and the position refuses it.
    Illegal,
}

/// The move as a score sheet writes it, with the least disambiguation that separates it from every
/// other move that could land on the same square.
pub fn format(board: &Board, mv: Move) -> Result<String, SanError> {
    let legal = legal_moves(board);
    if !legal.contains(&mv) {
        return Err(SanError::Illegal);
    }
    let mover = board.piece_at(mv.from).ok_or(SanError::Illegal)?;
    let mut text = String::with_capacity(8);

    if mover.kind == Kind::King && (file_of(mv.to) as i8 - file_of(mv.from) as i8).abs() > 1 {
        text.push_str(if file_of(mv.to) > file_of(mv.from) {
            "O-O"
        } else {
            "O-O-O"
        });
    } else if mover.kind == Kind::Pawn {
        if file_of(mv.from) != file_of(mv.to) {
            text.push(file_char(mv.from));
            text.push('x');
        }
        text.push_str(&square_name(mv.to));
        if let Some(kind) = mv.promotion {
            text.push('=');
            text.push(piece_char(kind));
        }
    } else {
        text.push(piece_char(mover.kind));
        text.push_str(&disambiguation(board, &legal, mv, mover.kind));
        if board.piece_at(mv.to).is_some() {
            text.push('x');
        }
        text.push_str(&square_name(mv.to));
    }

    let mut after = *board;
    // The move was legal a moment ago, so this cannot fail; the suffix is the last thing standing.
    if after.apply(mv).is_ok() {
        match outcome(&after) {
            Outcome::Checkmate { .. } => text.push('#'),
            _ if is_check(&after) => text.push('+'),
            _ => {}
        }
    }
    Ok(text)
}

/// Read one move of a score sheet against the position it belongs to.
pub fn parse(board: &Board, text: &str) -> Result<Move, SanError> {
    let text = text.trim().trim_end_matches(['!', '?']);
    let text = text.trim_end_matches(['+', '#']);

    if let Some(short) = castling_side(text) {
        // Only the king goes two files on its own rank, so there is nothing to disambiguate.
        let home = if board.turn() == Colour::White {
            4u8
        } else {
            60
        };
        let to = if short { home + 2 } else { home - 2 };
        return one(board, Kind::King, None, None, to, None);
    }

    let (body, promoted) = match text.split_once('=') {
        Some((body, piece)) => (body, Some(piece_char_of(piece).ok_or(SanError::Malformed)?)),
        None => (text, None),
    };
    if body.is_empty() {
        return Err(SanError::Malformed);
    }
    let kind = match body.as_bytes()[0] {
        b'N' | b'B' | b'R' | b'Q' | b'K' => piece_char_of(&body[..1]).ok_or(SanError::Malformed)?,
        _ => Kind::Pawn,
    };
    // Where a piece came from is written before the square it arrived on, and the capture mark
    // between them carries no information: drop it, and let what is left say the rest.
    let rest: Vec<u8> = body.as_bytes()[kind_letter_len(body)..]
        .iter()
        .copied()
        .filter(|b| *b != b'x')
        .collect();
    if rest.len() < 2 {
        return Err(SanError::Malformed);
    }
    let (origin, target) = rest.split_at(rest.len() - 2);
    let to = square_from_name(std::str::from_utf8(target).map_err(|_| SanError::Malformed)?)
        .ok_or(SanError::Malformed)?;
    let mut origin_file = None;
    let mut origin_rank = None;
    for byte in origin {
        match byte {
            b'a'..=b'h' => origin_file = Some(byte - b'a'),
            b'1'..=b'8' => origin_rank = Some(byte - b'1'),
            _ => return Err(SanError::Malformed),
        }
    }
    // A pawn promotion is the only promotion, and a pawn capture already writes the file it came from.
    let promotion = if kind == Kind::Pawn { promoted } else { None };
    one(board, kind, origin_file, origin_rank, to, promotion)
}

/// The one move that fits, when exactly one does.
fn one(
    board: &Board,
    kind: Kind,
    file: Option<u8>,
    rank: Option<u8>,
    to: Square,
    promotion: Option<Kind>,
) -> Result<Move, SanError> {
    let mut fits = legal_moves(board).into_iter().filter(|mv| {
        let Some(piece) = board.piece_at(mv.from) else {
            return false;
        };
        piece.kind == kind
            && mv.to == to
            && mv.promotion == promotion
            && file.is_none_or(|f| file_of(mv.from) == f)
            && rank.is_none_or(|r| rank_of(mv.from) == r)
    });
    let mv = fits.next().ok_or(SanError::Illegal)?;
    // Two pieces, one square, no word about which: the score sheet is broken, not the reader.
    if fits.next().is_some() {
        return Err(SanError::Ambiguous);
    }
    Ok(mv)
}

/// The least of file, rank, or both that no other piece bound for the same square also matches.
fn disambiguation(board: &Board, legal: &[Move], mv: Move, kind: Kind) -> String {
    let rivals: Vec<Move> = legal
        .iter()
        .copied()
        .filter(|other| {
            other.to == mv.to
                && other.from != mv.from
                && other.promotion == mv.promotion
                && board.piece_at(other.from).is_some_and(|p| p.kind == kind)
        })
        .collect();
    if rivals.is_empty() {
        return String::new();
    }
    let same_file = rivals.iter().any(|o| file_of(o.from) == file_of(mv.from));
    let same_rank = rivals.iter().any(|o| rank_of(o.from) == rank_of(mv.from));
    let mut text = String::with_capacity(2);
    if !same_file {
        text.push(file_char(mv.from));
    } else if !same_rank {
        text.push(rank_char(mv.from));
    } else {
        text.push(file_char(mv.from));
        text.push(rank_char(mv.from));
    }
    text
}

/// The castling forms a score sheet and a scanner agree on.
fn castling_side(text: &str) -> Option<bool> {
    match text
        .to_uppercase()
        .replace('-', "")
        .replace('0', "O")
        .as_str()
    {
        "OO" => Some(true),
        "OOO" => Some(false),
        _ => None,
    }
}

/// Zero for a pawn move, one for a move that starts with a piece letter.
fn kind_letter_len(body: &str) -> usize {
    if body.starts_with(['N', 'B', 'R', 'Q', 'K']) {
        1
    } else {
        0
    }
}

fn file_char(at: Square) -> char {
    char::from(b'a' + file_of(at))
}

fn rank_char(at: Square) -> char {
    char::from(b'1' + rank_of(at))
}

const fn piece_char(kind: Kind) -> char {
    match kind {
        Kind::Knight => 'N',
        Kind::Bishop => 'B',
        Kind::Rook => 'R',
        Kind::Queen => 'Q',
        Kind::King => 'K',
        Kind::Pawn => ' ',
    }
}

fn piece_char_of(letter: &str) -> Option<Kind> {
    match letter.to_ascii_uppercase().as_str() {
        "N" => Some(Kind::Knight),
        "B" => Some(Kind::Bishop),
        "R" => Some(Kind::Rook),
        "Q" => Some(Kind::Queen),
        "K" => Some(Kind::King),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::parse as from_fen;
    use crate::move_gen::perft;

    fn sq(name: &str) -> Square {
        square_from_name(name).unwrap()
    }

    fn written(fen: &str, from: &str, to: &str) -> String {
        let board = from_fen(fen).unwrap();
        format(&board, Move::simple(sq(from), sq(to))).unwrap()
    }

    #[test]
    fn two_pieces_bound_for_one_square_say_which_one_moves() {
        let knights = "3k4/8/8/8/8/8/8/1N3N2 w - - 0 1";
        assert_eq!(written(knights, "b1", "d2"), "Nbd2");
        assert_eq!(written(knights, "f1", "d2"), "Nfd2");
        // Two rooks on one file have no file to write, so they write the rank they left.
        let rooks = "3k4/8/8/8/R7/8/8/R2K4 w - - 0 1";
        assert_eq!(written(rooks, "a1", "a2"), "R1a2");
        assert_eq!(written(rooks, "a4", "a2"), "R4a2");
        // A square only one knight reaches needs nothing said about which.
        assert_eq!(written(knights, "f1", "h2"), "Nh2");
    }

    /// Three queens that can each reach c3, and a black king out of everyone's firing line.
    const CORNER_QUEENS: &str = "8/8/8/7k/8/Q7/8/Q1Q4K w - - 0 1";

    #[test]
    fn when_neither_file_nor_rank_separates_them_both_are_written() {
        // Three queens can all reach c3: from a1, from a3, and from c1.
        let board = from_fen(CORNER_QUEENS).unwrap();
        let from_a1 = Move::simple(sq("a1"), sq("c3"));
        let text = format(&board, from_a1).unwrap();
        assert_eq!(
            text, "Qa1c3",
            "the file alone still leaves a3, the rank alone still leaves c1"
        );
        assert_eq!(parse(&board, &text).unwrap(), from_a1);
        assert_eq!(
            written(CORNER_QUEENS, "a3", "c3"),
            "Q3c3",
            "no other queen is on the third rank"
        );
    }

    #[test]
    fn a_pawn_capture_writes_the_file_and_a_promotion_names_the_piece() {
        let board = from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap();
        assert_eq!(
            format(&board, Move::simple(sq("e5"), sq("d6"))).unwrap(),
            "exd6"
        );
        assert_eq!(
            parse(&board, "exd6").unwrap(),
            Move::simple(sq("e5"), sq("d6"))
        );

        let promoting = from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mv = Move {
            from: sq("a7"),
            to: sq("a8"),
            promotion: Some(Kind::Knight),
        };
        assert_eq!(format(&promoting, mv).unwrap(), "a8=N");
        assert_eq!(parse(&promoting, "a8=N").unwrap(), mv);
    }

    #[test]
    fn check_and_mate_get_their_suffixes() {
        let checking = from_fen("4k3/8/8/8/8/8/8/R3K3 w - - 0 1").unwrap();
        assert_eq!(
            format(&checking, Move::simple(sq("a1"), sq("a8"))).unwrap(),
            "Ra8+"
        );
        let mated = from_fen("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1").unwrap();
        assert_eq!(
            format(&mated, Move::simple(sq("a1"), sq("a8"))).unwrap(),
            "Ra8#"
        );
        // And the suffix is not part of what has to be understood to find the move.
        assert_eq!(
            parse(&mated, "Ra8#").unwrap(),
            Move::simple(sq("a1"), sq("a8"))
        );
        assert_eq!(
            parse(&mated, "Ra8").unwrap(),
            Move::simple(sq("a1"), sq("a8"))
        );
    }

    #[test]
    fn castling_is_two_castles_or_three() {
        let board = from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        assert_eq!(
            format(&board, Move::simple(sq("e1"), sq("g1"))).unwrap(),
            "O-O"
        );
        assert_eq!(
            format(&board, Move::simple(sq("e1"), sq("c1"))).unwrap(),
            "O-O-O"
        );
        for (written, mv) in [
            ("O-O", Move::simple(sq("e1"), sq("g1"))),
            ("0-0", Move::simple(sq("e1"), sq("g1"))),
            ("oo", Move::simple(sq("e1"), sq("g1"))),
            ("O-O-O", Move::simple(sq("e1"), sq("c1"))),
        ] {
            assert_eq!(parse(&board, written).unwrap(), mv, "{written}");
        }
    }

    #[test]
    fn a_score_sheet_omits_what_cannot_be_misread() {
        let board = Board::start();
        for (text, from, to) in [
            ("e4", "e2", "e4"),
            ("Nf3", "g1", "f3"),
            ("Nc3", "b1", "c3"),
            ("a4", "a2", "a4"),
        ] {
            let mv = Move::simple(sq(from), sq(to));
            assert_eq!(parse(&board, text).unwrap(), mv, "{text}");
            assert_eq!(format(&board, mv).unwrap(), text);
        }
    }

    #[test]
    fn a_move_nobody_can_play_is_refused() {
        let board = Board::start();
        assert_eq!(
            parse(&board, "e5"),
            Err(SanError::Illegal),
            "two squares is not three"
        );
        assert_eq!(parse(&board, "Ke4"), Err(SanError::Illegal));
        assert_eq!(
            parse(&board, "exd5"),
            Err(SanError::Illegal),
            "nothing stands there"
        );
        assert_eq!(parse(&board, "Qxg7"), Err(SanError::Illegal));
        assert_eq!(parse(&board, ""), Err(SanError::Malformed));
        assert_eq!(
            parse(&board, "Ono"),
            Err(SanError::Malformed),
            "no piece is an O"
        );
        assert_eq!(
            parse(&board, "e4=Q"),
            Err(SanError::Illegal),
            "a pawn on e4 does not promote"
        );
    }

    #[test]
    fn a_move_that_could_be_two_pieces_says_so() {
        let board = from_fen("3k4/8/8/8/R7/8/8/R2K4 w - - 0 1").unwrap();
        assert_eq!(parse(&board, "Ra2"), Err(SanError::Ambiguous));
        assert_eq!(
            parse(&board, "R1a2").unwrap(),
            Move::simple(sq("a1"), sq("a2"))
        );
    }

    #[test]
    fn every_legal_move_of_the_fixtures_round_trips() {
        // The spec's own test: encode every legal move of the perft positions and read it back.
        for fen in [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
            "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1",
        ] {
            let board = from_fen(fen).unwrap();
            let mut checked = 0;
            for mv in legal_moves(&board) {
                let text = format(&board, mv).unwrap();
                assert_eq!(parse(&board, &text).unwrap(), mv, "{fen}: {text}");
                checked += 1;
            }
            assert_eq!(checked as u64, perft(&board, 1), "{fen}");
        }
    }

    #[test]
    fn round_trips_go_deeper_than_the_first_move() {
        // Walking a game is how the awkward positions get found: the ones where three pieces
        // reach one square, and where a capture and a check land on the same move.
        let mut board =
            from_fen("r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5Q2/PPPP1PPP/RNB1K1NR w KQkq - 0 1").unwrap();
        let mut seen = 0;
        for _ in 0..60 {
            let legal = legal_moves(&board);
            let Some(&first) = legal.first() else { break };
            for &mv in &legal {
                let text = format(&board, mv).unwrap();
                assert_eq!(
                    parse(&board, &text).unwrap(),
                    mv,
                    "{text} on {fen}",
                    fen = crate::fen::format(&board)
                );
                seen += 1;
            }
            board.apply(first).unwrap();
        }
        assert!(seen > 500, "the walk only looked at {seen} moves");
    }
}
