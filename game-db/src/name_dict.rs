//! The interned name dictionary: header names as `u32` ids, one table per kind.
//!
//! Names are stored once and referenced by id from every header record, which is what keeps the
//! record 48 bytes and makes a name list free. Ids are stable while a name is in use; a compaction
//! hands back the remapping so the caller can rewrite the records that hold it.

use crate::collation::{fold, fold_key, matches_words, rotation_key, words};
use std::collections::HashMap;
use std::fmt;

/// Version byte at the head of a name file. A bump means rebuild, not translate.
pub const NAME_FILE_VERSION: u8 = 1;

/// The reserved id: no name. Every kind has one, at the same place, so an unset field is the same
/// bytes everywhere.
pub const NO_NAME: u32 = 0;

pub type NameId = u32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NameKind {
    Player,
    Event,
    Site,
    Round,
}

impl NameKind {
    const fn code(self) -> u8 {
        match self {
            NameKind::Player => 1,
            NameKind::Event => 2,
            NameKind::Site => 3,
            NameKind::Round => 4,
        }
    }

    const fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            1 => NameKind::Player,
            2 => NameKind::Event,
            3 => NameKind::Site,
            4 => NameKind::Round,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
struct Entry {
    name: String,
    /// How many games reference this name. Zero makes it collectable by [`compact`].
    uses: u32,
}

/// A dictionary of one kind. Ids are indexes into `entries`; id 0 is always the empty name.
#[derive(Clone, Debug)]
pub struct NameDictionary {
    kind: NameKind,
    entries: Vec<Entry>,
    ids: HashMap<String, NameId>,
    /// Built on first completion and dropped by any mutation.
    index: Option<PrefixIndex>,
}

impl NameDictionary {
    pub fn new(kind: NameKind) -> Self {
        Self {
            kind,
            entries: vec![Entry { name: String::new(), uses: u32::MAX }],
            ids: HashMap::new(),
            index: None,
        }
    }

    pub fn kind(&self) -> NameKind {
        self.kind
    }

    /// Names held, counting the reserved empty name.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.len() == 1
    }

    /// The id of `name`, adding it if new. Each call counts one more user of the name.
    pub fn intern(&mut self, name: &str) -> NameId {
        if let Some(&id) = self.ids.get(name) {
            self.entries[id as usize].uses = self.entries[id as usize].uses.saturating_add(1);
            return id;
        }
        let id = self.entries.len() as NameId;
        self.entries.push(Entry { name: name.to_owned(), uses: 1 });
        self.ids.insert(name.to_owned(), id);
        self.index = None;
        id
    }

    /// The id of `name` if the dictionary already holds it, without counting a user.
    pub fn find(&self, name: &str) -> Option<NameId> {
        self.ids.get(name).copied()
    }

    pub fn name(&self, id: NameId) -> Option<&str> {
        self.entries.get(id as usize).map(|e| e.name.as_str())
    }

    pub fn uses(&self, id: NameId) -> u32 {
        self.entries.get(id as usize).map(|e| e.uses).unwrap_or(0)
    }

    /// One fewer user. Returns the count left, so a caller can assert it did not go negative.
    pub fn release(&mut self, id: NameId) -> u32 {
        let Some(entry) = self.entries.get_mut(id as usize) else { return 0 };
        if id == NO_NAME {
            return entry.uses;
        }
        entry.uses = entry.uses.saturating_sub(1);
        entry.uses
    }

    /// Drop every name with no users and hand back how the surviving ids moved.
    pub fn compact(&mut self) -> IdRemap {
        let mut remap = vec![NO_NAME; self.entries.len()];
        let mut kept = Vec::with_capacity(self.entries.len());
        kept.push(Entry { name: String::new(), uses: u32::MAX });
        remap[0] = 0;
        for (old, entry) in self.entries.iter().enumerate().skip(1) {
            if entry.uses > 0 {
                remap[old] = kept.len() as NameId;
                kept.push(entry.clone());
            }
        }
        self.entries = kept;
        self.ids = self
            .entries
            .iter()
            .enumerate()
            .skip(1)
            .map(|(id, e)| (e.name.clone(), id as NameId))
            .collect();
        self.index = None;
        IdRemap { map: remap }
    }

