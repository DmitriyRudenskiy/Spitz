//! The game document (task 7.1): a tree an editor can change, and a history that does not run
//! out. The scenarios are the ones the game-management spec states.

use chess_core::pgn::document::{self, DIAGRAM};
use chess_core::pgn::{GameResult, GameResult::Unfinished, Glyph};
use chess_core::{fen, move_gen, pgn};

/// Stand at the end of the main line, where an editor typing the next move would be.
fn end_of_main_line(doc: &mut document::Document) {
    doc.goto(document::ROOT);
    doc.to_end();
}

/// Play one move at the cursor, chosen by index from the sorted legal moves so the walk below is
/// reproducible. A move already written there is walked to, which records nothing, and the model
/// under the test follows that.
fn play_here(doc: &mut document::Document, which: usize) {
    let board = doc.position(doc.cursor()).unwrap();
    let mut moves = move_gen::legal_moves(&board);
    moves.sort_by_key(|mv| format!("{mv:?}"));
    if let Some(mv) = moves.get(which % moves.len()).copied() {
        let _ = doc.play_move(mv);
    }
}

fn document() -> document::Document {
    document::Document::new()
}

#[test]
fn every_edit_undoes_and_redoes_exactly_however_many_steps() {
    // The spec's scenario: variations, comments, NAGs, glyphs and a diagram, then many undo and
    // redo steps in one session, every one of them restoring the exact prior document.
    let mut doc = document();
    // The states the document has been in, and which one the history stands on.
    let mut states = vec![doc.to_game()];
    let mut at = 0usize;

    // Each edit says where it stands first, because the walk below moves the cursor and an
    // editor's next move is written where the cursor is.
    let edits = [
        |doc: &mut document::Document| {
            doc.goto(document::ROOT);
            play_here(doc, 0);
        },
        |doc: &mut document::Document| {
            let at = doc.cursor();
            doc.add_comment(at, "an aggressive start");
        },
        |doc: &mut document::Document| {
            let at = doc.cursor();
            doc.set_glyphs(at, vec![Glyph::Interesting]);
        },
        |doc: &mut document::Document| {
            let at = doc.cursor();
            doc.set_nags(at, vec![14, 20]);
        },
        |doc: &mut document::Document| {
            let at = doc.cursor();
            doc.set_diagram(at, true);
        },
        |doc: &mut document::Document| {
            end_of_main_line(doc);
            play_here(doc, 1);
        },
        |doc: &mut document::Document| {
            end_of_main_line(doc);
            play_here(doc, 2);
        },
        |doc: &mut document::Document| {
            // A different move in the same position is another alternative, not a new line.
            end_of_main_line(doc);
            doc.up();
            play_here(doc, 1);
        },
        |doc: &mut document::Document| {
            doc.set_result(GameResult::Draw);
        },
        |doc: &mut document::Document| {
            doc.set_tag("White", "Spassky");
        },
        |doc: &mut document::Document| {
            doc.add_tail_comment("and they agreed to stop here");
        },
        |doc: &mut document::Document| {
            if let Some(last) = doc.main_line().last().copied() {
                doc.remove(last).unwrap();
            }
        },
    ];

    let mut seed = 0x9e37_79b9_7f4a_7c15u64;
    let mut below = |limit: u64| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) % limit
    };

    for step in 0..400 {
        // The first pass lays the edits down; after that the walk is theirs to choose.
        match if step < edits.len() { 0 } else { below(4) } {
            0 => {
                let before = doc.undo_depth();
                edits[step % edits.len()](&mut doc);
                // An edit that changed nothing records nothing, and the model follows that.
                if doc.undo_depth() > before {
                    states.truncate(at + 1);
                    states.push(doc.to_game());
                    at = states.len() - 1;
                }
            }
            1 | 2 if doc.can_undo() => {
                assert!(doc.undo());
                at -= 1;
                assert_eq!(
                    states[at],
                    doc.to_game(),
                    "step {step}: undo did not restore state {at}"
                );
            }
            _ if doc.can_redo() => {
                assert!(doc.redo());
                at += 1;
                assert_eq!(
                    states[at],
                    doc.to_game(),
                    "step {step}: redo did not restore state {at}"
                );
            }
            // A walk that runs into the end of the history is still a walk.
            _ => {}
        }
    }

    assert!(at > 1, "the walk barely moved: {at}");
    // And the whole history really is walkable, in both directions, from wherever it ended.
    while doc.can_undo() {
        assert!(doc.undo());
        at -= 1;
        assert_eq!(states[at], doc.to_game(), "undoing to the start");
    }
    assert_eq!(at, 0);
    assert_eq!(states[0], document().to_game());
    while doc.can_redo() {
        assert!(doc.redo());
        at += 1;
        assert_eq!(states[at], doc.to_game(), "redoing to the end");
    }
    assert_eq!(at, states.len() - 1);
}

