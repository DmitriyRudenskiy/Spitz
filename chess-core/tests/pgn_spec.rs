//! The spec's PGN section: read a game, keep what was written around the moves, report what is
//! not a game, and write it back.

use chess_core::pgn::{self, GameResult, Glyph};

const NAJDORF: &str = "[Event \"Test\"]\n[Site \"Home\"]\n[Result \"*\"]\n\n\
                       1. e4 c5 2. Nf3 {developing} d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 *\n";

#[test]
fn round_trips_a_game_with_tags_and_moves() {
    let game = pgn::parse(NAJDORF).unwrap();
    assert_eq!(game.tag("Event"), Some("Test"));
    assert_eq!(game.tag("Site"), Some("Home"));
    assert_eq!(game.moves.len(), 10);
    let again = pgn::parse(&pgn::format(&game)).unwrap();
    assert_eq!(again.moves, game.moves);
    assert_eq!(again.tags, game.tags);

    // And the moves are the moves: the whole game replays to the position it ends in.
    let mut board = game.start().unwrap();
    for move_ in &game.moves {
        let mv = chess_core::san::parse(&board, &move_.san).unwrap();
        board.apply(mv).unwrap();
    }
    assert_eq!(
        chess_core::fen::format(&board),
        "rnbqkb1r/1p2pppp/p2p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 6"
    );
}

#[test]
fn preserves_comments_and_nags() {
    let game = pgn::parse(
        "1. e4 {best} (1. d4 {a different game} 1... Nf6 2. c4) 1... c5 $5 2. Nf3!? d6 *",
    )
    .unwrap();
    assert_eq!(game.moves[0].comments, vec!["best"]);
    assert_eq!(game.moves[0].variations.len(), 1);
    assert_eq!(
        game.moves[0].variations[0][0].comments,
        vec!["a different game"]
    );
    assert_eq!(game.moves[1].nags, vec![5]);
    assert_eq!(game.moves[2].glyphs, vec![Glyph::Interesting]);

    // The annotation survives being written out and read back.
    let text = pgn::format(&game);
    let again = pgn::parse(&text).unwrap();
    assert_eq!(again.moves, game.moves);
    assert!(text.contains("{best}"));
    assert!(text.contains("$5"));
    assert!(text.contains("Nf3!?"));
}

#[test]
fn reads_a_variation_as_a_branch_off_the_move_it_follows() {
    let game =
        pgn::parse("1. e4 {best} (1. d4 {also good} 1... Nf6 2. c4) 1... c5 2. Nf3 *").unwrap();
    // The main line stays flat, and the alternative hangs off the move it answers.
    let main: Vec<&str> = game.moves.iter().map(|m| m.san.as_str()).collect();
    assert_eq!(main, ["e4", "c5", "Nf3"]);
    let branch = &game.moves[0].variations[0];
    let branch: Vec<&str> = branch.iter().map(|m| m.san.as_str()).collect();
    assert_eq!(branch, ["d4", "Nf6", "c4"]);

    // A variation of white's first move starts from the position before it, so it replays too.
    let mut board = game.start().unwrap();
    for san in branch {
        let mv = chess_core::san::parse(&board, san).unwrap();
        board.apply(mv).unwrap();
    }
    // The en passant field follows the spec's rule — a target whenever a double move just
    // happened — which is why it says c3 here and why python-chess, which only marks a target a
    // legal capture exists for, leaves it empty.
    assert_eq!(
        chess_core::fen::format(&board),
        "rnbqkb1r/pppppppp/5n2/8/2PP4/8/PP2PPPP/RNBQKBNR b KQkq c3 0 2"
    );
}

#[test]
fn reports_an_illegal_token_with_its_line_number() {
    let source = "[Result \"*\"]\n\n1. e4 e5\n2. &h5 *\n";
    let fault = pgn::parse(source).unwrap_err();
    assert_eq!(fault.to_string(), "line 4: UnexpectedChar('&')");
}

#[test]
fn reports_an_unbalanced_variation_with_its_line_number() {
    let source = "[Result \"*\"]\n\n1. e4 (1. d4 e5\n2. c4 *\n";
    let fault = pgn::parse(source).unwrap_err();
    assert_eq!(fault.to_string(), "line 3: UnbalancedVariation");
}

#[test]
fn refuses_a_move_that_is_not_legal_in_the_position_and_says_where() {
    let fault = pgn::parse("[Result \"*\"]\n\n1. e4 e5\n2. Rh3 *\n").unwrap_err();
    assert_eq!(
        fault.to_string(),
        "line 4: move 3 is not playable in the position: Rh3"
    );
    // A word that is not a move at all is the same kind of fault, at the same place.
    let fault = pgn::parse("[Result \"*\"]\n\n1. e4 e5\n2. nfc3 *\n").unwrap_err();
    assert_eq!(
        fault.to_string(),
        "line 4: move 3 is not playable in the position: nfc3"
    );
}