    /// Build the completion index now instead of on the first keystroke. A library loader calls
    /// this once, off the UI thread; [`Self::complete`] still builds it lazily if nobody did.
    pub fn warm_completion_index(&mut self) {
        if self.index.is_none() {
            self.index = Some(PrefixIndex::build(&self.entries));
        }
    }

    /// Up to `limit` names matching `query`, best first. See [`PrefixIndex`].
    pub fn complete(&mut self, query: &str, limit: usize) -> Vec<String> {
        let _limit = limit.min(10);
        let index = self.index.get_or_insert_with(|| PrefixIndex::build(&self.entries));
        let ids = index.complete(query, _limit);
        ids.iter().filter_map(|&id| self.name(id)).map(str::to_owned).collect()
    }

    /// The stored form: version byte, kind, then each name with its user count.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(6 + self.entries.len() * 24);
        out.push(NAME_FILE_VERSION);
        out.push(self.kind.code());
        out.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        for entry in &self.entries {
            out.extend_from_slice(&entry.uses.to_le_bytes());
            out.extend_from_slice(&(entry.name.len() as u32).to_le_bytes());
            out.extend_from_slice(entry.name.as_bytes());
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DictError> {
        let [version, kind, rest @ ..] = bytes else { return Err(DictError::TooShort) };
        if *version != NAME_FILE_VERSION {
            return Err(DictError::Version(*version));
        }
        let kind = NameKind::from_code(*kind).ok_or(DictError::Kind(*kind))?;
        if rest.len() < 4 {
            return Err(DictError::TooShort);
        }
        let count = u32::from_le_bytes(rest[..4].try_into().unwrap()) as usize;
        let mut rest = &rest[4..];
        let mut entries = Vec::with_capacity(count.min(1_000_000));
        for _ in 0..count {
            if rest.len() < 8 {
                return Err(DictError::TooShort);
            }
            let uses = u32::from_le_bytes(rest[..4].try_into().unwrap());
            let len = u32::from_le_bytes(rest[4..8].try_into().unwrap()) as usize;
            if rest.len() < 8 + len {
                return Err(DictError::TooShort);
            }
            let name = std::str::from_utf8(&rest[8..8 + len]).map_err(|_| DictError::NotUtf8)?;
            entries.push(Entry { name: name.to_owned(), uses });
            rest = &rest[8 + len..];
        }
        if entries.first().is_none_or(|e| !e.name.is_empty()) {
            return Err(DictError::MissingEmptyName);
        }
        let ids = entries
            .iter()
            .enumerate()
            .skip(1)
            .map(|(id, e)| (e.name.clone(), id as NameId))
            .collect();
        Ok(Self { kind, entries, ids, index: None })
    }
}

/// How ids moved during [`NameDictionary::compact`]. Ids absent from the map were dropped.
#[derive(Clone, Debug, Default)]
pub struct IdRemap {
    map: Vec<NameId>,
}

impl IdRemap {
    /// The new id of `old`, or [`NO_NAME`] when the name was collected.
    pub fn get(&self, old: NameId) -> NameId {
        self.map.get(old as usize).copied().unwrap_or(NO_NAME)
    }

    /// Ids that did not move, so a caller can skip rewriting whole files.
    pub fn unchanged(&self) -> bool {
        self.map.iter().enumerate().all(|(old, &new)| new == old as NameId)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DictError {
    TooShort,
    Version(u8),
    Kind(u8),
    NotUtf8,
    /// A file whose id 0 is not the empty name is not one of ours.
    MissingEmptyName,
}

impl fmt::Display for DictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DictError::TooShort => write!(f, "name file is truncated"),
            DictError::Version(v) => write!(f, "name file version {v}, expected {NAME_FILE_VERSION}"),
            DictError::Kind(k) => write!(f, "unknown name kind {k}"),
            DictError::NotUtf8 => write!(f, "name file holds invalid utf-8"),
            DictError::MissingEmptyName => write!(f, "name file id 0 is not the empty name"),
        }
    }
}