#[test]
fn an_embedded_diagram_survives_a_save_and_a_reopen() {
    let mut doc = document();
    doc.play("e4").unwrap();
    let at = doc.cursor();
    doc.play("e6").unwrap();
    doc.set_diagram(at, true);

    let saved = doc.format();
    assert!(saved.contains(DIAGRAM), "{saved}");
    let reopened = document::Document::parse(&saved).expect("the saved game reads back");
    let line = reopened.main_line();
    assert_eq!(line.len(), 2);
    assert!(
        reopened.diagram(line[0]),
        "the diagram renders again at the same move"
    );
    assert!(!reopened.diagram(line[1]));
    assert_eq!(
        fen::format(&reopened.position(line[0]).unwrap()),
        fen::format(&doc.position(at).unwrap()),
        "with the same position"
    );
    // And it is not left behind as a stray comment nobody asked for.
    assert_eq!(reopened.get(line[0]).comments, Vec::<String>::new());
}

#[test]
fn a_diagram_is_a_comment_to_everything_else() {
    let mut doc = document();
    doc.play("e4").unwrap();
    let at = doc.cursor();
    doc.set_diagram(at, true);

    let game = doc.to_game();
    assert_eq!(game.moves[0].comments, vec![DIAGRAM.to_string()]);
    // Read by anything else it is an annotation on the move, which is what it is.
    let plain = pgn::parse(&doc.format()).unwrap();
    assert_eq!(plain.moves[0].comments, vec![DIAGRAM.to_string()]);
}

#[test]
fn a_variation_is_an_alternative_to_the_move_it_follows() {
    let mut doc = document();
    doc.play("e4").unwrap();
    let e4 = doc.cursor();
    let c5 = doc.play("c5").unwrap();
    // Back to the position after 1.e4, and a different second move.
    assert!(doc.goto(e4));
    let d6 = doc.play("d6").unwrap();

    assert_eq!(doc.get(e4).children, vec![c5, d6]);
    assert_eq!(
        doc.parent(d6),
        Some(e4),
        "an alternative hangs where the move it replaces hangs"
    );
    assert_eq!(doc.depth(d6), doc.depth(c5));
    assert_eq!(
        fen::format(&doc.position(d6).unwrap())
            .split_whitespace()
            .next()
            .unwrap(),
        "rnbqkbnr/ppp1pppp/3p4/8/4P3/8/PPPP1PPP/RNBQKBNR"
    );
    let text = doc.format();
    assert!(text.contains("(1... d6)"), "{text}");
    assert_eq!(
        doc.main_line(),
        vec![e4, c5],
        "the first line stays the main line"
    );
}

#[test]
fn a_move_the_position_refuses_changes_nothing() {
    let mut doc = document();
    doc.play("e4").unwrap();
    let before = doc.to_game();
    let depth = doc.undo_depth();
    let cursor = doc.cursor();

    let error = doc.play("Ke1").unwrap_err();
    assert!(matches!(
        error,
        document::Error::Unplayable { text, .. } if text == "Ke1"
    ));
    assert_eq!(before, doc.to_game());
    assert_eq!(
        depth,
        doc.undo_depth(),
        "a refused edit is not in the history"
    );
    assert_eq!(doc.cursor(), cursor, "and the cursor did not move either");
    doc.play("e5").unwrap();
    assert_eq!(doc.main_line().len(), 2);
}

#[test]
fn a_move_already_written_there_is_not_written_twice() {
    let mut doc = document();
    let first = doc.play("e4").unwrap();
    // Standing in front of it again — the cursor moved back, or the file was reopened — the move
    // is walked to, not written a second time.
    assert!(doc.goto(document::ROOT));
    let again = doc.play("e4").unwrap();
    assert_eq!(first, again);
    assert_eq!(doc.alternatives(document::ROOT).len(), 1);
    assert_eq!(doc.undo_depth(), 1, "walking to a move is not an edit");
    assert_eq!(doc.cursor(), again);
}

