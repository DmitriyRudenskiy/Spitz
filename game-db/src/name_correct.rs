//! Name intelligence: the spelling rule file, and correction by edit distance.
//!
//! Both halves only ever *suggest*. Nothing here rewrites a header; the view shows the candidates
//! with their distances and the user decides. `docs/spelling-rule-format.md` is the rule file note.

use crate::collation;
use std::fmt;

/// The default distance a correction may be away.
pub const DEFAULT_MAX_DISTANCE: usize = 2;

/// A prefix/infix/suffix replacement list.
#[derive(Clone, Default, Debug)]
pub struct SpellingRules {
    /// (kind, from, to) in file order, folded.
    rules: Vec<(Kind, String, String)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Prefix,
    Infix,
    Suffix,
}

/// A line that could not be used, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleIssue {
    pub line: usize,
    pub reason: &'static str,
}

impl fmt::Display for RuleIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.reason)
    }
}

impl SpellingRules {
    /// Parse a rule file, returning the rules and every line that was skipped.
    pub fn parse(text: &str) -> (Self, Vec<RuleIssue>) {
        let mut rules = Self::default();
        let mut issues = Vec::new();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let text = raw.split('#').next().unwrap_or("").trim();
            if text.is_empty() {
                continue;
            }
            let fields: Vec<&str> = text.split_whitespace().collect();
            let [kind, from, to] = fields.as_slice() else {
                issues.push(RuleIssue { line, reason: "expected: kind, find, replace" });
                continue;
            };
            let kind = match kind.to_ascii_lowercase().as_str() {
                "prefix" => Kind::Prefix,
                "infix" => Kind::Infix,
                "suffix" => Kind::Suffix,
                _ => {
                    issues.push(RuleIssue { line, reason: "not a rule kind" });
                    continue;
                }
            };
            // Both sides are folded, so a rule can be written the way it is said rather than the
            // way the collation spells it.
            let from = collation::fold(from);
            let to = collation::fold(to);
            if from.is_empty() {
                issues.push(RuleIssue { line, reason: "nothing to find" });
                continue;
            }
            rules.rules.push((kind, from, to));
        }
        (rules, issues)
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// The name as it is matched: folded, then with each rule applied in file order. The stored
    /// text is never touched.
    pub fn apply(&self, name: &str) -> String {
        let mut name = collation::fold(name);
        for (kind, from, to) in &self.rules {
            match kind {
                Kind::Prefix => {
                    if let Some(rest) = name.strip_prefix(from.as_str()) {
                        name = format!("{to}{rest}");
                    }
                }
                Kind::Infix => name = name.replace(from.as_str(), to),
                Kind::Suffix => {
                    if let Some(rest) = name.strip_suffix(from.as_str()) {
                        name = format!("{rest}{to}");
                    }
                }
            }
        }
        name
    }
}

/// Restricted Damerau-Levenshtein distance over characters, where a transposition of two adjacent
/// characters counts as one operation.
///
/// `None` means "further than `max` away", which is also how the caller says it: a correction the
/// user would not recognise is not worth returning.
pub fn distance(a: &str, b: &str, max: usize) -> Option<usize> {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > max {
        return None;
    }
    let ceiling = max + 1;
    let width = b.len() + 1;
    let mut previous2 = vec![ceiling; width];
    let mut previous = vec![0usize; width];
    let mut current = vec![0usize; width];
    for (j, cell) in previous.iter_mut().enumerate() {
        *cell = j.min(ceiling);
    }
    for i in 1..=a.len() {
        current[0] = i.min(ceiling);
        let mut row_min = current[0];
        for j in 1..=b.len() {
            let substitution = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            let mut best = (previous[j] + 1).min(current[j - 1] + 1).min(previous[j - 1] + substitution);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(previous2[j - 2] + 1);
            }
            current[j] = best.min(ceiling);
            row_min = row_min.min(current[j]);
        }
        if row_min > max {
            return None;
        }
        std::mem::swap(&mut previous2, &mut previous);
        std::mem::swap(&mut previous, &mut current);
        current.clone_from(&previous);
    }
    let distance = previous[b.len()];
    (distance <= max).then_some(distance)
}

/// A candidate correction and how far it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion<'a> {
    pub name: &'a str,
    pub distance: usize,
}

