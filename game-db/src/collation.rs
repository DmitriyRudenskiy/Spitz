//! The one collation the whole app uses: name comparison, round comparison, and the folded
//! keys the completion index is built on.
//!
//! The rules come from the reference suite's documented behaviour — diacritic-insensitive,
//! case-insensitive, leading integers compared numerically, and completion matching whole
//! word-initial chunks — re-expressed here (see `NOTICE.md`).

use crate::fold_data::FOLDS;
use std::cmp::Ordering;

/// Fold one letter: strip its diacritics and lower it. Returns `None` for letters the table
/// does not hold, which are then used as they are.
fn fold_letter(c: char) -> Option<&'static str> {
    FOLDS
        .binary_search_by_key(&u32::from(c), |&(letter, _)| u32::from(letter))
        .ok()
        .map(|i| FOLDS[i].1)
}

/// Fold a name for comparison and completion: diacritic-free, lower case, and with runs of
/// whitespace collapsed to a single space. Non-Latin letters pass through.
pub fn fold(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut in_space = false;
    for c in name.chars() {
        if c.is_whitespace() {
            in_space = true;
            continue;
        }
        if in_space && !out.is_empty() {
            out.push(' ');
        }
        in_space = false;
        match fold_letter(c) {
            Some(folded) => out.push_str(folded),
            None => out.extend(c.to_lowercase()),
        }
    }
    out
}

/// Compare two names the way a name list sorts them: folded, with runs of digits compared as
/// numbers, then by the original so the order is total.
///
/// The digit rule is what keeps `Round 3` before `Round 12` and `Tata Steel 2024` next to
/// `Tata Steel 2025` instead of in a column of its own.
pub fn compare(a: &str, b: &str) -> Ordering {
    compare_digits(&fold(a), &fold(b)).then_with(|| a.cmp(b))
}

/// Compare two already-folded strings, treating each run of digits as a number.
fn compare_digits(a: &str, b: &str) -> Ordering {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i].is_ascii_digit() && b[j].is_ascii_digit() {
            let a_run = digit_run(a, i);
            let b_run = digit_run(b, j);
            match compare_number(a_run.0, b_run.0) {
                Ordering::Equal => {
                    i += a_run.1;
                    j += b_run.1;
                }
                other => return other,
            }
        } else {
            match a[i].cmp(&b[j]) {
                Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
                other => return other,
            }
        }
    }
    (a.len() - i).cmp(&(b.len() - j))
}

/// The digit run starting at `at`, as `(digits without leading zeros, length consumed)`.
fn digit_run(s: &[u8], at: usize) -> (&[u8], usize) {
    let mut end = at;
    while end < s.len() && s[end].is_ascii_digit() {
        end += 1;
    }
    let mut start = at;
    while start < end - 1 && s[start] == b'0' {
        start += 1;
    }
    (&s[start..end], end - at)
}

