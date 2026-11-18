//! A game as a document: the tags, the moves with everything written around them, the result.
//!
//! The lexer turns a stream into tokens and [`super::stream`] finds where one game stops and the
//! next begins. This module turns those tokens into something a program can act on, and back again.
//! The shape keeps the main line flat and hangs each variation off the move it branches from,
//! which is how a score sheet reads and how an editor wants to insert one.

use std::fmt;

use crate::board::{Board, Colour};
use crate::fen;
use crate::san;

use super::lexer::{
    GameResult, Glyph, LexError, Token, termination, unescape_tag_value, write_token,
};

/// One move, with the annotation written around it and the lines that branch off before it.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct MoveText {
    /// The move as the score sheet wrote it, which is not always the least a reader needs.
    pub san: String,
    pub glyphs: Vec<Glyph>,
    pub nags: Vec<u8>,
    pub comments: Vec<String>,
    /// Alternative lines, each starting from the position before this move was played.
    pub variations: Vec<Vec<MoveText>>,
}

/// A game: what was recorded about it, what was played, and how it ended.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Game {
    /// Tag pairs in the order they were written, because that order is the format.
    pub tags: Vec<(String, String)>,
    /// Comments before the first move.
    pub head: Vec<String>,
    pub moves: Vec<MoveText>,
    pub result: GameResult,
    /// Comments after the termination marker.
    pub tail: Vec<String>,
}

impl Game {
    /// The position the game starts from, which the tags may say is not the one chess starts from.
    pub fn start(&self) -> Result<Board, Fault> {
        let setup = self.tag("SetUp").is_some_and(|value| value == "1");
        match (setup, self.tag("FEN")) {
            (true, Some(text)) => {
                fen::parse(text).map_err(|reason| Fault::Board(reason.to_string()))
            }
            _ => Ok(Board::start()),
        }
    }

    /// The value of a tag, by name.
    pub fn tag(&self, name: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// Why a stream of tokens is not a game.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Fault {
    /// A token that cannot be a token, with the line it was found on.
    Lex(LexError),
    /// Nothing in the stream was a game.
    Empty,
    /// A glyph or a NAG with no move to attach to.
    Stray,
    /// A move the position refuses, with where it was written.
    Move { line: u32, ply: usize, text: String },
    /// A FEN tag that does not parse.
    Board(String),
}

impl fmt::Display for Fault {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // The lexer fault already says which line it happened on.
            Fault::Lex(fault) => write!(out, "{fault}"),
            Fault::Empty => write!(out, "the stream holds no game"),
            Fault::Stray => write!(out, "an annotation with no move to attach to"),
            Fault::Move { line, ply, text } => {
                write!(
                    out,
                    "line {line}: move {ply} is not playable in the position: {text}"
                )
            }
            Fault::Board(reason) => write!(out, "the FEN tag does not parse: {reason}"),
        }
    }
}

impl std::error::Error for Fault {}

/// One game. A file of concatenated games goes through [`parse_stream`], which splits first.
pub fn parse(source: &str) -> Result<Game, Fault> {
    let stream = super::stream::split_games(source);
    if let Some(fault) = stream.fault {
        return Err(Fault::Lex(fault));
    }
    let only = stream.games.first().ok_or(Fault::Empty)?;
    decode(only, source)
}

/// Every game of a concatenated stream, and the fault that stopped the stream if one did. Games
/// that parsed before the fault come back with it: a file of ten thousand games should not be
/// discarded because the nine-thousand-and-first has an unclosed brace.
pub fn parse_stream(source: &str) -> Result<(Vec<Game>, Option<Fault>), Fault> {
    let stream = super::stream::split_games(source);
    let fault = stream.fault.map(Fault::Lex);
    let mut games = Vec::with_capacity(stream.games.len());
    for tokens in &stream.games {
        match decode(tokens, source) {
            Ok(game) => games.push(game),
            // A game that does not decode is a fault of its own, and the games before it stand.
            Err(error) => return Ok((games, Some(error))),
        }
    }
    Ok((games, fault))
}

/// The tokens of one game, already split from any neighbours. `source` is only there so a fault can
/// say which line it happened on: every token is a slice of it.
pub fn decode(tokens: &[Token], source: &str) -> Result<Game, Fault> {
    let mut game = Game::default();
    let mut body = Vec::new();
    let mut marked = false;
    for token in tokens {
        match *token {
            Token::TagPair { name, value } => game
                .tags
                .push((name.to_string(), unescape_tag_value(value).into_owned())),
            Token::Result(result) => {
                game.result = result;
                marked = true;
            }
            other => body.push(other),
        }
    }
    // A game whose move text never reached a termination marker still says how it ended, in the
    // Result tag. The tag is the record; the move text is the transcript.
    if !marked {
        game.result = game
            .tag("Result")
            .and_then(termination)
            .unwrap_or(GameResult::Unfinished);
    }
    let board = game.start()?;
    let mut cursor = Cursor {
        tokens: &body,
        at: 0,
    };
    let (moves, head, tail) = line(&mut cursor, &board, 1, source)?;
    game.moves = moves;
    game.head = head;
    game.tail = tail;
    Ok(game)
}

struct Cursor<'a> {
    tokens: &'a [Token<'a>],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn peek(&self) -> Option<Token<'a>> {
        self.tokens.get(self.at).copied()
    }

    fn take(&mut self) -> Option<Token<'a>> {
        let token = self.peek();
        if token.is_some() {
            self.at += 1;
        }
        token
    }
}

/// What a line reads as: its moves, the comments before the first of them, and the comments after
/// the termination marker.
type Line = (Vec<MoveText>, Vec<String>, Vec<String>);