impl std::error::Error for DictError {}

/// One searchable form of one name.
#[derive(Clone, Debug)]
struct Candidate {
    /// Folded form of the name, in one of the two word orders.
    key: String,
    id: NameId,
    /// Whether this is the name's own word order or the rotated one.
    rotated: bool,
}

/// The completion index: every name's folded key in both word orders, sorted for a prefix
/// search, plus a map from a name's word-initial letters to the names carrying them.
///
/// A plain prefix of a name ("karp" for Karpov) is a binary search. A key made of word-initial
/// chunks ("piv" for Peter Ivanov) is not a prefix of anything, so it is looked up the other way
/// round: the key's own subsequences are the only word-initial strings that could match it, and
/// there are few of them. The longest are tried first, because they name the most specific names.
///
/// ponytail: the index costs about a second to build for a million names and is rebuilt from
/// scratch after any mutation. When that shows up in a profile, build it in the background on
/// load and patch it incrementally; the query API does not change.
#[derive(Clone, Debug, Default)]
pub struct PrefixIndex {
    candidates: Vec<Candidate>,
    /// Word-initial letters ("pi") to the candidates whose words start that way.
    by_word_initial: HashMap<String, Vec<u32>>,
}

impl PrefixIndex {
    fn build(entries: &[Entry]) -> Self {
        let mut candidates = Vec::with_capacity(entries.len() * 2);
        for (id, entry) in entries.iter().enumerate().skip(1) {
            let natural = fold(&entry.name);
            let rotated = fold(&rotation_key(&entry.name));
            for (rotated_form, key) in [(false, natural), (true, rotated)] {
                candidates.push(Candidate { id: id as NameId, key, rotated: rotated_form });
            }
        }
        candidates.sort_by(|a, b| a.key.cmp(&b.key).then(a.id.cmp(&b.id)));
        // Built after the sort, so the positions it holds are the final ones.
        let mut by_word_initial: HashMap<String, Vec<u32>> = HashMap::with_capacity(1024);
        for (at, candidate) in candidates.iter().enumerate() {
            by_word_initial.entry(word_initials(&candidate.key)).or_default().push(at as u32);
        }
        Self { candidates, by_word_initial }
    }

    /// Ranked ids matching `query`: exact prefix first, then the other word order, then the keys
    /// made of word-initial chunks. Never more than `limit`, and never more than ten.
    pub fn complete(&self, query: &str, limit: usize) -> Vec<NameId> {
        let limit = limit.min(10);
        let key = fold_key(query);
        if key.is_empty() {
            return Vec::new();
        }
        let mut found: Vec<(u8, String, NameId)> = Vec::new();
        let mut seen: Vec<NameId> = Vec::with_capacity(limit);
        let start = self
            .candidates
            .partition_point(|c| c.key.as_str() < key.as_str());
        for c in self.candidates[start..].iter().take_while(|c| c.key.starts_with(&key)) {
            let rank = if c.rotated { 1 } else { 0 };
            if !seen.contains(&c.id) {
                seen.push(c.id);
                found.push((rank, c.key.clone(), c.id));
                if found.len() == limit {
                    return found.into_iter().map(|(_, _, id)| id).collect();
                }
            }
        }
        // The prefix pass came up short. A key longer than this is a prefix search in practice,
        // and its subsequence set would be larger than the answer is worth.
        if key.len() > MAX_CHUNK_KEY_LEN {
            return seen;
        }
        // A keystroke has a budget, so the chunk pass gives up after this many candidates rather
        // than scanning a bucket of a million names that all share the key's word initials.
        let mut tested = 0;
        'chunks: for subsequence in subsequences_longest_first(&key) {
            let Some(indices) = self.by_word_initial.get(&subsequence) else { continue };
            for &at in indices {
                tested += 1;
                if tested > MAX_CHUNK_CANDIDATES {
                    break 'chunks;
                }
                let candidate = &self.candidates[at as usize];
                if seen.contains(&candidate.id) || !matches_words(&key, &words(&candidate.key)) {
                    continue;
                }
                seen.push(candidate.id);
                found.push((2, candidate.key.clone(), candidate.id));
                if found.len() == limit {
                    break 'chunks;
                }
            }
        }
        found.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        found.into_iter().map(|(_, _, id)| id).collect()
    }
}

