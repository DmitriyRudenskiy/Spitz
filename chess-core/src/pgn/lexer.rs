//! PGN tokenizer: splits a character stream into PGN tokens with line numbers.
//!
//! The token set and the per-character dispatch follow `src/pgn_lexer.h` of the reference
//! suite, which is MIT-licensed; its notice is reproduced in `NOTICE.md` and applies to this
//! module. The code here is Rust, with tag values read to their closing quote instead of to
//! the first `]`, and faults reported as values rather than dispatched to a visitor.

use std::borrow::Cow;
use std::fmt;

/// Deepest variation nesting accepted. `docs/pgn-token-coverage.md` records this limit.
pub const MAX_VARIATION_DEPTH: u32 = 32;

/// A suffix annotation attached to a move, and the annotation glyph it stands for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Glyph {
    /// `!`
    GoodMove,
    /// `?`
    BadMove,
    /// `!!`
    ExcellentMove,
    /// `??`
    Blunder,
    /// `!?`
    Interesting,
    /// `?!`
    Dubious,
}

impl Glyph {
    /// The `$<n>` number this suffix is equivalent to.
    pub const fn nag(self) -> u8 {
        match self {
            Glyph::GoodMove => 1,
            Glyph::BadMove => 2,
            Glyph::ExcellentMove => 3,
            Glyph::Blunder => 4,
            Glyph::Interesting => 5,
            Glyph::Dubious => 6,
        }
    }

    const fn symbol(self) -> &'static str {
        match self {
            Glyph::GoodMove => "!",
            Glyph::BadMove => "?",
            Glyph::ExcellentMove => "!!",
            Glyph::Blunder => "??",
            Glyph::Interesting => "!?",
            Glyph::Dubious => "?!",
        }
    }

    fn from_symbol(text: &str) -> Option<Self> {
        Some(match text {
            "!" => Glyph::GoodMove,
            "?" => Glyph::BadMove,
            "!!" => Glyph::ExcellentMove,
            "??" => Glyph::Blunder,
            "!?" => Glyph::Interesting,
            "?!" => Glyph::Dubious,
            _ => return None,
        })
    }
}

/// The game termination marker.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
    #[default]
    Unfinished,
}

impl GameResult {
    const fn symbol(self) -> &'static str {
        match self {
            GameResult::WhiteWins => "1-0",
            GameResult::BlackWins => "0-1",
            GameResult::Draw => "1/2-1/2",
            GameResult::Unfinished => "*",
        }
    }
}

