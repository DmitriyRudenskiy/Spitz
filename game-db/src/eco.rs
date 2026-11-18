//! ECO classification: a user-supplied code file, a prefix trie over main-line moves, and the
//! whole-library pass.
//!
//! `docs/eco-code-format.md` is the format note this implements. Matching is by move sequence,
//! longest entry wins, and a game matching nothing keeps an empty code.

use crate::session::Session;
use crate::session::SessionError;
use chess_core::pgn::lexer::{Lexer, Token};
use std::collections::HashMap;
use std::fmt;

/// The code a game carries when nothing matched.
pub const NO_ECO: u16 = 0;

/// The header record holds 16 bits, and zero is `NO_ECO`.
pub const MAX_CODES: usize = u16::MAX as usize;

/// One line of a code file, normalised.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EcoEntry {
    pub code: String,
    pub moves: Vec<String>,
}

/// A line that could not be used, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EcoIssue {
    pub line: usize,
    pub reason: &'static str,
}

impl fmt::Display for EcoIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.reason)
    }
}

/// Parse a code file, returning the usable entries and every line that was skipped. A file with
/// a few bad lines still loads; the caller shows the issues.
pub fn parse(text: &str) -> (Vec<EcoEntry>, Vec<EcoIssue>) {
    let mut entries = Vec::new();
    let mut issues = Vec::new();
    let mut seen: HashMap<String, String> = HashMap::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        // A hash starts a comment and never appears in a move, so the line ends there.
        let text = raw.split('#').next().unwrap_or("").trim();
        if text.is_empty() {
            continue;
        }
        let (code, moves) = match text.split_once(char::is_whitespace) {
            Some((code, rest)) => (code, rest),
            None => {
                issues.push(EcoIssue { line, reason: "no move sequence" });
                continue;
            }
        };
        let Some(code) = normalize_code(code) else {
            issues.push(EcoIssue { line, reason: "not a code" });
            continue;
        };
        let moves = match parse_moves(moves) {
            Some(moves) => moves,
            None => {
                issues.push(EcoIssue { line, reason: "no moves after the code" });
                continue;
            }
        };
        let sequence = moves.join("/");
        if let Some(first) = seen.get(&sequence) {
            issues.push(EcoIssue {
                line,
                reason: if first == &code {
                    "the same sequence again"
                } else {
                    "another code for a sequence already in the file; the first line wins"
                },
            });
            continue;
        }
        seen.insert(sequence, code.clone());
        entries.push(EcoEntry { code, moves });
    }
    (entries, issues)
}

/// Upper-case a code, rejecting anything that is not 2 to 6 characters starting with a letter.
pub fn normalize_code(code: &str) -> Option<String> {
    let code = code.trim();
    if code.len() < 2 || code.len() > 6 {
        return None;
    }
    let mut chars = code.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    if !code.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    Some(code.to_ascii_uppercase())
}

/// Move tokens, main line only, with numbers, results, NAGs and annotation suffixes gone.
fn parse_moves(text: &str) -> Option<Vec<String>> {
    let mut moves = Vec::new();
    for token in text.split_whitespace() {
        if token.starts_with('$') || token == "..." || token == ".." {
            continue;
        }
        if is_result(token) {
            break;
        }
        let bare = token.trim_end_matches('.');
        if !bare.is_empty() && bare.chars().all(|c| c.is_ascii_digit()) {
            continue; // a move number, with or without its dots
        }
        let move_ = normalize_move(token);
        if move_.is_empty() {
            return None;
        }
        moves.push(move_);
    }
    (!moves.is_empty()).then_some(moves)
}

fn is_result(token: &str) -> bool {
    matches!(token, "1-0" | "0-1" | "1/2-1/2" | "*")
}

/// One move as the code file and the game text agree to write it: no annotation suffixes, castling
/// spelled one way. Borrowed, because the classification pass calls this per half-move.
pub fn move_key(token: &str) -> &str {
    let token = token.trim().trim_end_matches(['!', '?', '+', '#']);
    match token {
        "0-0" => "O-O",
        "0-0-0" => "O-O-O",
        other => other,
    }
}

pub fn normalize_move(token: &str) -> String {
    move_key(token).to_owned()
}

/// The trie. One node per distinct move prefix; a node carries the code of the entry that ends
/// there, if any.
#[derive(Default)]
struct Node {
    children: HashMap<String, u32>,
    code: u16,
}

/// A built code file: the trie, and the code list the header record indexes into.
pub struct EcoTable {
    nodes: Vec<Node>,
    /// Distinct codes, sorted as text. The header record stores the position plus one; zero is
    /// `NO_ECO`.
    codes: Vec<String>,
}

