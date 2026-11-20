//! The game as something you edit: a node per move, a cursor, and a history with no depth.
//!
//! [`super::game`] reads and writes a score sheet. A score sheet is text with parentheses; an
//! editor wants the other way round — one node per move, an identity for each that survives an
//! insert next to it, and an edit that can be taken back. This module holds the same tree the
//! score sheet describes as an arena of nodes, so a node is a [`NodeId`] and not a path that
//! goes stale when a variation is added before it.
//!
//! Undo is a log of inverses rather than snapshots: an edit records what it touched, so taking
//! it back costs the size of the edit and the history is bounded by memory and nothing else.
//!
//! A diagram is a flag on a node — "draw the board here" — because the position belongs to the
//! node and cannot disagree with it. It travels through the score sheet as a comment whose whole
//! text is [`DIAGRAM`], which is our marker and not a PGN standard: another program reads it as
//! a comment and keeps it, and reading it back with this module turns it into a diagram again.

use std::fmt;

use crate::board::{Board, Move};
use crate::fen;
use crate::san::{self, SanError};

use super::game::{self, Fault, Game, MoveText};
use super::lexer::{GameResult, Glyph};

/// A node of the tree: the root, and one node per move written anywhere in it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct NodeId(u32);

/// The node every line starts under: no move, the head comments, and the alternatives the game
/// opens with.
pub const ROOT: NodeId = NodeId(0);

/// The comment that means "a diagram is embedded here". Written as a brace comment by
/// [`Document::to_game`] and read back by [`Document::from_game`].
pub const DIAGRAM: &str = "diagram";

/// One node: the move, what was written around it, and where it hangs.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Node {
    /// As the score sheet wrote it. Only the root has none.
    pub san: Option<String>,
    pub glyphs: Vec<Glyph>,
    pub nags: Vec<u8>,
    pub comments: Vec<String>,
    /// Draw the board at this move.
    pub diagram: bool,
    pub parent: Option<NodeId>,
    /// The alternatives written in this position, the one played first.
    pub children: Vec<NodeId>,
}

/// A game you can change.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Document {
    /// The headers, the result, and the comments after it: everything the score sheet says that
    /// is not a move.
    sheet: Sheet,
    nodes: Vec<Node>,
    cursor: NodeId,
    undo: Vec<Edit>,
    redo: Vec<Edit>,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct Sheet {
    tags: Vec<(String, String)>,
    result: GameResult,
    tail: Vec<String>,
}

/// What one edit touched, in both directions.
#[derive(Clone, PartialEq, Eq, Debug)]
enum Edit {
    /// A child was put into, or taken out of, a parent's list of alternatives.
    Link {
        parent: NodeId,
        at: usize,
        child: NodeId,
        added: bool,
    },
    /// One node's own annotation.
    Note {
        id: NodeId,
        was: Box<Node>,
        now: Box<Node>,
    },
    /// The headers, the result, or the comments after it.
    Sheet { was: Box<Sheet>, now: Box<Sheet> },
}

/// Why an edit was refused. Nothing here leaves the document changed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    /// A move the position at the cursor refuses, offered at this ply of the line.
    Unplayable { ply: usize, text: String },
    /// A node this document never had.
    Unknown(NodeId),
    /// The root is not a move: it has no SAN, and it cannot be removed.
    Root,
    /// The document's own FEN tag does not parse.
    Board(String),
    /// A move whose SAN could not be written.
    Notation(SanError),
}

impl fmt::Display for Error {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Unplayable { ply, text } => {
                write!(out, "move {ply} is not playable in the position: {text}")
            }
            Error::Unknown(id) => write!(out, "no node {id:?} in this document"),
            Error::Root => write!(out, "the root is not a move"),
            Error::Board(reason) => write!(out, "the FEN tag does not parse: {reason}"),
            Error::Notation(reason) => write!(out, "the move cannot be written: {reason:?}"),
        }
    }
}

impl std::error::Error for Error {}

impl Document {
    /// A game from the position chess starts in.
    pub fn new() -> Self {
        Self::from_board(&Board::start())
    }