#[test]
fn the_document_and_the_score_sheet_agree_both_ways() {
    let source = r#"[Event "Rated game"]
[Site "Berlin"]
[Result "1-0"]

{a study, more or less} 1. e4 c5 2. Nf3 {developing} (2. Nc3 Nc6 3. Nf3 {transposing} 3...Nf6
4. d4 cxd4) 2...d6 3. d4 cxd4 4. Nxd4 Nf6 $14 {the main line} 1-0"#;
    let game = pgn::parse(source).expect("the score sheet reads");
    let doc = document::Document::from_game(&game);
    assert_eq!(
        doc.to_game(),
        game,
        "the tree and the score sheet hold the same game"
    );
    assert_eq!(pgn::format(&doc.to_game()), pgn::format(&game));

    // And the other way round: a document edited into shape writes a score sheet that reads back
    // as the same document.
    let mut built = document::Document::new();
    built.play("e4").unwrap();
    let e4 = built.cursor();
    built.play("e5").unwrap();
    assert!(built.goto(e4));
    built.play("c5").unwrap();
    assert!(built.goto(e4));
    built.set_comments(e4, vec!["bold".into()]);
    built.set_diagram(e4, true);
    built.set_result(GameResult::WhiteWins);
    let saved = built.format();
    let read_back = document::Document::parse(&saved).expect("what we wrote reads back");
    assert_eq!(built.to_game(), read_back.to_game(), "{saved}");
}

#[test]
fn a_removed_variation_comes_back_whole() {
    let mut doc = document();
    doc.play("e4").unwrap();
    let e4 = doc.cursor();
    doc.play("c5").unwrap();
    doc.play("Nf3").unwrap();
    let branch = doc.get(e4).children[0];
    doc.remove(branch).unwrap();
    assert_eq!(doc.get(e4).children, Vec::new());
    assert_eq!(
        doc.main_line(),
        vec![e4],
        "the line the score sheet writes is the line that is left"
    );

    assert!(doc.undo());
    assert_eq!(doc.get(e4).children, vec![branch]);
    assert_eq!(doc.get(branch).san.as_deref(), Some("c5"));
    assert_eq!(
        doc.get(branch).children.len(),
        1,
        "and what hung under it came back with it"
    );
    assert_eq!(
        doc.get(doc.get(branch).children[0]).san.as_deref(),
        Some("Nf3")
    );
}

#[test]
fn undoing_the_move_under_the_cursor_leaves_it_somewhere_that_exists() {
    let mut doc = document();
    doc.play("e4").unwrap();
    doc.play("c5").unwrap();
    let c5 = doc.cursor();
    doc.remove(c5).unwrap();
    assert_eq!(doc.cursor(), doc.parent(c5).unwrap());
    assert!(doc.undo());
    assert!(doc.goto(doc.cursor()), "the cursor was parked, not lost");
}

#[test]
fn undoing_past_the_first_edit_is_not_an_error() {
    let mut doc = document();
    assert!(!doc.can_undo());
    assert!(!doc.undo());
    assert!(!doc.can_redo());
    assert!(!doc.redo());
    doc.play("e4").unwrap();
    assert!(doc.undo());
    assert!(!doc.undo());
    assert!(doc.can_redo());
    assert_eq!(doc.redo_depth(), 1);
}

#[test]
fn the_position_at_a_node_is_the_position_its_moves_reach() {
    let mut doc = document();
    doc.play("e4").unwrap();
    let e4 = doc.cursor();
    let e5 = doc.play("e5").unwrap();
    doc.play("Nf3").unwrap();
    let nf3 = doc.cursor();

    assert_eq!(doc.depth(e4), 1);
    assert_eq!(doc.depth(nf3), 3);
    assert_eq!(doc.line(nf3), vec![document::ROOT, e4, e5, nf3]);
    assert_eq!(
        fen::format(&doc.position(nf3).unwrap())
            .split_whitespace()
            .next()
            .unwrap(),
        "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R"
    );

    // A variation has its own position, not the one the main line reached.
    assert!(doc.goto(e4));
    let c5 = doc.play("c5").unwrap();
    assert_ne!(
        fen::format(&doc.position(c5).unwrap()),
        fen::format(&doc.position(nf3).unwrap())
    );
    assert_eq!(
        fen::format(&doc.position(c5).unwrap())
            .split_whitespace()
            .next()
            .unwrap(),
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR"
    );
    assert_eq!(doc.line(c5), vec![document::ROOT, e4, c5]);
    assert_eq!(doc.main_line(), doc.line(nf3)[1..]);
}

