//! What a position has already decided, and what a player may claim from it.
//!
//! The two are kept apart on purpose. Checkmate, stalemate and a dead position end the game where
//! they appear; the 50-move rule and a repetition are claims — a player asks for them and the game
//! ends when the ask is granted. A result that mixes the two cannot be handed to an arbiter, a
//! GUI, or a database without someone guessing which kind it was.

use std::collections::HashMap;

use crate::board::{Board, Colour, Kind, file_of, rank_of};
use crate::move_gen::legal_moves;

/// What the position itself says.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    /// The side to move is mated.
    Checkmate { winner: Colour },
    /// The side to move has no legal move and is not in check.
    Stalemate,
    /// No sequence of legal moves leads to mate for either side.
    DeadPosition,
    /// Play can still matter.
    Ongoing,
}

/// A draw a player asks for. The game is not over until they do.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Claim {
    /// A hundred halfmoves without a pawn move or a capture.
    FiftyMoves,
    /// The same position for the third time, with the same side to move and the same rights.
    Threefold,
}

/// Is the side to move in check?
pub fn is_check(board: &Board) -> bool {
    board
        .king_square(board.turn())
        .is_some_and(|king| board.is_attacked(king, board.turn().other()))
}

/// The result the position forces, if it forces one. Claims are [`claim`].
pub fn outcome(board: &Board) -> Outcome {
    if !legal_moves(board).is_empty() {
        return if dead_position(board) { Outcome::DeadPosition } else { Outcome::Ongoing };
    }
    if is_check(board) {
        Outcome::Checkmate { winner: board.turn().other() }
    } else {
        Outcome::Stalemate
    }
}

/// The claim available in this position, given the positions it has been reached among.
pub fn claim(board: &Board, seen: &Repetition) -> Option<Claim> {
    if board.halfmove() >= 100 {
        return Some(Claim::FiftyMoves);
    }
    if seen.threefold(board) {
        return Some(Claim::Threefold);
    }
    None
}

/// The classic reductions: bare kings, one minor against a bare king, and any number of
/// same-colour bishops against a bare king.
///
/// Deliberately narrower than the general question. Two knights can mate with help, so they are
/// not here; a side with any piece of its own might be mated by accident, so a position where both
/// sides have material is never called dead. Claiming a draw that is not one is worse than waiting.
pub fn insufficient_material(board: &Board) -> bool {
    // [colour][pawn, knight, bishop, rook, queen] — kings are nobody's material.
    let mut count = [[0u8; 5]; 2];
    for slot in 0..64 {
        if let Some(piece) = board.piece_at(slot) && piece.kind != Kind::King {
            count[piece.colour as usize][piece.kind as usize] += 1;
        }
    }
    let [black, white] = count;
    let has = |side: [u8; 5]| side[0] + side[1] + side[2] + side[3] + side[4];
    match (has(black) == 0, has(white) == 0) {
        (true, true) => true,
        (true, false) => cannot_mate(white, board),
        (false, true) => cannot_mate(black, board),
        // Both sides can still be mated, however badly.
        _ => false,
    }
}

/// Can this pile mate a bare king? One minor cannot, and any number of same-colour bishops
/// cannot; two knights can, with help, and help is enough for the rules to keep playing.
fn cannot_mate(side: [u8; 5], board: &Board) -> bool {
    let [pawns, knights, bishops, rooks, queens] = side;
    if pawns + rooks + queens > 0 {
        return false;
    }
    match knights + bishops {
        0 => true,
        1 => true,
        count if knights == 0 && count == bishops => bishop_colours(board) < 2,
        _ => false,
    }
}

/// Can neither side be mated by any sequence of legal moves?
///
/// `ponytail:` this is [`insufficient_material`] and nothing more. The general dead-position proof
/// — locked pawn walls, a bishop of the wrong rook pawn, a piece trapped behind them — is a
/// research project, and every position it misses is still a draw by another route. The upgrade
/// path is a reachability search over mating networks, not another clause here.
pub fn dead_position(board: &Board) -> bool {
    insufficient_material(board)
}

/// How many square colours the bishops of this board stand on.
fn bishop_colours(board: &Board) -> u8 {
    let mut colours: u8 = 0;
    for slot in 0..64 {
        if let Some(piece) = board.piece_at(slot)
            && piece.kind == Kind::Bishop
        {
            let colour = (file_of(slot) + rank_of(slot)) % 2;
            colours |= 1 << colour;
        }
    }
    colours.count_ones() as u8
}

/// How often each position of a game has been reached, counted by the position key — which is the
/// placement, the side to move, the castling rights and the capturable en-passant square, and
/// nothing else. Two positions that differ only in a right nobody can use are the same position.
#[derive(Clone, Default, Debug)]
pub struct Repetition {
    counts: HashMap<u64, u8>,
}

impl Repetition {
    /// A position entered. Call this for the start position and for every position reached.
    pub fn note(&mut self, board: &Board) -> u8 {
        *self.counts.entry(board.key()).or_insert(0) += 1;
        self.counts[&board.key()]
    }

    pub fn count(&self, board: &Board) -> u8 {
        self.counts.get(&board.key()).copied().unwrap_or(0)
    }

    pub fn threefold(&self, board: &Board) -> bool {
        self.count(board) >= 3
    }

