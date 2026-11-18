//! Every move the side to move has. The board can already refuse an illegal move; this is the
//! other direction — asking what is playable without naming a move first.
//!
//! Generation is deliberately naive: pseudo-legal moves come from the piece's geometry, and each
//! one is handed to [`Board::apply`], which is the single place legality lives. Pins, en-passant
//! that exposes the king, and castling across the board's checks are therefore decided once, here
//! and in the interchange path, not twice.

use crate::board::{
    Board, CASTLE_BLACK_KING, CASTLE_BLACK_QUEEN, CASTLE_WHITE_KING, CASTLE_WHITE_QUEEN, Colour,
    Kind, Move, Piece, Square, file_of, rank_of,
};

/// The most pseudo-legal moves a position can hold: every pawn promoting in every direction, plus
/// the pieces that can still move. Generous, and asserted on rather than trusted.
pub const MAX_PSEUDO: usize = 256;

const KNIGHT_STEPS: [(i8, i8); 8] =
    [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)];
const DIAGONALS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const STRAIGHTS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const KING_STEPS: [(i8, i8); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];
const PROMOTIONS: [Kind; 4] = [Kind::Queen, Kind::Rook, Kind::Bishop, Kind::Knight];

/// Every legal move, in square order, promotions expanded into four.
pub fn legal_moves(board: &Board) -> Vec<Move> {
    let mut buffer = [Move::simple(0, 0); MAX_PSEUDO];
    let count = pseudo_moves(board, &mut buffer);
    let mut legal = Vec::with_capacity(count);
    for &mv in &buffer[..count] {
        let mut next = *board;
        if next.apply(mv).is_ok() {
            legal.push(mv);
        }
    }
    legal
}

/// Is this move playable? Same answer [`Board::apply`] gives, without playing it.
pub fn is_legal(board: &Board, mv: Move) -> bool {
    let mut next = *board;
    next.apply(mv).is_ok()
}

/// Write every pseudo-legal move into `out` and say how many. `out` must hold [`MAX_PSEUDO`];
/// some of what comes back leaves the king in check, so filter before playing.
pub fn pseudo_moves(board: &Board, out: &mut [Move]) -> usize {
    let mut at = 0;
    each_pseudo(board, &mut |mv| {
        debug_assert!(at < out.len(), "a position produced more than {MAX_PSEUDO} moves");
        if at < out.len() {
            out[at] = mv;
            at += 1;
        }
    });
    at
}

fn each_pseudo(board: &Board, push: &mut impl FnMut(Move)) {
    let mover = board.turn();
    let ep = board.en_passant();
    for from in 0..64 {
        let Some(piece) = board.piece_at(from) else { continue };
        if piece.colour != mover {
            continue;
        }
        match piece.kind {
            Kind::Pawn => pawn_moves(board, mover, from, ep, push),
            Kind::Knight => step_moves(board, mover, from, &KNIGHT_STEPS, push),
            Kind::King => {
                step_moves(board, mover, from, &KING_STEPS, push);
                castles(board, mover, push);
            }
            Kind::Bishop => slide_moves(board, mover, from, &DIAGONALS, push),
            Kind::Rook => slide_moves(board, mover, from, &STRAIGHTS, push),
            Kind::Queen => {
                slide_moves(board, mover, from, &DIAGONALS, push);
                slide_moves(board, mover, from, &STRAIGHTS, push);
            }
        }
    }
}

fn pawn_moves(
    board: &Board,
    mover: Colour,
    from: Square,
    ep: Option<Square>,
    push: &mut impl FnMut(Move),
) {
    let forward: i8 = if mover == Colour::White { 1 } else { -1 };
    let home = if mover == Colour::White { 1 } else { 6 };
    let promoting = if mover == Colour::White { 7 } else { 0 };
    let (file, rank) = (file_of(from), rank_of(from));
    if let Some(ahead) = square_at(file, rank as i8 + forward)
        && board.piece_at(ahead).is_none()
    {
        push_pawn(from, ahead, promoting, push);
        if rank == home
            && let Some(two) = square_at(file, rank as i8 + 2 * forward)
            && board.piece_at(two).is_none()
        {
            push(Move::simple(from, two));
        }
    }
    for step in [-1i8, 1] {
        let diagonal = file as i8 + step;
        if !(0..8).contains(&diagonal) {
            continue;
        }
        let Some(to) = square_at(diagonal as u8, rank as i8 + forward) else { continue };
        let takes = board.piece_at(to).is_some_and(|piece| piece.colour != mover);
        if takes || ep == Some(to) {
            push_pawn(from, to, promoting, push);
        }
    }
}