/// One PGN token. Comments and tag values keep their source text so a round trip cannot
/// quietly rewrite them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token<'a> {
    /// `[Name "value"]`. `value` excludes the quotes and still holds the backslash escapes;
    /// [`unescape_tag_value`] decodes it.
    TagPair { name: &'a str, value: &'a str },
    /// `{text}`, which may span lines.
    BraceComment(&'a str),
    /// `;text`, a comment to the end of the line.
    LineComment(&'a str),
    /// `%text`, an escape comment starting in column one.
    Escape(&'a str),
    /// `$<n>`, an annotation glyph.
    Nag(u8),
    /// One of the six suffix annotations.
    Suffix(Glyph),
    /// A move number such as the `12` of `12.` or `12...`.
    MoveNumber(&'a str),
    /// A move, or anything else that is a symbol but neither a move number nor a result.
    SanMove(&'a str),
    /// One of the four game termination markers.
    Result(GameResult),
    /// `(`
    VariationStart,
    /// `)`
    VariationEnd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LexError {
    pub line: u32,
    pub kind: LexErrorKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexErrorKind {
    /// A `[` in the movetext section, where a tag pair cannot appear.
    TagInMovetext,
    /// A tag pair that reaches the end of the input, or whose `]` is missing, unterminated.
    UnterminatedTag,
    /// A brace comment that reaches the end of the input unterminated.
    UnterminatedComment,
    /// `$` with no digits, or with a number outside 1..=255.
    MalformedNag,
    /// A run of `!` and `?` longer than two characters.
    MalformedGlyph,
    /// A `%` escape comment not in column one.
    EscapeNotInFirstColumn,
    /// `)` with no matching `(`.
    UnbalancedVariation,
    /// More than [`MAX_VARIATION_DEPTH`] nested variations.
    VariationTooDeep,
    /// A character that cannot start a token.
    UnexpectedChar(char),
}

/// Decode the backslash escapes PGN allows inside a tag value: `\"` and `\\`.
pub fn unescape_tag_value(value: &str) -> Cow<'_, str> {
    if !value.contains('\\') {
        return Cow::Borrowed(value);
    }
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(at) = rest.find('\\') {
        out.push_str(&rest[..at]);
        rest = &rest[at + 1..];
        match rest.chars().next() {
            Some(escaped @ ('"' | '\\')) => {
                out.push(escaped);
                rest = &rest[escaped.len_utf8()..];
            }
            Some(other) => out.push(other),
            None => break,
        }
    }
    out.push_str(rest);
    Cow::Owned(out)
}

/// The centiseconds carried by a PGN clock escape (`%t7300`) inside a comment, if any.
pub fn clock_centi_seconds(comment: &str) -> Option<u32> {
    let at = comment.find("%t")? + 2;
    let digits = comment[at..].bytes().take_while(u8::is_ascii_digit).count();
    comment[at..][..digits].parse().ok()
}

/// Splits PGN text into tokens. Iteration ends after the first fault.
pub struct Lexer<'a> {
    src: &'a str,
    at: usize,
    line: u32,
    depth: u32,
    movetext: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, at: 0, line: 1, depth: 0, movetext: false }
    }

    /// Variation nesting reached so far, for a caller that wants to reject unbalanced games.
    pub const fn depth(&self) -> u32 {
        self.depth
    }

    fn rest(&self) -> &'a str {
        &self.src[self.at..]
    }

    fn fail(&self, line: u32, kind: LexErrorKind) -> Result<Option<Token<'a>>, LexError> {
        Err(LexError { line, kind })
    }

    fn count_lines(&mut self, text: &str) {
        self.line += text.matches('\n').count() as u32;
    }

    /// Consume up to and including `byte`, or to the end of the input. The second element of
    /// the result says whether `byte` was found.
    fn eat_to(&mut self, byte: char) -> (&'a str, bool) {
        let rest = self.rest();
        match rest.find(byte) {
            Some(end) => {
                let found = &rest[..end];
                self.at += end + 1;
                self.count_lines(found);
                (found, true)
            }
            None => {
                let found = rest;
                self.at = self.src.len();
                self.count_lines(found);
                (found, false)
            }
        }
    }

    fn eat_symbol(&mut self) -> &'a str {
        let rest = self.rest();
        let end = rest.find(|c: char| !is_symbol(c)).unwrap_or(rest.len());
        let found = &rest[..end];
        self.at += end;
        found
    }

    fn skip_spaces(&mut self) {
        let rest = self.rest();
        let end = rest.find(|c: char| !c.is_whitespace()).unwrap_or(rest.len());
        self.count_lines(&rest[..end]);
        self.at += end;
    }

    fn next_token(&mut self) -> Result<Option<Token<'a>>, LexError> {
        loop {
        let Some(first) = self.rest().chars().next() else {
            return Ok(None);
        };
        let line = self.line;
        let column = self.at;
        self.at += first.len_utf8();
        let token = match first {
            ' ' | '\t' | '\r' | '\u{b}' | '.' | '<' | '>' => continue,
            '\n' => {
                self.line += 1;
                continue;
            }
            '[' => {
                if self.movetext {
                    return self.fail(line, LexErrorKind::TagInMovetext);
                }
                return self.tag_pair(line);
            }
            '{' => {
                let (text, closed) = self.eat_to('}');
                if !closed {
                    return self.fail(line, LexErrorKind::UnterminatedComment);
                }
                Token::BraceComment(text)
            }
            ';' => Token::LineComment(self.eat_to('\n').0),
            '%' => {
                if column > 0 && !self.src[..column].ends_with('\n') {
                    return self.fail(line, LexErrorKind::EscapeNotInFirstColumn);
                }
                Token::Escape(self.eat_to('\n').0)
            }
            '$' => match self.eat_symbol().parse::<u8>() {
                Ok(0) | Err(_) => return self.fail(line, LexErrorKind::MalformedNag),
                Ok(n) => Token::Nag(n),
            },
            '!' | '?' => {
                let start = column;
                while let Some('!' | '?') = self.rest().chars().next() {
                    self.at += 1;
                }
                let text = &self.src[start..self.at];
                match Glyph::from_symbol(text) {
                    Some(glyph) => Token::Suffix(glyph),
                    None => return self.fail(line, LexErrorKind::MalformedGlyph),
                }
            }
            '(' => {
                self.depth += 1;
                if self.depth > MAX_VARIATION_DEPTH {
                    return self.fail(line, LexErrorKind::VariationTooDeep);
                }
                Token::VariationStart
            }
            ')' => {
                if self.depth == 0 {
                    return self.fail(line, LexErrorKind::UnbalancedVariation);
                }
                self.depth -= 1;
                Token::VariationEnd
            }
            '*' => {
                self.movetext = false;
                Token::Result(GameResult::Unfinished)
            }
            _ if is_symbol(first) => {
                self.at = column;
                let text = self.eat_symbol();
                self.movetext = true;
                if let Some(result) = termination(text) {
                    self.movetext = false;
                    Token::Result(result)
                } else if text.bytes().all(|c| c.is_ascii_digit()) {
                    Token::MoveNumber(text)
                } else {
                    Token::SanMove(text)
                }
            }
            _ => return self.fail(line, LexErrorKind::UnexpectedChar(first)),
        };
        return Ok(Some(token));
        }
    }

    /// A tag pair, whose value runs to its closing quote and may hold escaped quotes.
    fn tag_pair(&mut self, line: u32) -> Result<Option<Token<'a>>, LexError> {
        self.skip_spaces();
        let name = self.eat_symbol();
        self.skip_spaces();
        if !self.rest().starts_with('"') {
            return self.fail(line, LexErrorKind::UnterminatedTag);
        }
        self.at += 1;
        let bytes = self.rest().as_bytes();
        let mut quote = 0;
        loop {
            match bytes.get(quote) {
                Some(b'\\') => quote += 2,
                Some(b'"') => break,
                Some(_) => quote += 1,
                None => return self.fail(line, LexErrorKind::UnterminatedTag),
            }
        }
        let value = &self.src[self.at..self.at + quote];
        self.at += quote + 1;
        self.count_lines(value);
        self.skip_spaces();
        if !self.rest().starts_with(']') {
            return self.fail(line, LexErrorKind::UnterminatedTag);
        }
        self.at += 1;
        Ok(Some(Token::TagPair { name, value }))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(None) => None,
            Ok(Some(token)) => Some(Ok(token)),
            Err(error) => {
                self.at = self.src.len();
                Some(Err(error))
            }
        }
    }
}