/// Read moves until the line ends: at a result, at a closing parenthesis, or at the tokens.
/// Comments before the first move come back as the head, comments after the result as the tail.
fn line(cursor: &mut Cursor, board: &Board, ply: usize, source: &str) -> Result<Line, Fault> {
    let mut moves: Vec<MoveText> = Vec::new();
    let mut head: Vec<String> = Vec::new();
    let mut tail: Vec<String> = Vec::new();
    // Where each move started from, so a variation can branch from the right position without
    // unwinding anything.
    let mut before: Vec<Board> = vec![*board];
    let mut early: Vec<Vec<MoveText>> = Vec::new();
    loop {
        match cursor.peek() {
            Some(Token::BraceComment(text) | Token::LineComment(text) | Token::Escape(text)) => {
                let text = text.trim().to_string();
                cursor.take();
                match moves.last_mut() {
                    Some(move_) => move_.comments.push(text),
                    None => head.push(text),
                }
            }
            Some(Token::Suffix(glyph)) => {
                cursor.take();
                moves.last_mut().ok_or(Fault::Stray)?.glyphs.push(glyph);
            }
            Some(Token::Nag(nag)) => {
                cursor.take();
                moves.last_mut().ok_or(Fault::Stray)?.nags.push(nag);
            }
            Some(Token::MoveNumber(_)) => {
                // The number is written for whoever reads the score sheet. The position already
                // knows whose move it is, and the writer recomputes what it needs.
                cursor.take();
            }
            Some(Token::SanMove(text)) => {
                cursor.take();
                let from = *before.last().expect("a line starts with a position");
                let fault = |ply: usize| Fault::Move {
                    line: line_of(source, text),
                    ply,
                    text: text.to_string(),
                };
                let mv = san::parse(&from, text).map_err(|_| fault(ply + moves.len()))?;
                let mut next = from;
                next.apply(mv).map_err(|_| fault(ply + moves.len()))?;
                before.push(next);
                moves.push(MoveText {
                    san: text.to_string(),
                    ..Default::default()
                });
                moves.last_mut().unwrap().variations.append(&mut early);
            }
            Some(Token::VariationStart) => {
                cursor.take();
                // A variation written after a move is an alternative to that move, so it starts
                // from the position before it. Written before any move, it is an alternative to
                // the game, and waits for the first move to hang off.
                let from = before[before.len() - 1 - (before.len() > 1) as usize];
                let (branch, _, _) = line(cursor, &from, ply, source)?;
                match moves.last_mut() {
                    Some(move_) => move_.variations.push(branch),
                    None => early.push(branch),
                }
            }
            Some(Token::VariationEnd) => {
                cursor.take();
                break;
            }
            Some(Token::Result(_)) => {
                cursor.take();
                // Anything after the result annotates the game, not the last move.
                while let Some(token) = cursor.take() {
                    if let Token::BraceComment(text)
                    | Token::LineComment(text)
                    | Token::Escape(text) = token
                    {
                        tail.push(text.trim().to_string());
                    }
                }
                break;
            }
            Some(Token::TagPair { .. }) | None => break,
        }
    }
    Ok((moves, head, tail))
}

/// Which line of the source a token was read from. Every token is a slice of the same string, so
/// the distance from its start is the distance into the file.
fn line_of(source: &str, token: &str) -> u32 {
    let start = token.as_ptr() as usize - source.as_ptr() as usize;
    // Saturating, so a token from somewhere else reports line 1 instead of unwinding.
    let start = start.min(source.len());
    1 + source[..start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count() as u32
}

/// The score sheet a game turns into.
pub fn format(game: &Game) -> String {
    let mut text = String::with_capacity(256);
    for (name, value) in &game.tags {
        text.push_str(&format!("[{name} \"{}\"]\n", escape_tag_value(value)));
    }
    if !game.tags.is_empty() {
        text.push('\n');
    }
    let board = game.start().unwrap_or_else(|_| Board::start());
    let mut body = String::new();
    for comment in &game.head {
        write_token(Token::BraceComment(comment), &mut body);
    }
    write_moves(&mut body, &game.moves, board.turn(), board.fullmove());
    write_token(Token::Result(game.result), &mut body);
    for comment in &game.tail {
        write_token(Token::BraceComment(comment), &mut body);
    }
    text.push_str(body.trim_end());
    text.push('\n');
    text
}

/// Write a line of moves, numbering them as the position says they should be numbered.
fn write_moves(out: &mut String, moves: &[MoveText], colour: Colour, number: u16) {
    let mut colour = colour;
    let mut number = number;
    for (at, move_) in moves.iter().enumerate() {
        if colour == Colour::White {
            out.push_str(&format!("{number}. "));
        } else if at == 0 {
            out.push_str(&format!("{number}... "));
        }
        write_token(Token::SanMove(&move_.san), out);
        for glyph in &move_.glyphs {
            write_token(Token::Suffix(*glyph), out);
        }
        for nag in &move_.nags {
            write_token(Token::Nag(*nag), out);
        }
        for comment in &move_.comments {
            write_token(Token::BraceComment(comment), out);
        }
        for variation in &move_.variations {
            out.push('(');
            let mut inner = String::new();
            // A variation branches from the position before this move, so it opens with the same
            // colour's move and the same number.
            write_moves(&mut inner, variation, colour, number);
            out.push_str(inner.trim_end());
            out.push_str(") ");
        }
        if colour == Colour::Black {
            number += 1;
        }
        colour = colour.other();
    }
}

/// Quote what a tag value needs quoted: a double quote and a backslash.
pub fn escape_tag_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    for ch in value.chars() {
        if ch == '"' || ch == '\\' {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}