fn compare_number(a: &[u8], b: &[u8]) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// Compare two round names: a leading integer is a number, and a number sorts before the
/// non-number that follows it.
///
/// So `10` comes after `9`, and `1` comes before `10` — which is what a round list of
/// `1, 2, …, 10, 11` needs. Rounds named `A`, `B`, `Final` sort as text.
pub fn compare_round(a: &str, b: &str) -> Ordering {
    let (a_number, a_rest) = split_leading_int(a);
    let (b_number, b_rest) = split_leading_int(b);
    match (a_number, b_number) {
        (Some(x), Some(y)) => x.cmp(&y).then_with(|| compare(a_rest, b_rest)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => compare(a, b),
    }
}

/// Split off a leading run of digits as a number. A run too long for `u64` counts as no number.
fn split_leading_int(name: &str) -> (Option<u64>, &str) {
    let trimmed = name.trim_start();
    let end = trimmed.find(|c: char| !c.is_ascii_digit()).unwrap_or(trimmed.len());
    match trimmed[..end].parse::<u64>() {
        Ok(value) if end > 0 => (Some(value), &trimmed[end..]),
        _ => (None, name),
    }
}

/// The rotation key of a name: its last word moved to the front, which is the order a name list
/// is searched in ("Ivanov, Peter" for "Peter Ivanov"). A name already written that way — one
/// containing a comma — is left alone.
pub fn rotation_key(name: &str) -> String {
    if name.contains(',') {
        return name.to_owned();
    }
    let words: Vec<&str> = name.split_whitespace().collect();
    match words.len() {
        0 => String::new(),
        1 => words[0].to_owned(),
        _ => format!("{}, {}", words[words.len() - 1], words[..words.len() - 1].join(" ")),
    }
}

/// The key a completion is matched against: folded, with the spaces between words removed. The
/// words themselves still bound where a match may start, so the spaces are information, not
/// noise — they are just not typed.
pub fn fold_key(name: &str) -> String {
    fold(name).replace(' ', "")
}

/// The words of a folded name.
pub fn words(name: &str) -> Vec<String> {
    fold(name).split(' ').map(str::to_owned).collect()
}

/// Can `key` be read as successive prefixes of `word`?
///
/// Each word consumes a whole prefix of itself, possibly nothing, and the key must be consumed
/// exactly. That is what makes `piv` match "Peter Ivanov" (`p` + `iv`) and `peteri` match it too
/// (`peter` + `i`), while `pivanov` fails: the `i` of the second word is not a prefix of it.
pub fn matches_words(key: &str, words: &[String]) -> bool {
    if key.is_empty() {
        return true;
    }
    let Some((first, rest)) = words.split_first() else {
        return false;
    };
    let limit = first.len().min(key.len());
    (0..=limit).rev().any(|taken| {
        key.get(..taken) == Some(&first[..taken]) && matches_words(&key[taken..], rest)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding_strips_diacritics_and_case() {
        assert_eq!(fold("Müller"), "muller");
        assert_eq!(fold("ÁCS"), "acs");
        assert_eq!(fold("Erdős"), "erdos");
        assert_eq!(fold("Đukić"), "dukic");
        assert_eq!(fold("Ķekšavihs"), "keksavihs");
        assert_eq!(fold("Sigurðsson"), "sigurdsson");
        assert_eq!(fold("Ałeksandrow"), "aleksandrow");
        assert_eq!(fold("Groenhoff"), "groenhoff");
    }

    #[test]
    fn folding_collapses_whitespace_and_keeps_other_scripts() {
        assert_eq!(fold("  Peter\tIvanov  "), "peter ivanov");
        assert_eq!(fold("Карпов, Анатолий"), "карпов, анатолий");
        assert_eq!(fold("Anand"), "anand");
    }

    #[test]
    fn names_compare_folded_then_exactly() {
        assert_eq!(compare("Müller", "muller"), Ordering::Less);
        assert_eq!(compare("Müller", "Naiditsch"), Ordering::Less);
        assert_eq!(compare("A", "A"), Ordering::Equal);
        let mut names = vec!["Zlatić", "Åberg", "Müller", "Aberg"];
        names.sort_by(|a, b| compare(a, b));
        assert_eq!(names, vec!["Aberg", "Åberg", "Müller", "Zlatić"]);
    }

    #[test]
    fn rounds_compare_their_leading_integer_numerically() {
        let mut rounds = vec!["10", "9", "1", "11", "2", "Final", "A", "1a", "1b"];
        rounds.sort_by(|a, b| compare_round(a, b));
        assert_eq!(rounds, vec!["1", "1a", "1b", "2", "9", "10", "11", "A", "Final"]);
        assert_eq!(compare_round("10", "9"), Ordering::Greater);
        assert_eq!(compare_round("Round 3", "Round 12"), Ordering::Less);
    }

    #[test]
    fn rotation_moves_the_last_word_first() {
        assert_eq!(rotation_key("Peter Ivanov"), "Ivanov, Peter");
        assert_eq!(rotation_key("Ivanov, Peter"), "Ivanov, Peter");
        assert_eq!(rotation_key("Anand"), "Anand");
        assert_eq!(rotation_key("  "), "");
    }

    #[test]
    fn a_key_is_word_initial_chunks_of_the_name() {
        let name = words("Peter Ivanov");
        for key in ["p", "pi", "piv", "peteri", "ivan", "pet", "iv", "peterivanov", "pivanov", ""] {
            assert!(matches_words(key, &name), "{key} should match Peter Ivanov");
        }
        for key in ["pivx", "pivz", "xyz", "peterivanovs", "ivanovp"] {
            assert!(!matches_words(key, &name), "{key} should not match Peter Ivanov");
        }
    }
}