/// A pawn reaching the last rank is four moves, not one.
fn push_pawn(from: Square, to: Square, promoting: u8, push: &mut impl FnMut(Move)) {
    if rank_of(to) == promoting {
        for kind in PROMOTIONS {
            push(Move { from, to, promotion: Some(kind) });
        }
    } else {
        push(Move::simple(from, to));
    }
}

fn step_moves(board: &Board, mover: Colour, from: Square, steps: &[(i8, i8)], push: &mut impl FnMut(Move)) {
    for &(dx, dy) in steps {
        let (file, rank) = (file_of(from) as i8 + dx, rank_of(from) as i8 + dy);
        if !(0..8).contains(&file) || !(0..8).contains(&rank) {
            continue;
        }
        let to = (rank * 8 + file) as Square;
        if board.piece_at(to).is_none_or(|piece| piece.colour != mover) {
            push(Move::simple(from, to));
        }
    }
}

fn slide_moves(board: &Board, mover: Colour, from: Square, rays: &[(i8, i8)], push: &mut impl FnMut(Move)) {
    for &(dx, dy) in rays {
        let (mut file, mut rank) = (file_of(from) as i8 + dx, rank_of(from) as i8 + dy);
        while (0..8).contains(&file) && (0..8).contains(&rank) {
            let to = (rank * 8 + file) as Square;
            match board.piece_at(to) {
                None => push(Move::simple(from, to)),
                Some(piece) if piece.colour != mover => push(Move::simple(from, to)),
                Some(_) => break,
            }
            file += dx;
            rank += dy;
        }
    }
}

/// The two castles, when the right survives. Whether the way is clear and the king safe is left
/// to `apply`, which is where that is decided.
fn castles(board: &Board, mover: Colour, push: &mut impl FnMut(Move)) {
    let (from, king_side, queen_side, king_right, queen_right) = match mover {
        Colour::White => (4u8, 6u8, 2u8, CASTLE_WHITE_KING, CASTLE_WHITE_QUEEN),
        Colour::Black => (60, 62, 58, CASTLE_BLACK_KING, CASTLE_BLACK_QUEEN),
    };
    if board.piece_at(from) != Some(Piece { colour: mover, kind: Kind::King }) {
        return;
    }
    if board.castling() & king_right != 0 {
        push(Move::simple(from, king_side));
    }
    if board.castling() & queen_right != 0 {
        push(Move::simple(from, queen_side));
    }
}

