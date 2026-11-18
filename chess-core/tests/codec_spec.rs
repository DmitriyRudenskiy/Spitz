//! The spec's notation-codec requirements: SAN with the disambiguation it needs and the suffix it
//! carries, UCI long algebraic, and both round-tripping every legal move.

use chess_core::board::Board;
use chess_core::fen;
use chess_core::move_gen::legal_moves;
use chess_core::san;
use chess_core::uci;

/// The same six positions the move generator is checked against, so the codecs are measured over
/// the shapes the generator already has to get right: castling, en passant, promotions, pins.
/// The depth is how far out from each one the codecs are run: writing SAN means asking whether the
/// move gives check, which means generating the answers to it, and the crowded positions get
/// expensive faster than they get more interesting.
const FIXTURES: [(&str, &str, u8); 6] = [
    (
        "start",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        2,
    ),
    (
        "kiwipete",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        1,
    ),
    (
        "promotions",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        2,
    ),
    ("en passant", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 2),
    (
        "two knights",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        1,
    ),
    (
        "the middlegame",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        1,
    ),
];

#[test]
fn san_disambiguates_by_file_rank_or_both_as_minimally_required() {
    // Three queens can reach e4. The one on h1 is the only queen on the h file, the one on b7 is
    // the only queen on the 7th rank, and the one on b1 shares its file with b7 and its rank with
    // h1, so only the pair identifies it.
    let board = fen::parse("4k3/1Q6/8/8/8/K7/8/1Q5Q w - - 0 1").unwrap();
    let mut written = Vec::new();
    for mv in legal_moves(&board) {
        if mv.to != square("e4") {
            continue;
        }
        written.push(san::format(&board, mv).unwrap());
    }
    written.sort();
    assert_eq!(written, ["Q7e4+", "Qb1e4+", "Qhe4+"]);
}

#[test]
fn san_carries_the_plus_or_hash_suffix() {
    let board = fen::parse("6k1/5ppp/8/8/8/8/8/R6K w - - 0 1").unwrap();
    let rook_to_a8 = legal_moves(&board)
        .into_iter()
        .find(|mv| mv.from == square("a1") && mv.to == square("a8"))
        .unwrap();
    assert_eq!(san::format(&board, rook_to_a8).unwrap(), "Ra8#");

    // The same rook move, with an escape square left open, is a check and not a mate.
    let board = fen::parse("6k1/5pp1/7p/8/8/8/8/R5K1 w - - 0 1").unwrap();
    let rook_to_a8 = legal_moves(&board)
        .into_iter()
        .find(|mv| mv.from == square("a1") && mv.to == square("a8"))
        .unwrap();
    assert_eq!(san::format(&board, rook_to_a8).unwrap(), "Ra8+");

    // And a move that is neither carries nothing.
    let board = fen::parse("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let knight_to_f3 = legal_moves(&board)
        .into_iter()
        .find(|mv| mv.from == square("g1") && mv.to == square("f3"))
        .unwrap();
    assert_eq!(san::format(&board, knight_to_f3).unwrap(), "Nf3");
}

#[test]
fn uci_writes_the_promotion_suffix() {
    let board = fen::parse("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let promote = legal_moves(&board)
        .into_iter()
        .find(|mv| mv.to == square("a8"))
        .unwrap();
    assert_eq!(uci::format(promote), "a7a8q");
    assert_eq!(uci::parse(&board, "a7a8q").unwrap(), promote);
    assert_eq!(san::format(&board, promote).unwrap(), "a8=Q+");
}

#[test]
fn both_codecs_round_trip_every_legal_move() {
    let mut checked = 0;
    for (name, start, depth) in FIXTURES {
        for board in around(start, depth) {
            for mv in legal_moves(&board) {
                let text = san::format(&board, mv)
                    .unwrap_or_else(|fault| panic!("{name}: cannot write {mv:?}: {fault:?}"));
                let back = san::parse(&board, &text)
                    .unwrap_or_else(|fault| panic!("{name}: cannot read back {text}: {fault:?}"));
                assert_eq!(back, mv, "{name}: san {text}");

                let text = uci::format(mv);
                let back = uci::parse(&board, &text)
                    .unwrap_or_else(|fault| panic!("{name}: cannot read uci {text}: {fault:?}"));
                assert_eq!(back, mv, "{name}: uci {text}");
                checked += 1;
            }
        }
    }
    // A round trip that checked nothing would pass just as loudly.
    assert!(checked > 5_000, "only {checked} moves round-tripped");
}

/// The positions within `depth` moves of the start, which is where the interesting shapes are:
/// two pieces of one kind reaching one square, a promotion, a discovered check.
fn around(start: &str, depth: u8) -> Vec<Board> {
    let mut all = vec![fen::parse(start).unwrap()];
    let mut frontier = all.clone();
    for _ in 0..depth {
        let mut next = Vec::new();
        for board in &frontier {
            for mv in legal_moves(board) {
                let mut after = *board;
                after.apply(mv).expect("a legal move is always applicable");
                next.push(after);
            }
        }
        all.extend_from_slice(&next);
        frontier = next;
    }
    all
}

fn square(name: &str) -> chess_core::board::Square {
    let bytes = name.as_bytes();
    (bytes[1] - b'1') * 8 + (bytes[0] - b'a')
}