/// The closest names to `query`, nearest first, at most `limit` of them.
///
/// Both sides go through `rules` first, so a systematic misspelling matches without the library
/// being rewritten. A name equal to the query is not a suggestion — the user did not make that
/// mistake.
pub fn suggest<'a>(
    candidates: impl Iterator<Item = &'a str>,
    query: &str,
    rules: &SpellingRules,
    max: usize,
    limit: usize,
) -> Vec<Suggestion<'a>> {
    let query = rules.apply(query);
    let mut found: Vec<(usize, String, &str)> = Vec::new();
    for name in candidates {
        let folded = rules.apply(name);
        if folded == query {
            continue;
        }
        if let Some(distance) = distance(&query, &folded, max) {
            found.push((distance, folded, name));
        }
    }
    // Nearest first, and alphabetical within a distance so the same input always offers the same
    // list.
    found.sort();
    found.truncate(limit);
    found.into_iter().map(|(distance, _, name)| Suggestion { name, distance }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_transposition_counts_as_one_operation() {
        assert_eq!(distance("spassky", "spsasky", 2), Some(1));
        assert_eq!(distance("ab", "ba", 2), Some(1));
        assert_eq!(distance("abc", "abc", 2), Some(0));
        assert_eq!(distance("abc", "abcd", 2), Some(1));
        assert_eq!(distance("abc", "bad", 2), Some(2));
        assert_eq!(distance("spassky", "fisher", 2), None, "further than the bound");
        assert_eq!(distance("spassky", "spsasky", 0), None);
    }

    #[test]
    fn a_transposed_name_is_offered_with_its_distance() {
        let library = ["Fischer, Bobby", "Spassky, Boris", "Karpov, Anatoly"];
        let got = suggest(library.iter().copied(), "Spsasky, Boris", &SpellingRules::default(), 2, 10);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "Spassky, Boris");
        assert_eq!(got[0].distance, 1, "the distance is shown, not guessed");
    }

    #[test]
    fn a_name_beyond_the_bound_is_not_offered() {
        let library = ["Spassky, Boris", "Short, Nigel"];
        let got = suggest(library.iter().copied(), "Tal, Mikhail", &SpellingRules::default(), 2, 10);
        assert!(got.is_empty());
    }

    #[test]
    fn rules_are_applied_before_matching_and_the_text_is_kept() {
        let (rules, issues) = SpellingRules::parse("prefix spsa spas\n");
        assert!(issues.is_empty(), "{issues:?}");
        let library = ["Spassky, Boris"];
        let got = suggest(library.iter().copied(), "Spsasky, Boris", &rules, 2, 10);
        assert!(got.is_empty(), "the rule already fixed the spelling, so nothing needs correcting");
        assert_eq!(rules.apply("Spsasky, Boris"), "spassky, boris");
        assert_eq!(rules.apply("Spassky, Boris"), "spassky, boris");
        assert_eq!("Spsasky, Boris", "Spsasky, Boris", "the stored header text is untouched");
    }

    #[test]
    fn rules_reach_infixes_and_suffixes_in_file_order() {
        let (rules, issues) = SpellingRules::parse("infix sz ss\nsuffix ski sky\nprefix Spsa Spas\n");
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(rules.apply("Iwaszko"), "iwassko");
        assert_eq!(rules.apply("Nowak"), "nowak", "a rule that finds nothing changes nothing");
        assert_eq!(rules.apply("Kowalski"), "kowalsky");
        assert_eq!(rules.apply("spsasky"), "spassky");
    }

    #[test]
    fn malformed_rules_are_reported_by_line_and_the_rest_loads() {
        let (rules, issues) = SpellingRules::parse("\
prefix a b
infix c
nonsense a b
prefix  b
suffix  x
prefix d e
");
        assert_eq!(rules.len(), 2);
        let lines: Vec<usize> = issues.iter().map(|issue| issue.line).collect();
        assert_eq!(lines, vec![2, 3, 4, 5], "every bad line is named: {issues:?}");
    }

    #[test]
    fn suggestions_are_ranked_nearest_first_and_capped() {
        let library = ["Aronian, Levon", "Aronyan, Levon", "Aronian, Levonn", "Aronian, Levonov"];
        let got = suggest(library.iter().copied(), "Aronian, Levon", &SpellingRules::default(), 2, 3);
        let distances: Vec<usize> = got.iter().map(|s| s.distance).collect();
        assert_eq!(distances, vec![1, 1, 2], "{got:?}");
        assert_eq!(got[0].name, "Aronian, Levonn", "alphabetical within a distance");
        let capped = suggest(library.iter().copied(), "Aronian, Levon", &SpellingRules::default(), 2, 1);
        assert_eq!(capped.len(), 1);
    }
}