/// The square at these coordinates, when they are on the board.
const fn square_at(file: u8, rank: i8) -> Option<Square> {
    if file > 7 || rank < 0 || rank > 7 {
        None
    } else {
        Some(rank as Square * 8 + file)
    }
}
/// Positions at the bottom of this tree, counted the way perft counts them: `perft(d)` is the
/// number of distinct positions reachable in exactly `d` legal moves.
pub fn perft(board: &Board, depth: u8) -> u64 {
    let mut buffer = [Move::simple(0, 0); MAX_PSEUDO];
    let count = pseudo_moves(board, &mut buffer);
    let mut leaves = 0;
    for &mv in &buffer[..count] {
        let mut next = *board;
        if next.apply(mv).is_ok() {
            leaves += if depth <= 1 { 1 } else { perft(&next, depth - 1) };
        }
    }
    leaves
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{CASTLE_BLACK_KING, CASTLE_WHITE_KING, CASTLE_WHITE_QUEEN};
    use crate::fen::parse;

    /// Squares by name, so a fixture says `e1` rather than 4.
    fn sq(name: &str) -> Square {
        let file = name.as_bytes()[0] - b'a';
        let rank = name.as_bytes()[1] - b'1';
        rank * 8 + file
    }

    /// The published perft fixtures: the counts are the point of the exercise, not a check that
    /// the generator agrees with itself.
    const FIXTURES: [(&str, &str, &[u64]); 6] = [
        (
            "start",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            &[20, 400, 8902, 197281, 4865609, 119060324],
        ),
        (
            "kiwipete",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            &[48, 2039, 97862, 4085603, 193690690],
        ),
        (
            "promotions",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            &[6, 264, 9467, 422333, 15833292],
        ),
        (
            "en passant",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            &[14, 191, 2812, 43238, 674624, 11030083],
        ),
        (
            "two knights",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            &[44, 1486, 62379, 2103487, 89941194],
        ),
        (
            "the middlegame",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            &[46, 2079, 89890, 3894594, 164075551],
        ),
    ];

    #[test]
    fn the_start_position_has_twenty_moves_and_they_are_the_moves_it_has() {
        let board = parse("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let moves = legal_moves(&board);
        assert_eq!(moves.len(), 20);
        let froms: Vec<u8> = moves.iter().map(|mv| mv.from).collect();
        assert_eq!(froms.iter().filter(|&&at| at == sq("b1")).count(), 2);
        assert_eq!(froms.iter().filter(|&&at| at == sq("a2")).count(), 2);
        assert!(moves.iter().all(|mv| mv.from != sq("e1")));
    }

    #[test]
    fn perft_matches_the_fixtures() {
        for (name, fen, counts) in FIXTURES {
            let board = parse(fen).unwrap();
            for (depth, &expected) in counts.iter().take(3).enumerate() {
                assert_eq!(perft(&board, (depth + 1) as u8), expected, "{name} depth {}", depth + 1);
            }
        }
    }

    /// Depth four and five are a few million positions, which is a wait in a debug build and a
    /// rounding error in a release one — the same split the timing budgets make.
    #[test]
    #[cfg(not(debug_assertions))]
    fn perft_matches_the_fixtures_deeper() {
        for (name, fen, counts) in FIXTURES {
            let board = parse(fen).unwrap();
            for (depth, &expected) in counts.iter().enumerate().skip(3) {
                assert_eq!(perft(&board, (depth + 1) as u8), expected, "{name} depth {}", depth + 1);
            }
        }
    }

    #[test]
    #[ignore = "the throughput budget is measured on the reference machine (ADR-0010)"]
    fn perft_reaches_the_reference_throughput() {
        let board = parse(FIXTURES[0].1).unwrap();
        let started = std::time::Instant::now();
        let nodes = perft(&board, 6);
        let elapsed = started.elapsed();
        assert_eq!(nodes, 119_060_324);
        let per_second = nodes as f64 / elapsed.as_secs_f64();
        println!(
            "perft depth 6: {nodes} positions in {elapsed:?} — {:.1} M/s (budget 30 M/s)",
            per_second / 1e6
        );
        assert!(per_second >= 30e6, "budget is 30M nodes/s, this reached {per_second:.0}");
    }

    #[test]
    fn a_pinned_rook_stays_on_the_file_that_pins_it() {
        let board = parse("4r3/8/8/8/8/8/4R3/4K2k w - - 0 1").unwrap();
        let moves = legal_moves(&board);
        let e2 = sq("e2");
        assert!(moves.iter().any(|mv| mv.from == e2 && mv.to == sq("e8")), "the capture along the file is legal");
        assert!(
            moves.iter().all(|mv| mv.from != e2 || file_of(mv.to) == 4),
            "a rook that leaves the file uncovers its king: {moves:?}"
        );
    }

    #[test]
    fn a_castling_right_survives_only_when_the_king_is_not_in_check() {
        let board = parse("4r2k/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
        assert_eq!(board.castling(), CASTLE_WHITE_KING | CASTLE_WHITE_QUEEN);
        let moves = legal_moves(&board);
        assert!(moves.iter().all(|mv| mv.from != sq("e1") || (mv.to != sq("g1") && mv.to != sq("c1"))));
        assert!(moves.contains(&Move::simple(sq("e1"), sq("d1"))), "the king has to answer it");
    }

    #[test]
    fn castling_is_generated_when_it_is_playable() {
        let board = parse("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let moves = legal_moves(&board);
        assert!(moves.contains(&Move::simple(sq("e1"), sq("g1"))));
        assert!(moves.contains(&Move::simple(sq("e1"), sq("c1"))));
    }

    #[test]
    fn an_en_passant_that_opens_the_file_it_stands_on_is_not_a_move() {
        // The capturing pawn leaves c5, and c5 is what keeps the black rook off the white king.
        let board = parse("2r5/8/8/1pP5/8/8/8/2K5 w - b6 0 1").unwrap();
        assert_eq!(board.en_passant(), Some(sq("b6")));
        let moves = legal_moves(&board);
        assert!(
            !moves.contains(&Move::simple(sq("c5"), sq("b6"))),
            "the capture would leave the king attacked: {moves:?}"
        );
    }

    #[test]
    fn an_en_passant_that_is_a_real_capture_is_a_move() {
        let board = parse("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap();
        assert!(legal_moves(&board).contains(&Move::simple(sq("e5"), sq("d6"))));
    }

    #[test]
    fn a_pawn_on_the_last_rank_is_four_moves() {
        let board = parse("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let moves = legal_moves(&board);
        let promotions: Vec<Move> = moves
            .iter()
            .filter(|mv| mv.from == sq("a7") && mv.promotion.is_some())
            .copied()
            .collect();
        assert_eq!(promotions.len(), 4);
        assert!(moves.iter().any(|mv| mv == &Move { from: sq("a7"), to: sq("a8"), promotion: Some(Kind::Knight) }));
    }

    #[test]
    fn a_knight_jumps_and_its_own_king_may_stand_where_it_attacks() {
        let board = parse("4k3/8/8/8/8/8/8/N2K4 w - - 0 1").unwrap();
        let moves = legal_moves(&board);
        assert!(moves.contains(&Move::simple(sq("a1"), sq("b3"))));
        assert!(moves.contains(&Move::simple(sq("a1"), sq("c2"))));
        assert!(moves.contains(&Move::simple(sq("d1"), sq("c2"))), "own pieces threaten nobody");
    }

    #[test]
    fn two_kings_never_end_up_next_to_each_other() {
        let board = parse("8/8/8/8/8/4K3/8/4k3 w - - 0 1").unwrap();
        let moves = legal_moves(&board);
        for square in ["d2", "e2", "f2", "d1", "f1"] {
            assert!(!moves.contains(&Move::simple(sq("e3"), sq(square))), "{} is next to the black king", square);
        }
        assert!(moves.contains(&Move::simple(sq("e3"), sq("e4"))), "and the rest of the board is open");
    }

    #[test]
    fn black_to_move_generates_for_black() {
        let board = parse("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
        assert_eq!(legal_moves(&board).len(), 20);
        let board = parse("4k3/8/8/8/8/8/P7/4K3 b - - 0 1").unwrap();
        assert!(legal_moves(&board).contains(&Move::simple(sq("e8"), sq("d7"))));
    }

    #[test]
    fn the_black_castles_are_generated_too() {
        let board = parse("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1").unwrap();
        let moves = legal_moves(&board);
        assert!(moves.contains(&Move::simple(sq("e8"), sq("g8"))));
        assert!(moves.contains(&Move::simple(sq("e8"), sq("c8"))));
        assert_eq!(board.castling(), CASTLE_WHITE_KING | CASTLE_WHITE_QUEEN | CASTLE_BLACK_KING | 1 << 3);
    }

    #[test]
    fn every_generated_move_is_playable_and_the_count_agrees_with_the_board() {
        for (_, fen, _) in FIXTURES {
            let board = parse(fen).unwrap();
            let mut buffer = [Move::simple(0, 0); MAX_PSEUDO];
            let pseudo = pseudo_moves(&board, &mut buffer);
            let legal = legal_moves(&board);
            assert!(pseudo >= legal.len(), "{fen}: {pseudo} pseudo, {} legal", legal.len());
            for mv in legal {
                let mut next = board;
                assert!(next.apply(mv).is_ok(), "{fen}: {mv:?} was generated but refused");
            }
        }
    }
}