/// Symbol characters: letters, digits, and the punctuation a PGN symbol may hold.
fn is_symbol(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(c, '_' | '+' | '#' | '=' | ':' | '-' | '/' | '~' | ',' | '@')
}

/// The four termination markers, accepting the `:` and `1/2` spellings seen in the wild.
fn termination(text: &str) -> Option<GameResult> {
    match text {
        "1-0" | "1:0" => Some(GameResult::WhiteWins),
        "0-1" | "0:1" => Some(GameResult::BlackWins),
        "1/2" | "1/2-1/2" | "1/2:1/2" => Some(GameResult::Draw),
        _ => None,
    }
}

/// Re-emit a token in canonical form. Whitespace and move-number punctuation are normalised,
/// so a round trip preserves the tokens rather than the bytes.
pub fn write_token(token: Token<'_>, out: &mut String) {
    match token {
        Token::TagPair { name, value } => {
            out.push('[');
            out.push_str(name);
            out.push_str(" \"");
            out.push_str(value);
            out.push_str("\"] ");
        }
        Token::BraceComment(text) => {
            out.push('{');
            out.push_str(text);
            out.push_str("} ");
        }
        Token::LineComment(text) => {
            out.push(';');
            out.push_str(text.trim_end_matches('\n'));
            out.push('\n');
        }
        Token::Escape(text) => {
            out.push('\n');
            out.push('%');
            out.push_str(text.trim_end_matches('\n'));
            out.push('\n');
        }
        Token::Nag(n) => out.push_str(&format!("${n} ")),
        Token::Suffix(glyph) => {
            out.push_str(glyph.symbol());
            out.push(' ');
        }
        Token::MoveNumber(text) => {
            out.push_str(text);
            out.push_str(". ");
        }
        Token::SanMove(text) => {
            out.push_str(text);
            out.push(' ');
        }
        Token::Result(result) => {
            out.push_str(result.symbol());
            out.push(' ');
        }
        Token::VariationStart => out.push_str("( "),
        Token::VariationEnd => out.push_str(") "),
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {:?}", self.line, self.kind)
    }
}