#[test]
fn keeps_games_separate_in_a_concatenated_stream() {
    let source = format!("[Event \"One\"]\n[Result \"1-0\"]\n\n1. e4 1-0\n{NAJDORF}");
    let (games, fault) = pgn::parse_stream(&source).unwrap();
    assert_eq!(fault, None);
    assert_eq!(games.len(), 2);
    assert_eq!(games[0].tag("Event"), Some("One"));
    assert_eq!(games[0].result, GameResult::WhiteWins);
    assert_eq!(games[0].moves.len(), 1);
    assert_eq!(games[1].tag("Event"), Some("Test"));
    assert_eq!(games[1].moves.len(), 10);
}

#[test]
fn keeps_the_order_of_the_tags() {
    let game =
        pgn::parse("[Black \"B\"]\n[White \"A\"]\n[Event \"E\"]\n[Result \"*\"]\n\n1. e4 *\n")
            .unwrap();
    let names: Vec<&str> = game.tags.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["Black", "White", "Event", "Result"]);
    assert!(pgn::format(&game).starts_with("[Black \"B\"]\n[White \"A\"]\n[Event \"E\"]\n"));
}

#[test]
fn escapes_and_unescapes_a_quote_in_a_tag_value() {
    let game =
        pgn::parse("[Event \"He said \\\"hi\\\" to me\"]\n[Result \"*\"]\n\n1. e4 *\n").unwrap();
    assert_eq!(game.tag("Event"), Some("He said \"hi\" to me"));
    let text = pgn::format(&game);
    assert!(
        text.contains("[Event \"He said \\\"hi\\\" to me\"]"),
        "{text}"
    );
    assert_eq!(
        pgn::parse(&text).unwrap().tag("Event"),
        Some("He said \"hi\" to me")
    );
}

#[test]
fn reads_a_fen_tagged_start_position() {
    let game = pgn::parse(
        "[SetUp \"1\"]\n[FEN \"4k3/8/8/8/8/8/8/R3K3 w - - 0 1\"]\n[Result \"*\"]\n\n1. Ra2 Ke7 *\n",
    )
    .unwrap();
    let board = game.start().unwrap();
    assert_eq!(
        chess_core::fen::format(&board),
        "4k3/8/8/8/8/8/8/R3K3 w - - 0 1"
    );
    let main: Vec<&str> = game.moves.iter().map(|m| m.san.as_str()).collect();
    assert_eq!(main, ["Ra2", "Ke7"]);
}

#[test]
fn reads_a_game_with_no_result_marker() {
    let game = pgn::parse("[Result \"*\"]\n\n1. e4 e5").unwrap();
    assert_eq!(game.result, GameResult::Unfinished);
    assert_eq!(game.moves.len(), 2);
}

#[test]
fn reads_a_game_with_no_move_text() {
    let game = pgn::parse("[Event \"Forfeited\"]\n[Result \"1-0\"]\n").unwrap();
    assert!(game.moves.is_empty());
    assert_eq!(game.result, GameResult::WhiteWins);
    assert_eq!(
        pgn::format(&game),
        "[Event \"Forfeited\"]\n[Result \"1-0\"]\n\n1-0\n"
    );
}

#[test]
fn a_comment_after_the_result_belongs_to_the_game() {
    // The marker ends the moves, not the game: what is written after it annotates the game, and
    // dropping it would lose a note on the way in and on the way out.
    let game = pgn::parse("{a blindfold game} 1. e4 e5 1/2-1/2 {and they stopped there}").unwrap();
    assert_eq!(game.head, ["a blindfold game"]);
    assert_eq!(game.tail, ["and they stopped there"]);
    assert_eq!(game.moves.len(), 2);
    assert_eq!(game.result, GameResult::Draw);

    let text = pgn::format(&game);
    let again = pgn::parse(&text).unwrap();
    assert_eq!(again.tail, game.tail, "{text}");
    assert_eq!(again.head, game.head);
}

#[test]
fn a_game_ends_where_the_next_one_starts() {
    let source =
        "[Event \"First\"]\n\n1. e4 e5 1/2-1/2 {a draw}\n\n[Event \"Second\"]\n\n1. d4 Nf6 1-0\n";
    let (games, fault) = pgn::parse_stream(source).unwrap();
    assert_eq!(fault, None);
    assert_eq!(games.len(), 2);
    assert_eq!(games[0].tag("Event"), Some("First"));
    assert_eq!(games[0].tail, ["a draw"]);
    assert_eq!(games[1].tag("Event"), Some("Second"));
    assert_eq!(games[1].result, GameResult::WhiteWins);
    assert!(games[1].tail.is_empty());
}
