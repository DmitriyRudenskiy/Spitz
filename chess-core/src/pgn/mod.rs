//! PGN tokens: the tokenizer and the multi-game split.

pub mod document;
pub mod game;
pub mod lexer;
pub mod stream;

pub use document::{DIAGRAM, Document, Error as DocumentError, Node, NodeId, ROOT};
pub use game::{Fault, Game, MoveText, decode, escape_tag_value, format, parse, parse_stream};
pub use lexer::{
    GameResult, Glyph, LexError, LexErrorKind, Lexer, MAX_VARIATION_DEPTH, Token,
    clock_centi_seconds, termination, unescape_tag_value, write_token,
};
pub use stream::{Games, split_games};