    /// A pawn move or a capture makes earlier positions unreachable, so counting starts again.
    pub fn clear(&mut self) {
        self.counts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Move;
    use crate::fen::parse;

    fn at(fen: &str) -> Board {
        parse(fen).unwrap()
    }

    #[test]
    fn checkmate_and_stalemate_are_reported_as_finished() {
        let mut board = at("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1");
        // The rook to the eighth rank is mate; play it rather than describe it.
        board.apply(Move::simple(0, 56)).unwrap();
        assert_eq!(outcome(&board), Outcome::Checkmate { winner: Colour::White });
        assert!(is_check(&board));

        let stalemate = at("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");
        assert_eq!(outcome(&stalemate), Outcome::Stalemate);
        assert!(!is_check(&stalemate));
    }

    #[test]
    fn a_mate_is_not_a_draw_claim_even_when_the_clock_allows_one() {
        let mut board = at("6k1/5ppp/8/8/8/8/8/R5K1 w - - 99 120");
        board.apply(Move::simple(0, 56)).unwrap();
        assert_eq!(board.halfmove(), 100);
        assert_eq!(outcome(&board), Outcome::Checkmate { winner: Colour::White });
        assert_eq!(claim(&board, &Repetition::default()), Some(Claim::FiftyMoves));
    }

    #[test]
    fn the_hundredth_halfmove_is_claimable_and_the_ninety_ninth_is_not() {
        let board = at("4k3/8/8/8/8/8/8/4K3 w - - 100 180");
        assert_eq!(claim(&board, &Repetition::default()), Some(Claim::FiftyMoves));
        let board = at("4k3/8/8/8/8/8/8/4K3 w - - 99 180");
        assert_eq!(claim(&board, &Repetition::default()), None);
    }

    #[test]
    fn a_third_repetition_of_the_same_position_is_claimable() {
        let start = at("4k3/8/8/8/8/8/8/N3K3 w - - 0 1");
        let mut board = start;
        let mut seen = Repetition::default();
        seen.note(&board);
        // Both sides shuffle: the knight and the black king step out and back, which returns the
        // position with the same side to move.
        let shuffle = [
            Move::simple(0, 17),
            Move::simple(60, 61),
            Move::simple(17, 0),
            Move::simple(61, 60),
        ];
        for round in 1..=2u8 {
            for mv in shuffle {
                board.apply(mv).unwrap();
                seen.note(&board);
            }
            assert_eq!(seen.count(&start), round + 1, "back where we started");
            assert_eq!(
                claim(&board, &seen),
                if round == 2 { Some(Claim::Threefold) } else { None },
                "the third time is the claimable one"
            );
        }
    }

    #[test]
    fn a_lost_castling_right_is_a_different_position() {
        let with = at("4k3/8/8/8/8/8/8/4K2R w K - 0 1");
        let without = at("4k3/8/8/8/8/8/8/4K2R w - - 0 1");
        let mut seen = Repetition::default();
        seen.note(&with);
        seen.note(&without);
        seen.note(&with);
        assert_eq!(seen.count(&with), 2, "the same army with a lost right is not the same position");
        assert_eq!(seen.count(&without), 1);
        assert_eq!(claim(&with, &seen), None);
    }

    #[test]
    fn an_en_passant_right_is_a_different_position_too() {
        let with = at("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
        let without = at("4k3/8/8/3p4/8/8/8/4K3 w - - 0 1");
        let mut seen = Repetition::default();
        seen.note(&with);
        seen.note(&without);
        seen.note(&with);
        assert_eq!(seen.count(&with), 2);
    }

    #[test]
    fn a_pawn_move_or_a_capture_starts_the_count_again() {
        let board = at("4k3/8/8/8/8/8/8/N3K3 w - - 0 1");
        let mut seen = Repetition::default();
        seen.note(&board);
        seen.note(&board);
        seen.clear();
        assert_eq!(seen.count(&board), 0);
        assert_eq!(claim(&board, &seen), None);
    }

    #[test]
    fn the_classic_reductions_are_dead_and_the_rest_is_not() {
        for (fen, dead) in [
            ("4k3/8/8/8/8/8/8/4K3 w - - 0 1", true),
            ("4k3/8/8/8/8/8/8/3BK3 w - - 0 1", true),
            ("4k3/8/8/8/8/8/8/3NK3 w - - 0 1", true),
            ("4k3/8/8/8/8/8/8/B1B1K3 w - - 0 1", true),
            ("4k3/8/8/8/8/8/8/B2BK3 w - - 0 1", false),
            ("4k3/8/8/8/8/8/8/3NKN2 w - - 0 1", false),
            ("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1", false),
            ("4k3/8/8/8/8/8/8/3BKB2 w - - 0 1", true),
            ("4kb2/8/8/8/8/8/8/3BKB2 w - - 0 1", false),
            ("4k3/8/8/8/8/8/8/3RKB2 w - - 0 1", false),
            ("4k3/8/8/8/8/8/8/3QKB2 w - - 0 1", false),
        ] {
            let board = at(fen);
            assert_eq!(dead_position(&board), dead, "{fen}");
            assert_eq!(outcome(&board), if dead { Outcome::DeadPosition } else { Outcome::Ongoing }, "{fen}");
        }
    }

    #[test]
    fn a_bare_king_facing_two_same_colour_bishops_cannot_be_mated() {
        // a1 and c1 are both dark; d1 is light, so moving one bishop changes the answer.
        assert!(insufficient_material(&at("4k3/8/8/8/8/8/8/B1B1K3 w - - 0 1")));
        assert!(!insufficient_material(&at("4k3/8/8/8/8/8/8/B2BK3 w - - 0 1")));
    }

    #[test]
    fn the_start_position_is_ongoing() {
        let board = Board::start();
        assert_eq!(outcome(&board), Outcome::Ongoing);
        assert_eq!(claim(&board, &Repetition::default()), None);
        assert!(!is_check(&board));
    }
}
