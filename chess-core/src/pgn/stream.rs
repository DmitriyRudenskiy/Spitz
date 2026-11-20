//! Splitting a PGN character stream into games at the termination markers.

use super::lexer::{LexError, LexErrorKind, Lexer, Token};

/// The result of a split: the games that parsed, and the fault that stopped the stream.
pub struct Games<'a> {
    pub games: Vec<Vec<Token<'a>>>,
    pub fault: Option<LexError>,
}

/// A game ends where the next one starts: at the first header of the next game, once this game
/// reached its termination marker. The marker does not end it, because what is written after the
/// result annotates the game and belongs to it. When the stream faults, the games that already
/// parsed are returned along with the fault.
pub fn split_games(src: &str) -> Games<'_> {
    let mut lexer = Lexer::new(src);
    let mut games: Vec<Vec<Token>> = Vec::new();
    let mut current: Vec<Token> = Vec::new();
    let mut fault = None;
    // Whether the game being read has signed itself off, so the next header starts another.
    let mut ended = false;
    while let Some(item) = lexer.next() {
        match item {
            Ok(token) => {
                // Anything but an annotation, after a game signed itself off, starts the next one.
                if ended
                    && !matches!(
                        token,
                        Token::BraceComment(_) | Token::LineComment(_) | Token::Escape(_)
                    )
                {
                    games.push(std::mem::take(&mut current));
                    ended = false;
                }
                let marks_the_end = matches!(token, Token::Result(_)) && lexer.depth() == 0;
                current.push(token);
                ended |= marks_the_end;
            }
            Err(error) => {
                fault = Some(error);
                break;
            }
        }
    }
    // A stream that ends inside a variation never finished the game it was in.
    if fault.is_none() && lexer.depth() > 0 {
        fault = Some(LexError {
            line: lexer.opened_at(),
            kind: LexErrorKind::UnbalancedVariation,
        });
    }
    if fault.is_none() && !current.is_empty() {
        games.push(current);
    }
    // A game that never reached a marker is still a game: the caller says how it ended.
    Games { games, fault }
}

#[cfg(test)]
mod tests {
    use super::super::lexer::GameResult;
    use super::*;

    const GAME: &str = "[Event \"Rated\"]\n\
                        [Date \"2011.05.??.\"]\n\
                        1. e4 e5 2. Nf3 *";

    #[test]
    fn a_termination_marker_ends_a_game() {
        let src = format!("{GAME}\n\n{GAME}\n");
        let stream = split_games(&src);
        assert_eq!(stream.fault, None);
        assert_eq!(stream.games.len(), 2);
        for game in &stream.games {
            assert_eq!(game.last(), Some(&Token::Result(GameResult::Unfinished)));
        }
    }

    #[test]
    fn a_result_inside_a_variation_does_not_end_the_game() {
        let stream = split_games("1. e4 (1. Nf3 1/2-1/2) 1... e5 2. Nf3 1-0");
        assert_eq!(stream.fault, None);
        assert_eq!(stream.games.len(), 1);
    }

    #[test]
    fn games_before_a_fault_are_returned() {
        let src = format!("{GAME}\n\n1. e4 [Event \"late\"]\n");
        let stream = split_games(&src);
        assert_eq!(stream.games.len(), 1);
        assert_eq!(
            stream.games[0].last(),
            Some(&Token::Result(GameResult::Unfinished))
        );
        let fault = stream.fault.expect("the second game faults");
        assert_eq!(fault.line, 5);
    }

    #[test]
    fn a_stream_of_only_a_fault_returns_no_games() {
        let stream = split_games("[Event \"a");
        assert!(stream.games.is_empty());
        assert_eq!(stream.fault.map(|e| e.line), Some(1));
    }
}