#[derive(Debug)]
pub enum EcoError {
    TooManyCodes,
}

impl fmt::Display for EcoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a code file can hold at most {MAX_CODES} distinct codes")
    }
}

impl EcoTable {
    /// Build the trie. Codes are numbered by sorted text order, so the same file always gives the
    /// same ids.
    pub fn build(entries: &[EcoEntry]) -> Result<Self, EcoError> {
        let mut codes: Vec<String> = Vec::new();
        for entry in entries {
            if !codes.contains(&entry.code) {
                codes.push(entry.code.clone());
            }
        }
        codes.sort();
        if codes.len() > MAX_CODES {
            return Err(EcoError::TooManyCodes);
        }
        let ids: HashMap<String, u16> = codes
            .iter()
            .enumerate()
            .map(|(index, code)| (code.clone(), index as u16 + 1))
            .collect();
        let mut table = EcoTable { nodes: vec![Node::default()], codes };
        for entry in entries {
            let id = ids[&entry.code];
            let mut node = 0usize;
            for move_ in &entry.moves {
                let next = *table.nodes[node].children.get(move_).unwrap_or(&u32::MAX);
                if next == u32::MAX {
                    table.nodes.push(Node::default());
                    let last = table.nodes.len() as u32 - 1;
                    table.nodes[node].children.insert(move_.clone(), last);
                    node = last as usize;
                } else {
                    node = next as usize;
                }
            }
            table.nodes[node].code = id;
        }
        Ok(table)
    }

    pub fn from_text(text: &str) -> Result<(Self, Vec<EcoIssue>), EcoError> {
        let (entries, issues) = parse(text);
        Ok((Self::build(&entries)?, issues))
    }

    /// Every code this table can assign, in the order the header record numbers them.
    pub fn codes(&self) -> &[String] {
        &self.codes
    }

    pub fn code_of(&self, id: u16) -> Option<&str> {
        if id == NO_ECO { None } else { self.codes.get(id as usize - 1).map(String::as_str) }
    }

    pub fn id_of(&self, code: &str) -> Option<u16> {
        let code = normalize_code(code)?;
        self.codes.iter().position(|c| *c == code).map(|index| index as u16 + 1)
    }

    /// The code of the longest entry that is a prefix of these moves.
    pub fn classify(&self, moves: &[String]) -> u16 {
        let mut node = 0usize;
        let mut best = NO_ECO;
        for move_ in moves {
            match self.nodes[node].children.get(move_key(move_)) {
                Some(&next) => {
                    node = next as usize;
                    if self.nodes[node].code != NO_ECO {
                        best = self.nodes[node].code;
                    }
                }
                None => break,
            }
        }
        best
    }

    /// The code of a game's main line. Moves inside variations are not part of the sequence.
    pub fn classify_movetext(&self, movetext: &str) -> u16 {
        let mut node = 0usize;
        let mut best = NO_ECO;
        let mut depth = 0u32;
        for token in Lexer::new(movetext) {
            match token {
                Ok(Token::VariationStart) => depth += 1,
                Ok(Token::VariationEnd) => depth = depth.saturating_sub(1),
                Ok(Token::Result(_)) => break,
                Ok(Token::SanMove(raw)) if depth == 0 => {
                    let move_ = move_key(raw);
                    match self.nodes[node].children.get(move_) {
                        Some(&next) => {
                            node = next as usize;
                            if self.nodes[node].code != NO_ECO {
                                best = self.nodes[node].code;
                            }
                        }
                        None => return best, // the line has left the trie; nothing deeper can match
                    }
                }
                Err(_) => break,
                _ => {}
            }
        }
        best
    }
}

/// What one classification pass did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClassificationReport {
    pub games: usize,
    pub classified: usize,
    pub unclassified: usize,
    /// Games whose code changed, which is how a repeat pass proves it did nothing twice.
    pub changed: usize,
}