#[cfg(test)]
mod tests {
    use super::LexErrorKind::*;
    use super::*;

    fn lex(src: &str) -> Result<Vec<Token<'_>>, LexError> {
        Lexer::new(src).collect()
    }

    fn kind(src: &str) -> LexError {
        lex(src).expect_err("faults")
    }

    /// Every form the tokenizer has to tolerate, in one game.
    const TOLERATED: &str = "%t7300 an escape comment in column one\n\
         [Event \"?\"]\n\
         [Site \"Berlin \\\"East\\\"\"]\n\
         [Date \"2011.05.??.??\"]\n\
         [Annotator \"back\\\\slash\"]\n\
         [Result \"*\"]\n\
         1. e4 {a brace comment\nspanning lines, with %t7300 inside} e5\n\
         ; a semicolon comment\n\
         2. Nf3 g6 $3 3. h4!? Bg7?! 4. a3! Bg4?? 5. Rh2!! Nf6?\n\
         5. d4 (5... cxd4 6. cxd4 (6... Bb4+ 7. Qd2 Bxd2+ 8. Nbxd2 Nf6)) 1-0\n";

    #[test]
    fn tolerates_every_form_and_round_trips() {
        let tokens = lex(TOLERATED).expect("no fault");
        let mut written = String::new();
        for token in &tokens {
            write_token(*token, &mut written);
        }
        assert_eq!(lex(&written).expect("the written form re-lexes"), tokens);
        assert!(matches!(tokens[0], Token::Escape("t7300 an escape comment in column one")));
        assert!(matches!(tokens[2], Token::TagPair { name: "Site", value } if value == "Berlin \\\"East\\\""));
        assert!(matches!(tokens[4], Token::TagPair { name: "Annotator", value } if value == "back\\\\slash"));
        assert!(matches!(tokens[8], Token::BraceComment(text)
            if text.contains('\n') && clock_centi_seconds(text) == Some(7300)));
        assert!(matches!(tokens[10], Token::LineComment(" a semicolon comment")));
        assert!(tokens.contains(&Token::Nag(3)));
        assert!(matches!(tokens[11], Token::MoveNumber("2")));
    }

    #[test]
    fn the_six_suffixes_map_to_their_annotation_glyphs() {
        let tokens = lex("e4! e5? Nf3!! Ng6?? a3!? h6?!").expect("no fault");
        let suffixes: Vec<u8> = tokens
            .iter()
            .filter_map(|token| match token {
                Token::Suffix(glyph) => Some(glyph.nag()),
                _ => None,
            })
            .collect();
        assert_eq!(suffixes, [1, 2, 3, 4, 5, 6]);
        assert_eq!(
            tokens.iter().find_map(|t| match t {
                Token::Suffix(Glyph::Dubious) => Some("?!"),
                _ => None,
            }),
            Some("?!")
        );
    }

    #[test]
    fn escaped_quotes_do_not_end_a_tag_value() {
        let tokens = lex("[Site \"Berlin \\\"East\\\"\"]\n[Round \"1\"]").expect("no fault");
        assert_eq!(
            tokens,
            [
                Token::TagPair { name: "Site", value: "Berlin \\\"East\\\"" },
                Token::TagPair { name: "Round", value: "1" },
            ]
        );
        assert_eq!(unescape_tag_value("Berlin \\\"East\\\""), "Berlin \"East\"");
        assert_eq!(unescape_tag_value("back\\\\slash"), "back\\slash");
        assert_eq!(unescape_tag_value("plain"), "plain");
    }

    #[test]
    fn a_tag_pair_may_span_lines() {
        let tokens = lex("[PlyCount \"80\"\n ]").expect("no fault");
        assert_eq!(tokens, [Token::TagPair { name: "PlyCount", value: "80" }]);
    }

    #[test]
    fn faults_name_the_line() {
        assert_eq!(kind("1. e4 e5\n2. Nf3 [Event \"late\"]\n").kind, TagInMovetext);
        assert_eq!(kind("1. e4 e5\n2. Nf3 [Event \"late\"]\n").line, 2);
        assert_eq!(kind("1. e4 {open\nstill open\n").kind, UnterminatedComment);
        assert_eq!(kind("1. e4 {open\nstill open\n").line, 1);
        assert_eq!(kind("[Event \"x\n").kind, UnterminatedTag);
        assert_eq!(kind("1. e4 %escape\n").kind, EscapeNotInFirstColumn);
        assert_eq!(kind("1. e4 $x\n").kind, MalformedNag);
        assert_eq!(kind("1. e4 $\n").kind, MalformedNag);
        assert_eq!(kind("1. e4 $0\n").kind, MalformedNag);
        assert_eq!(kind("1. e4 $256\n").kind, MalformedNag);
        assert_eq!(kind("1. e4!!!\n").kind, MalformedGlyph);
        assert_eq!(kind("1. e4)\n").kind, UnbalancedVariation);
        assert_eq!(kind("1. e4\n\n\"stray\n").kind, UnexpectedChar('"'));
        assert_eq!(kind("1. e4\n\n\"stray\n").line, 3);
    }

    #[test]
    fn accepts_deeply_nested_variations_up_to_the_recorded_limit() {
        for depth in [1, 8, 16, MAX_VARIATION_DEPTH] {
            let src = format!("{}e4{}", "(".repeat(depth as usize), ")".repeat(depth as usize));
            let tokens = lex(&src).unwrap_or_else(|e| panic!("depth {depth}: {e}"));
            assert_eq!(tokens.iter().filter(|t| matches!(t, Token::VariationStart)).count(), depth as usize);
        }
        let too_deep = format!("{}e4{}", "(".repeat(33), ")".repeat(33));
        assert_eq!(kind(&too_deep).kind, VariationTooDeep);
    }

    #[test]
    fn results_and_move_numbers_are_told_apart() {
        let tokens = lex("12. e4 1/2-1/2 1-0 0-1 * 34").expect("no fault");
        assert_eq!(
            tokens,
            [
                Token::MoveNumber("12"),
                Token::SanMove("e4"),
                Token::Result(GameResult::Draw),
                Token::Result(GameResult::WhiteWins),
                Token::Result(GameResult::BlackWins),
                Token::Result(GameResult::Unfinished),
                Token::MoveNumber("34"),
            ]
        );
    }
}