/// Beyond this, only the prefix pass runs.
const MAX_CHUNK_KEY_LEN: usize = 12;
/// Candidates the word-chunk pass will test before giving up, which keeps a keystroke inside its
/// budget whatever the dictionary holds.
const MAX_CHUNK_CANDIDATES: usize = 20_000;

/// The word-initial letters of a folded key, as text: "peter ivanov" to "pi".
fn word_initials(key: &str) -> String {
    key.split([' ', ','])
        .filter_map(|word| word.chars().next())
        .filter(|c| c.is_ascii_alphanumeric())
        .take(8)
        .collect()
}

/// Every subsequence of `key`, longest first, dropping duplicates.
fn subsequences_longest_first(key: &str) -> Vec<String> {
    let bytes = key.as_bytes();
    let mut out: Vec<(u8, String)> = Vec::with_capacity(1 << bytes.len().min(12));
    for mask in 1u32..1u32 << bytes.len() {
        let mut word = String::with_capacity(bytes.len());
        for (at, byte) in bytes.iter().enumerate() {
            if mask >> at & 1 == 1 {
                word.push(*byte as char);
            }
        }
        out.push((word.len() as u8, word));
    }
    out.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    out.dedup_by(|a, b| a.1 == b.1);
    out.into_iter().map(|(_, word)| word).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dictionary() -> NameDictionary {
        let mut dict = NameDictionary::new(NameKind::Player);
        for name in ["Ivanov, Peter", "Karpov, Anatoly", "Peter Svidler", "Müller, Thomas"] {
            dict.intern(name);
        }
        dict
    }

    #[test]
    fn interning_is_idempotent_and_counts_users() {
        let mut dict = NameDictionary::new(NameKind::Player);
        let first = dict.intern("Anand");
        let second = dict.intern("Anand");
        assert_eq!(first, second);
        assert_eq!(dict.uses(first), 2);
        assert_eq!(dict.len(), 2, "only the empty name and Anand");
        assert_eq!(dict.name(first), Some("Anand"));
        assert_eq!(dict.name(NO_NAME), Some(""));
        assert_eq!(dict.find("Nobody"), None);
    }

    #[test]
    fn ids_are_stable_until_a_compaction_moves_them() {
        let mut dict = dictionary();
        let (ivanov, karpov, svidler, muller) = (
            dict.find("Ivanov, Peter").unwrap(),
            dict.find("Karpov, Anatoly").unwrap(),
            dict.find("Peter Svidler").unwrap(),
            dict.find("Müller, Thomas").unwrap(),
        );
        dict.release(karpov);
        dict.release(muller);
        let remap = dict.compact();
        assert_eq!(dict.name(remap.get(ivanov)), Some("Ivanov, Peter"));
        assert_eq!(dict.name(remap.get(svidler)), Some("Peter Svidler"));
        assert_eq!(remap.get(karpov), NO_NAME, "a name with no users is gone");
        assert_eq!(dict.find("Karpov, Anatoly"), None);
        assert_eq!(dict.intern("Karpov, Anatoly"), 3, "the freed id is reused");
        assert_eq!(dict.intern("Müller, Thomas"), 4);
    }

    #[test]
    fn a_compaction_that_removes_nothing_leaves_ids_alone() {
        let mut dict = dictionary();
        let remap = dict.compact();
        assert!(remap.unchanged());
        assert_eq!(remap.get(3), 3);
        assert_eq!(remap.get(9999), NO_NAME);
    }

    #[test]
    fn a_dictionary_survives_its_stored_form() {
        let mut dict = dictionary();
        dict.release(dict.find("Peter Svidler").unwrap());
        let bytes = dict.encode();
        let back = NameDictionary::decode(&bytes).unwrap();
        assert_eq!(back.kind(), NameKind::Player);
        assert_eq!(back.len(), dict.len());
        for id in 0..back.len() as u32 {
            assert_eq!(back.name(id), dict.name(id));
            assert_eq!(back.uses(id), dict.uses(id));
        }
    }

    #[test]
    fn a_foreign_or_truncated_file_is_refused() {
        let dict = dictionary();
        let bytes = dict.encode();
        assert!(matches!(NameDictionary::decode(&bytes[..5]), Err(DictError::TooShort)));
        let mut wrong_version = bytes.clone();
        wrong_version[0] = 9;
        assert!(matches!(NameDictionary::decode(&wrong_version), Err(DictError::Version(9))));
        let mut wrong_kind = bytes.clone();
        wrong_kind[1] = 7;
        assert!(matches!(NameDictionary::decode(&wrong_kind), Err(DictError::Kind(7))));
        assert!(matches!(NameDictionary::decode(&[]), Err(DictError::TooShort)));
    }

    #[test]
    fn completion_ignores_case_diacritics_and_word_order() {
        let mut dict = dictionary();
        assert!(dict.complete("piv", 10).contains(&"Ivanov, Peter".to_owned()));
        assert!(!dict.complete("piv", 10).contains(&"Peter Svidler".to_owned()));
        assert_eq!(dict.complete("muller", 10), vec!["Müller, Thomas".to_owned()]);
        assert_eq!(dict.complete("MÜLLER", 10), vec!["Müller, Thomas".to_owned()]);
        assert_eq!(dict.complete("karp", 10), vec!["Karpov, Anatoly".to_owned()]);
        assert!(dict.complete("nobody", 10).is_empty());
        assert!(dict.complete("", 10).is_empty());
    }

    #[test]
    fn completion_prefers_the_strongest_prefix_and_honours_the_limit() {
        let mut dict = NameDictionary::new(NameKind::Player);
        for name in ["Peter Ivanov", "Ivanov, Peter", "Ivan Svidler", "Ivanov, Sergei"] {
            dict.intern(name);
        }
        let best = dict.complete("ivan", 10);
        assert_eq!(best[0], "Ivan Svidler".to_owned(), "a natural-order prefix ranks first");
        assert!(dict.complete("ivanov", 10).contains(&"Peter Ivanov".to_owned()));
        let capped = dict.complete("iva", 2);
        assert_eq!(capped.len(), 2);
        // Both stored forms of the name match, and each is returned once.
        assert_eq!(dict.complete("ivanov p", 10), vec!["Peter Ivanov".to_owned(), "Ivanov, Peter".to_owned()]);
        let mut once = NameDictionary::new(NameKind::Player);
        once.intern("Peter Ivanov");
        assert_eq!(once.complete("ivanov p", 10), vec!["Peter Ivanov".to_owned()]);
    }

    #[test]
    fn completion_stays_inside_its_budget_on_a_large_dictionary() {
        let mut dict = NameDictionary::new(NameKind::Player);
        for i in 0..1_000_000u32 {
            dict.intern(&format!("Player{i}, Given{}", i % 997));
        }
        dict.warm_completion_index();
        let started = std::time::Instant::now();
        let found = dict.complete("player5", 10);
        let elapsed = started.elapsed();
        assert_eq!(found.len(), 10);
        assert!(found[0].starts_with("Player5"));
        assert!(
            dict.complete("pgiv", 10).iter().any(|n| n.contains(", Given")),
            "word-chunk match at scale"
        );
        // Debug builds spend most of their time elsewhere; the release number is the one the
        // 50 ms requirement is about.
        #[cfg(not(debug_assertions))]
        assert!(elapsed.as_millis() < 50, "completion took {elapsed:?}");
        let _ = elapsed;
    }
}