    /// A game from any position, which the score sheet will write as a FEN tag so it can be read
    /// back. Used by the analysis board as much as by a game imported from move 1.
    pub fn from_board(board: &Board) -> Self {
        let tags = if fen::format(board) == fen::format(&Board::start()) {
            Vec::new()
        } else {
            vec![
                ("FEN".to_string(), fen::format(board)),
                ("SetUp".to_string(), "1".to_string()),
            ]
        };
        Document {
            sheet: Sheet {
                tags,
                ..Default::default()
            },
            nodes: vec![Node::default()],
            cursor: ROOT,
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    /// A score sheet read into an editable tree.
    pub fn parse(source: &str) -> Result<Self, Fault> {
        game::parse(source).map(|game| Self::from_game(&game))
    }

    /// The score sheet this tree turns into.
    pub fn format(&self) -> String {
        game::format(&self.to_game())
    }

    /// A game read by [`super::game`] becomes a document, and [`Document::to_game`] gives back a
    /// game that formats to the same text.
    pub fn from_game(game: &Game) -> Self {
        let mut head = Vec::with_capacity(game.head.len());
        let mut diagram = false;
        for comment in &game.head {
            if comment == DIAGRAM {
                diagram = true;
            } else {
                head.push(comment.clone());
            }
        }
        let mut document = Document {
            sheet: Sheet {
                tags: game.tags.clone(),
                result: game.result,
                tail: game.tail.clone(),
            },
            nodes: vec![Node {
                comments: head,
                diagram,
                ..Default::default()
            }],
            cursor: ROOT,
            undo: Vec::new(),
            redo: Vec::new(),
        };
        document.build(ROOT, &game.moves);
        document
    }

    /// The same tree as a game: the first child of each node is the line the score sheet writes
    /// first, and the rest are the variations in parentheses.
    pub fn to_game(&self) -> Game {
        // The first child of a node is the line the score sheet writes first, and the rest of the
        // alternatives in that position are its variations — including under the root, where the
        // first child is the game's first move.
        let moves = self.line_of(self.get(ROOT).children.first().copied());
        let head = {
            let mut head = self.get(ROOT).comments.clone();
            if self.get(ROOT).diagram {
                head.push(DIAGRAM.to_string());
            }
            head
        };
        Game {
            tags: self.sheet.tags.clone(),
            head,
            moves,
            result: self.sheet.result,
            tail: self.sheet.tail.clone(),
        }
    }

    // --- the tree -------------------------------------------------------

    /// The node itself. Panics only for a [`NodeId`] from another document, which the API never
    /// hands out.
    pub fn get(&self, at: NodeId) -> &Node {
        self.nodes
            .get(at.0 as usize)
            .unwrap_or_else(|| panic!("node {at:?} is not from this document"))
    }

    /// The alternatives written in this position, the one played first.
    pub fn alternatives(&self, at: NodeId) -> &[NodeId] {
        &self.get(at).children
    }

    pub fn parent(&self, at: NodeId) -> Option<NodeId> {
        self.get(at).parent
    }

    /// The node's move as written, which the board and the move list both show.
    pub fn san(&self, at: NodeId) -> Option<&str> {
        self.get(at).san.as_deref()
    }

    pub fn diagram(&self, at: NodeId) -> bool {
        self.get(at).diagram
    }

    /// The nodes from the root to this one, the root first.
    pub fn line(&self, at: NodeId) -> Vec<NodeId> {
        let mut line = vec![at];
        let mut here = at;
        while let Some(above) = self.get(here).parent {
            line.push(above);
            here = above;
        }
        line.reverse();
        line
    }

    /// The line the score sheet writes first: the first child of every node, to the end.
    pub fn main_line(&self) -> Vec<NodeId> {
        let mut line = Vec::new();
        let mut here = self.get(ROOT).children.first().copied();
        while let Some(at) = here {
            line.push(at);
            here = self.get(at).children.first().copied();
        }
        line
    }

    /// How many moves deep this node is. The root is zero.
    pub fn depth(&self, at: NodeId) -> usize {
        self.line(at).len() - 1
    }

    /// The position after the moves from the root to here, which is the position the node's own
    /// move was played in for its children, and the one a diagram of it shows.
    pub fn position(&self, at: NodeId) -> Result<Board, Error> {
        let mut board = self.start()?;
        for id in self.line(at).into_iter().skip(1) {
            let text = self.get(id).san.as_deref().ok_or(Error::Root)?;
            let ply = self.depth(id);
            let mv = san::parse(&board, text).map_err(|_| Error::Unplayable {
                ply,
                text: text.to_string(),
            })?;
            board.apply(mv).map_err(|_| Error::Unplayable {
                ply,
                text: text.to_string(),
            })?;
        }
        Ok(board)
    }

    /// Where the game starts, which the FEN tag may say is not where chess starts.
    pub fn start(&self) -> Result<Board, Error> {
        let probe = Game {
            tags: self.sheet.tags.clone(),
            ..Default::default()
        };
        probe.start().map_err(|fault| match fault {
            Fault::Board(reason) => Error::Board(reason),
            other => Error::Board(other.to_string()),
        })
    }

    // --- the cursor -----------------------------------------------------

    pub fn cursor(&self) -> NodeId {
        self.cursor
    }

    /// Move the cursor. The cursor is where the next move goes, and it is not part of the
    /// document's state, so this is never undone.
    pub fn goto(&mut self, at: NodeId) -> bool {
        if self.nodes.get(at.0 as usize).is_none() {
            return false;
        }
        self.cursor = at;
        true
    }

    pub fn up(&mut self) -> bool {
        self.goto(self.get(self.cursor).parent.unwrap_or(ROOT))
    }

    /// Down to the first alternative, which is the line the score sheet writes first.
    pub fn down(&mut self) -> bool {
        match self.get(self.cursor).children.first() {
            Some(child) => self.goto(*child),
            None => false,
        }
    }

    /// Sideways to the next alternative written in this position.
    pub fn next_sibling(&mut self) -> bool {
        self.sibling(1)
    }

    pub fn prev_sibling(&mut self) -> bool {
        self.sibling(-1)
    }

    /// Follow the first alternatives to the end of this line.
    pub fn to_end(&mut self) {
        while self.down() {}
    }

    fn sibling(&mut self, step: isize) -> bool {
        let at = self.cursor;
        let Some(parent) = self.get(at).parent else {
            return false;
        };
        let siblings = &self.get(parent).children;
        let Some(at) = siblings.iter().position(|sibling| *sibling == at) else {
            return false;
        };
        let next = at.checked_add_signed(step).unwrap_or(siblings.len());
        match siblings.get(next) {
            Some(sibling) => self.goto(*sibling),
            None => false,
        }
    }

    // --- edits ----------------------------------------------------------

    /// Play the move the cursor's position allows, extending the line. A move already written
    /// there moves the cursor instead of writing it twice; a different move becomes another
    /// alternative in the same position, which is what a variation is.
    pub fn play(&mut self, text: &str) -> Result<NodeId, Error> {
        let board = self.position(self.cursor)?;
        let mv = san::parse(&board, text).map_err(|_| Error::Unplayable {
            ply: self.depth(self.cursor) + 1,
            text: text.to_string(),
        })?;
        self.play_move(mv)
    }

    /// The same, for a move the board handed over — a drag, a click, an engine's answer.
    pub fn play_move(&mut self, mv: Move) -> Result<NodeId, Error> {
        let board = self.position(self.cursor)?;
        let written = san::format(&board, mv).map_err(Error::Notation)?;
        if let Some(existing) = self
            .get(self.cursor)
            .children
            .iter()
            .copied()
            .find(|child| self.get(*child).san.as_deref() == Some(written.as_str()))
        {
            self.cursor = existing;
            return Ok(existing);
        }
        let mut next = board;
        next.apply(mv).map_err(|_| Error::Unplayable {
            ply: self.depth(self.cursor) + 1,
            text: written.clone(),
        })?;
        let child = self.alloc(Node {
            san: Some(written),
            parent: Some(self.cursor),
            ..Default::default()
        });
        self.link(self.cursor, child);
        self.cursor = child;
        Ok(child)
    }

    /// Take a move, and everything written under it, out of the tree. The nodes stay until the
    /// edit is undone, which is why undoing this brings the whole variation back.
    pub fn remove(&mut self, at: NodeId) -> Result<(), Error> {
        if at == ROOT {
            return Err(Error::Root);
        }
        let parent = self.get(at).parent.ok_or(Error::Root)?;
        let at = self
            .get(parent)
            .children
            .iter()
            .position(|child| *child == at)
            .ok_or(Error::Unknown(at))?;
        self.unlink(parent, at);
        if !self.reachable(self.cursor) {
            self.cursor = parent;
        }
        Ok(())
    }

    /// Replace what is written under one move.
    pub fn set_comments(&mut self, at: NodeId, comments: Vec<String>) {
        self.note(at, |node| node.comments = comments);
    }

    pub fn add_comment(&mut self, at: NodeId, text: &str) {
        let text = text.to_string();
        self.note(at, |node| node.comments.push(text));
    }

    pub fn set_glyphs(&mut self, at: NodeId, glyphs: Vec<Glyph>) {
        self.note(at, |node| node.glyphs = glyphs);
    }

    pub fn set_nags(&mut self, at: NodeId, nags: Vec<u8>) {
        self.note(at, |node| node.nags = nags);
    }

    /// Embed or take back a diagram.
    pub fn set_diagram(&mut self, at: NodeId, embedded: bool) {
        self.note(at, |node| node.diagram = embedded);
    }

    pub fn set_tag(&mut self, name: &str, value: &str) {
        let name = name.to_string();
        let value = value.to_string();
        self.change_sheet(
            |sheet| match sheet.tags.iter().position(|(key, _)| *key == name) {
                Some(at) => sheet.tags[at].1 = value.clone(),
                None => sheet.tags.push((name.clone(), value)),
            },
        );
    }

    pub fn set_result(&mut self, result: GameResult) {
        self.change_sheet(|sheet| sheet.result = result);
    }

    pub fn add_tail_comment(&mut self, text: &str) {
        let text = text.to_string();
        self.change_sheet(|sheet| sheet.tail.push(text));
    }

    // --- history --------------------------------------------------------

    /// Take one edit back. There is no depth to the history: it runs out only when the edits do.
    pub fn undo(&mut self) -> bool {
        let Some(edit) = self.undo.pop() else {
            return false;
        };
        self.replay(&edit, false);
        self.redo.push(edit);
        self.settle();
        true
    }

    /// Put one edit back after undoing it.
    pub fn redo(&mut self) -> bool {
        let Some(edit) = self.redo.pop() else {
            return false;
        };
        self.replay(&edit, true);
        self.undo.push(edit);
        self.settle();
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// How far back the history goes, which is how far back it can go.
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_depth(&self) -> usize {
        self.redo.len()
    }

    // --- internals ------------------------------------------------------

    fn alloc(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        NodeId(self.nodes.len() as u32 - 1)
    }

    fn link(&mut self, parent: NodeId, child: NodeId) {
        let at = self.get(parent).children.len();
        self.push(Edit::Link {
            parent,
            at,
            child,
            added: true,
        });
    }

    fn unlink(&mut self, parent: NodeId, at: usize) {
        let child = self.get(parent).children[at];
        self.push(Edit::Link {
            parent,
            at,
            child,
            added: false,
        });
    }

    fn note(&mut self, at: NodeId, change: impl FnOnce(&mut Node)) {
        let was = self.get(at).clone();
        let mut now = was.clone();
        change(&mut now);
        if now != was {
            self.push(Edit::Note {
                id: at,
                was: Box::new(was),
                now: Box::new(now),
            });
        }
    }

    fn change_sheet(&mut self, change: impl FnOnce(&mut Sheet)) {
        let was = self.sheet.clone();
        let mut now = was.clone();
        change(&mut now);
        if now != was {
            self.push(Edit::Sheet {
                was: Box::new(was),
                now: Box::new(now),
            });
        }
    }

    /// Apply an edit and remember it, which is what makes a new edit throw the redo stack away.
    fn push(&mut self, edit: Edit) {
        self.replay(&edit, true);
        self.redo.clear();
        self.undo.push(edit);
    }

    fn replay(&mut self, edit: &Edit, forward: bool) {
        match edit {
            Edit::Link {
                parent,
                at,
                child,
                added,
            } => {
                let adding = forward == *added;
                let siblings = &mut self.nodes[parent.0 as usize].children;
                if adding {
                    siblings.insert(*at, *child);
                } else if *at < siblings.len() {
                    siblings.remove(*at);
                }
            }
            Edit::Note { id, was, now } => {
                let node = if forward { now } else { was };
                self.nodes[id.0 as usize] = *node.clone();
            }
            Edit::Sheet { was, now } => {
                self.sheet = *if forward { now } else { was }.clone();
            }
        }
    }

    /// An undo can take the node under the cursor out of the tree. The cursor is a view, not the
    /// document, so it goes somewhere that exists rather than pretending the state is broken.
    fn settle(&mut self) {
        while !self.reachable(self.cursor) {
            // The node is off the tree, but it still knows what it hung under, and that is the
            // nearest place that does exist.
            match self.get(self.cursor).parent {
                Some(above) => self.cursor = above,
                None => self.cursor = ROOT,
            }
        }
    }

    fn reachable(&self, at: NodeId) -> bool {
        let mut here = match self.nodes.get(at.0 as usize) {
            Some(_) => at,
            None => return false,
        };
        loop {
            match self.get(here).parent {
                None => return here == ROOT,
                // A node taken out of the tree still points at its old parent, so the chain is
                // only a line while each node is still one of that parent's alternatives.
                Some(above) => {
                    if !self.get(above).children.contains(&here) {
                        return false;
                    }
                    here = above;
                }
            }
        }
    }

    /// Read the tree under `parent` from a score sheet's line: each move hangs off the previous
    /// one, and each move's variations hang off that move's parent, as alternatives to it.
    fn build(&mut self, parent: NodeId, line: &[MoveText]) {
        let mut above = parent;
        for written in line {
            let mut comments = Vec::with_capacity(written.comments.len());
            let mut diagram = false;
            for comment in &written.comments {
                if comment == DIAGRAM {
                    diagram = true;
                } else {
                    comments.push(comment.clone());
                }
            }
            let at = self.alloc(Node {
                san: Some(written.san.clone()),
                glyphs: written.glyphs.clone(),
                nags: written.nags.clone(),
                comments,
                diagram,
                parent: Some(above),
                children: Vec::new(),
            });
            self.nodes[above.0 as usize].children.push(at);
            for variation in &written.variations {
                self.build(above, variation);
            }
            above = at;
        }
    }

    /// The line starting at a node, following the first alternative, with the rest of each
    /// position's alternatives written as variations.
    fn line_of(&self, from: Option<NodeId>) -> Vec<MoveText> {
        let mut line = Vec::new();
        let mut here = from;
        while let Some(at) = here {
            let node = self.get(at);
            let siblings = match node.parent {
                // The root is not a move, so a line never starts or continues through it.
                None => break,
                Some(parent) => &self.get(parent).children,
            };
            let at = siblings
                .iter()
                .position(|sibling| *sibling == at)
                .unwrap_or_default();
            let alternatives = siblings.get(at + 1..).unwrap_or_default();
            line.push(MoveText {
                san: node.san.clone().unwrap_or_default(),
                glyphs: node.glyphs.clone(),
                nags: node.nags.clone(),
                comments: {
                    let mut comments = node.comments.clone();
                    if node.diagram {
                        comments.push(DIAGRAM.to_string());
                    }
                    comments
                },
                variations: alternatives
                    .iter()
                    .map(|alternative| self.line_of(Some(*alternative)))
                    .collect(),
            });
            here = node.children.first().copied();
        }
        line
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