/// Write each game's code into its header record. Games that match nothing keep whatever they had,
/// which for a first pass is nothing.
pub fn classify_library(session: &mut Session, table: &EcoTable) -> Result<ClassificationReport, SessionError> {
    let mut report = ClassificationReport::default();
    for row in 0..session.len() {
        report.games += 1;
        let text = match session.game(row) {
            Some(bytes) => String::from_utf8_lossy(bytes).into_owned(),
            None => String::new(),
        };
        let code = table.classify_movetext(&text);
        if code != NO_ECO {
            report.classified += 1;
        } else {
            report.unclassified += 1;
        }
        if session.record(row).is_some_and(|record| record.eco != code) {
            report.changed += 1;
        }
        session.set_eco(row, code);
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header_record::HeaderRecord;
    use crate::session::{LibraryFiles, Session};
    use std::fs;

    const CODES: &str = "\
# a tiny code file
B20 1. e4 c5
B92 1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6
A00 b3
C20 e4 e5
";

    fn table() -> EcoTable {
        let (entries, issues) = parse(CODES);
        assert!(issues.is_empty(), "{issues:?}");
        EcoTable::build(&entries).unwrap()
    }

    #[test]
    fn the_longest_matching_entry_wins() {
        let table = table();
        let short = ["e4".to_string(), "c5".to_string()];
        assert_eq!(table.classify(&short), table.id_of("B20").unwrap());
        let long = ["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "a6"]
            .map(String::from)
            .to_vec();
        assert_eq!(table.classify(&long), table.id_of("B92").unwrap(), "10 half-moves beat 2");
        // The spec case: a 4 half-move entry under a 2 half-move entry.
        let (mut entries, _) = parse(CODES);
        entries.retain(|entry| entry.code != "B92");
        entries.push(EcoEntry {
            code: "B93".into(),
            moves: ["e4", "c5", "Nf3", "d6"].map(String::from).to_vec(),
        });
        let table = EcoTable::build(&entries).unwrap();
        assert_eq!(
            table.classify(["e4", "c5", "Nf3", "d6", "d4"].map(String::from).as_slice()),
            table.id_of("B93").unwrap(),
            "the 4 half-move entry, not the 2 half-move one"
        );
    }

    #[test]
    fn a_game_that_matches_nothing_gets_no_code() {
        let table = table();
        assert_eq!(table.classify(&[]), NO_ECO);
        assert_eq!(table.classify(&["d4".to_string()]), NO_ECO);
        assert_eq!(table.classify_movetext("1.d4 Nf6 2.c4 e5 3.dxe5"), NO_ECO);
    }

    #[test]
    fn move_numbers_annotations_and_castling_are_ignored() {
        let (entries, issues) = parse("C00 1. e4 e6 2. d4 d5 3. Nc6!? Bb4+\nD00 d4 d5 O-O\n");
        assert!(issues.is_empty(), "{issues:?}");
        let table = EcoTable::build(&entries).unwrap();
        assert_eq!(table.classify_movetext("1.e4! e6 2.d4 d5 3.Nc6 Bb4 4.Nf3 *"), table.id_of("C00").unwrap());
        assert_eq!(table.classify(&["d4".into(), "d5".into(), "0-0".into()]), table.id_of("D00").unwrap());
    }

    #[test]
    fn variations_are_not_part_of_the_sequence() {
        let (entries, _) = parse("B01 1. e4 d5 2. exd5\n");
        let table = EcoTable::build(&entries).unwrap();
        assert_eq!(table.classify_movetext("1.e4 d5 2.exd5 (2.Nf3 Nf6 3.exd5) 2...Nf6 *"), table.id_of("B01").unwrap());
        // A game whose main line diverges at move two does not match, even though the variation
        // contains the entry's moves.
        assert_eq!(table.classify_movetext("1.e4 d5 2.Nf3 (2.exd5) *"), NO_ECO);
    }

    #[test]
    fn malformed_lines_are_reported_by_line_and_the_rest_loads() {
        let (entries, issues) = parse("\
1. e4 e5            # no code
X 1. e4             # code too short
A01                 # no moves
A02 1. e4 c5        # fine
A0! 1. d4           # code is not a code
A03 1-0             # a result where a move should be
A01 1. e4 c5        # the same sequence and code again
");
        assert_eq!(entries.len(), 1, "only A02 loaded");
        assert_eq!(entries[0].code, "A02");
        let lines: Vec<usize> = issues.iter().map(|issue| issue.line).collect();
        assert_eq!(lines, vec![1, 2, 3, 5, 6, 7], "every bad line is named: {issues:?}");
    }

    #[test]
    fn a_duplicate_sequence_with_another_code_is_reported_and_the_first_wins() {
        let (entries, issues) = parse("A01 1. e4 e5\nA02 1. e4 e5\n");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].code, "A01");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line, 2);
    }

    #[test]
    fn codes_are_numbered_by_sorted_text_order_and_survive_a_rebuild() {
        let table = table();
        let mut ids: Vec<(u16, &str)> =
            table.codes().iter().enumerate().map(|(index, code)| (index as u16 + 1, code.as_str())).collect();
        assert_eq!(ids, vec![(1, "A00"), (2, "B20"), (3, "B92"), (4, "C20")]);
        ids.sort();
        for (id, code) in ids {
            assert_eq!(table.code_of(id), Some(code));
            assert_eq!(table.id_of(code), Some(id));
        }
        assert_eq!(table.code_of(NO_ECO), None);
    }

    fn library(dir: &std::path::Path, games: &[&str]) -> Session {
        let files = LibraryFiles::new(dir.join("games"));
        let mut session = Session::create(files).unwrap();
        for text in games {
            session.add_game(HeaderRecord::default(), text.as_bytes()).unwrap();
        }
        session
    }

    #[test]
    fn a_pass_writes_codes_and_counts_what_it_could_not_classify() {
        let dir = std::env::temp_dir().join(format!("spitz-eco-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut session = library(
            &dir,
            &[
                "1.e4 c5 2.Nf3 d6 3.d4 cxd4 4.Nxd4 Nf6 5.Nc3 a6 1-0",
                "1.e4 c5 2.Nf3 *",
                "1.b3 e5 2.Bb2 *",
                "1.d4 d5 2.c4 *",
                "1.e4 e5 2.Nf3 *",
            ],
        );
        let report = classify_library(&mut session, &table()).unwrap();
        assert_eq!(report.games, 5);
        assert_eq!(report.classified, 4, "B92, B20, A00 and C20");
        assert_eq!(report.unclassified, 1);
        assert_eq!(report.changed, 4);
        assert_eq!(session.record(0).unwrap().eco, table().id_of("B92").unwrap());
        assert_eq!(session.record(1).unwrap().eco, table().id_of("B20").unwrap());
        assert_eq!(session.record(2).unwrap().eco, table().id_of("A00").unwrap());
        assert_eq!(session.record(3).unwrap().eco, NO_ECO, "an unclassified game keeps an empty code");
        assert_eq!(session.record(4).unwrap().eco, table().id_of("C20").unwrap());

        // The same file again changes nothing.
        let report = classify_library(&mut session, &table()).unwrap();
        assert_eq!(report.changed, 0, "a repeat pass is deterministic");
        assert_eq!(report.unclassified, 1);
        drop(session);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_million_games_are_classified_inside_the_budget() {
        let dir = std::env::temp_dir().join(format!("spitz-eco-budget-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let files = LibraryFiles::new(dir.join("games"));
        let mut session = Session::create(files).unwrap();
        let lines = [
            "1.e4 c5 2.Nf3 d6 3.d4 cxd4 4.Nxd4 Nf6 5.Nc3 a6 6.Be2 e6 7.O-O Be7 1-0",
            "1.e4 c5 2.Nf3 Nc6 3.d4 cxd4 4.Nxd4 e5 5.Nb5 d6 6.N1c3 a6 7.Na3 b5 1-0",
            "1.d4 Nf6 2.c4 e6 3.Nf3 b6 4.g3 Ba6 5.b3 Bb4+ 6.Bd2 Be7 7.Bg2 c6 1-0",
            "1.c4 e5 2.Nc3 Nf6 3.g3 d5 4.cxd5 Nxd5 5.Bg2 Nb6 6.Nf3 Nc6 7.O-O Be7 1-0",
            "1.e3 e5 2.d3 d5 3.Nd2 Nf6 4.g3 Nc6 5.Bg2 Bc5 6.Ne2 O-O 7.O-O a6 1-0",
        ];
        for row in 0..1_000_000 {
            let text = lines[row % lines.len()];
            session.add_game(HeaderRecord::default(), text.as_bytes()).unwrap();
        }
        let (entries, issues) = parse(
            "B92 1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6\n\
             B56 1. e4 c5 2. Nf3 Nc6 3. d4 cxd4 4. Nxd4 e5 5. Nb5 d6\n\
             E15 1. d4 Nf6 2. c4 e6 3. Nf3 b6 4. g3 Ba6\n\
             A10 1. c4 e5 2. Nc3 Nf6 3. g3 d5 4. cxd5 Nxd5\n",
        );
        assert!(issues.is_empty(), "{issues:?}");
        let table = EcoTable::build(&entries).unwrap();

        let start = std::time::Instant::now();
        let report = classify_library(&mut session, &table).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(report.games, 1_000_000);
        assert_eq!(report.classified, 800_000, "four of the five lines are in the file");
        assert_eq!(report.unclassified, 200_000);
        for row in 0..1_000_000 {
            let expected = match row % lines.len() {
                0 => table.id_of("B92").unwrap(),
                1 => table.id_of("B56").unwrap(),
                2 => table.id_of("E15").unwrap(),
                3 => table.id_of("A10").unwrap(),
                _ => NO_ECO,
            };
            assert_eq!(session.record(row).unwrap().eco, expected, "row {row}");
        }
        println!("classified {} games in {elapsed:?}", report.games);
        #[cfg(not(debug_assertions))]
        assert!(elapsed.as_secs() < 600, "the budget is ten minutes, this took {elapsed:?}");
        session.discard();
        let _ = fs::remove_dir_all(&dir);
    }
}