#[test]
fn a_game_from_a_fen_tag_starts_where_the_tag_says() {
    let source = r#"[SetUp "1"]
[FEN "8/8/4k3/8/8/2B5/1P6/4K3 w - - 0 1"]

1. Ke2 Kd5 2. Kd3 *"#;
    let doc = document::Document::parse(source).unwrap();
    let line = doc.main_line();
    assert_eq!(
        fen::format(&doc.position(line[1]).unwrap())
            .split_whitespace()
            .next()
            .unwrap(),
        "8/8/8/3k4/8/2B5/1P2K3/8"
    );
    // The tag survives an edit and a save, or the game would reopen from the wrong position.
    let saved = doc.format();
    assert!(saved.contains("[FEN"), "{saved}");
    let again = document::Document::parse(&saved).unwrap();
    assert_eq!(again.to_game(), doc.to_game());
}

#[test]
fn a_diagram_before_the_first_move_marks_the_starting_position() {
    let mut doc = document();
    doc.set_diagram(document::ROOT, true);
    doc.add_comment(document::ROOT, "the position before anything happens");
    let saved = doc.format();
    assert!(
        saved.contains("{the position before anything happens} {diagram}"),
        "{saved}"
    );
    let reopened = document::Document::parse(&saved).unwrap();
    assert!(reopened.diagram(document::ROOT));
    assert_eq!(
        reopened.get(document::ROOT).comments,
        vec!["the position before anything happens"]
    );
    assert_eq!(reopened.to_game(), doc.to_game());
}

#[test]
fn an_edit_after_an_undo_forgets_what_was_undone() {
    let mut doc = document();
    doc.play("e4").unwrap();
    doc.play("c5").unwrap();
    doc.undo();
    assert!(doc.can_redo());
    doc.play("e5").unwrap();
    assert!(
        !doc.can_redo(),
        "a new edit takes the future with it, as every editor does"
    );
    assert_eq!(doc.main_line().len(), 2);
}

#[test]
fn the_head_and_the_tail_are_edited_like_anything_else() {
    let mut doc = document();
    doc.play("e4").unwrap();
    doc.set_result(GameResult::Draw);
    doc.add_tail_comment("a quiet draw");
    doc.add_comment(document::ROOT, "played blindfold");

    let saved = doc.format();
    assert!(
        saved.contains("{played blindfold} 1. e4 1/2-1/2 {a quiet draw}"),
        "{saved}"
    );
    let reopened = document::Document::parse(&saved).unwrap();
    assert_eq!(reopened.to_game(), doc.to_game());

    assert!(doc.undo());
    assert_eq!(doc.get(document::ROOT).comments, Vec::<String>::new());
    assert!(doc.undo());
    assert_eq!(doc.to_game().tail, Vec::<String>::new());
    assert!(doc.undo());
    assert_eq!(doc.to_game().result, Unfinished);
    assert!(doc.undo());
    assert_eq!(doc.main_line(), Vec::new());
    assert!(!doc.undo());
    assert_eq!(doc.to_game(), document().to_game());
}

#[test]
fn the_cursor_walks_the_tree_without_editing_anything() {
    let mut doc = document();
    let e4 = doc.play("e4").unwrap();
    let c5 = doc.play("c5").unwrap();
    assert!(doc.goto(e4));
    let e5 = doc.play("e5").unwrap();

    assert!(doc.goto(c5));
    assert!(doc.next_sibling());
    assert_eq!(doc.cursor(), e5);
    assert!(doc.prev_sibling());
    assert_eq!(doc.cursor(), c5);
    assert!(
        !doc.prev_sibling(),
        "there is nothing written before the first"
    );
    assert!(doc.up());
    assert_eq!(doc.cursor(), e4);
    assert!(doc.down());
    assert_eq!(doc.cursor(), c5, "down takes the first alternative");
    doc.to_end();
    assert_eq!(doc.cursor(), c5);
    assert!(!doc.down(), "and this line ends here");

    let depth = doc.undo_depth();
    assert!(doc.goto(document::ROOT));
    doc.to_end();
    assert_eq!(doc.cursor(), c5);
    assert_eq!(doc.undo_depth(), depth, "walking is not editing");
}
