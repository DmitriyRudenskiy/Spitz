//! The filter: one byte per game saying whether it is in the current view, and the plan that
//! fills it in the cheapest order.
//!
//! A byte rather than a bit is the deliberate choice recorded in the design note: the flag
//! filters need a third state, and a live match count is one atomic add per row instead of a
//! read-modify-write on a shared word. That costs 1 MB per million games per active filter.

use crate::header_record::HeaderRecord;
use chess_core::date::{Date, DateRange};
use chess_core::pgn::GameResult;
use std::sync::atomic::{AtomicU32, Ordering};

/// What the filter says about one game.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    No,
    Yes,
    /// Neither answer — the state a "show both" flag filter leaves behind. Counts as a match.
    Both,
}

impl State {
    const fn byte(self) -> u8 {
        match self {
            State::No => 0,
            State::Yes => 1,
            State::Both => 2,
        }
    }

    const fn from_byte(byte: u8) -> Self {
        match byte {
            1 => State::Yes,
            2 => State::Both,
            _ => State::No,
        }
    }

    /// Whether a game in this state belongs to the view.
    pub const fn matches(self) -> bool {
        !matches!(self, State::No)
    }
}

/// A byte per game, and how many of them currently match.
///
/// Nothing is allocated until the view actually filters something, so a library opened without a
/// filter costs nothing.
#[derive(Debug, Default)]
pub struct Filter {
    bytes: Vec<u8>,
    matches: AtomicU32,
}

impl Clone for Filter {
    fn clone(&self) -> Self {
        Self { bytes: self.bytes.clone(), matches: AtomicU32::new(self.matches() as u32) }
    }
}

impl Filter {
    /// An empty filter holding nothing.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the per-game bytes exist yet.
    pub fn is_allocated(&self) -> bool {
        !self.bytes.is_empty()
    }

    /// Games the filter covers.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Games currently matching. Safe to read while another thread fills the filter.
    pub fn matches(&self) -> usize {
        self.matches.load(Ordering::Relaxed) as usize
    }

    /// Set the number of games, keeping the state of the rows that survive. A filter that has
    /// never been allocated stays unallocated until it holds something.
    pub fn resize(&mut self, len: usize) {
        if len < self.bytes.len() {
            let lost =
                self.bytes[len..].iter().filter(|&&byte| State::from_byte(byte).matches()).count();
            self.matches.fetch_sub(lost as u32, Ordering::Relaxed);
        }
        // New rows are not matches, so growing leaves the count alone.
        self.bytes.resize(len, State::No.byte());
    }

    /// Set every game to the same state.
    pub fn fill(&mut self, state: State) {
        let matching = state.matches() as u32;
        self.bytes.fill(state.byte());
        self.matches.store(self.bytes.len() as u32 * matching, Ordering::Relaxed);
    }

    /// The state of one game. Out of range reads as [`State::No`].
    pub fn get(&self, row: usize) -> State {
        State::from_byte(*self.bytes.get(row).unwrap_or(&0))
    }

    /// Set one game, keeping the live count honest. Returns whether the answer changed.
    pub fn set(&mut self, row: usize, state: State) -> bool {
        let Some(byte) = self.bytes.get_mut(row) else { return false };
        let before = State::from_byte(*byte).matches();
        *byte = state.byte();
        let after = state.matches();
        if before == after {
            return false;
        }
        if after {
            self.matches.fetch_add(1, Ordering::Relaxed);
        } else {
            self.matches.fetch_sub(1, Ordering::Relaxed);
        }
        true
    }

    /// Drop every game that does not satisfy `test`, counting as it goes.
    pub fn retain(&mut self, mut test: impl FnMut(usize) -> bool) {
        let mut count = 0u32;
        for (row, byte) in self.bytes.iter_mut().enumerate() {
            if State::from_byte(*byte).matches() {
                if test(row) {
                    count += 1;
                } else {
                    *byte = State::No.byte();
                }
            }
        }
        self.matches.store(count, Ordering::Relaxed);
    }

    /// Fill from a list of matching rows, used by the index-backed path.
    pub fn select(&mut self, rows: impl IntoIterator<Item = usize>) {
        self.bytes.fill(State::No.byte());
        let mut count = 0u32;
        for row in rows {
            if let Some(byte) = self.bytes.get_mut(row)
                && *byte == State::No.byte()
            {
                *byte = State::Yes.byte();
                count += 1;
            }
        }
        self.matches.store(count, Ordering::Relaxed);
    }
}

/// What a view can be filtered by. Each variant knows how selective it expects to be, which is
/// what the plan orders by.
#[derive(Clone, Copy, Debug)]
pub enum Criterion {
    /// Games whose indexed name is exactly this id. Answered from the position index.
    Name { id: u32 },
    DateRange(DateRange),
    Result(GameResult),
    Eco(u16),
    /// A flag character set on the game.
    Flag(char),
    WhiteRating { low: u16, high: u16 },
    BlackRating { low: u16, high: u16 },
}

impl Criterion {
    /// Rough share of a normal library this criterion leaves in. Only the order matters: it is
    /// what makes the cheapest test run first.
    pub fn expected_selectivity(&self) -> f32 {
        match self {
            Criterion::Name { .. } => 0.0005,
            Criterion::Eco(_) => 0.01,
            Criterion::Flag(_) => 0.05,
            Criterion::Result(_) => 0.33,
            Criterion::WhiteRating { .. } | Criterion::BlackRating { .. } => 0.3,
            Criterion::DateRange(_) => 0.2,
        }
    }

    /// Whether a position index answers this without reading a single header record.
    pub fn is_indexed(&self) -> bool {
        matches!(self, Criterion::Name { .. })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Criterion::Name { .. } => "name",
            Criterion::DateRange(_) => "date range",
            Criterion::Result(_) => "result",
            Criterion::Eco(_) => "eco",
            Criterion::Flag(_) => "flag",
            Criterion::WhiteRating { .. } => "white rating",
            Criterion::BlackRating { .. } => "black rating",
        }
    }

    fn holds(&self, record: &HeaderRecord) -> bool {
        match self {
            // A name is compared by id, which is the whole point of interning it.
            Criterion::Name { .. } => true,
            Criterion::DateRange(range) => range.matches(record.date),
            Criterion::Result(result) => record.result == *result,
            Criterion::Eco(eco) => record.eco == *eco,
            Criterion::Flag(flag) => record.flags & flag_bit(*flag) != 0,
            Criterion::WhiteRating { low, high } => record.white_rating >= *low && record.white_rating <= *high,
            Criterion::BlackRating { low, high } => record.black_rating >= *low && record.black_rating <= *high,
        }
    }
}

/// The bit a flag character occupies in [`HeaderRecord::flags`].
pub const fn flag_bit(flag: char) -> u32 {
    match flag {
        'r' => 1 << 0,
        'n' => 1 << 1,
        'b' => 1 << 2,
        'w' => 1 << 3,
        'x' => 1 << 4,
        'd' => 1 << 5,
        'p' => 1 << 6,
        'm' => 1 << 7,
        'o' => 1 << 8,
        'g' => 1 << 9,
        _ => 0,
    }
}

/// Where the games of one name are, without reading the library.
#[derive(Debug, Default)]
pub struct PositionIndex {
    /// Row numbers grouped by name id.
    rows: Vec<u32>,
    /// The name id of each entry in `rows`, so a name's games are one contiguous range.
    ids: Vec<u32>,
}

impl PositionIndex {
    /// Group every row by its name id. `id_of` reads the field being indexed.
    pub fn build(records: &[HeaderRecord], id_of: impl Fn(&HeaderRecord) -> u32) -> Self {
        let mut pairs: Vec<(u32, u32)> = records
            .iter()
            .enumerate()
            .map(|(row, record)| (id_of(record), row as u32))
            .collect();
        pairs.sort_unstable();
        Self { ids: pairs.iter().map(|p| p.0).collect(), rows: pairs.iter().map(|p| p.1).collect() }
    }

    /// The rows whose name id is `id`.
    pub fn rows_for(&self, id: u32) -> std::ops::Range<usize> {
        let start = self.ids.partition_point(|&x| x < id);
        let end = self.ids.partition_point(|&x| x <= id);
        start..end
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// What applying a plan did, so a UI can say "1,000,000 games, 340 scanned".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// Header records read from the library.
    pub rows_scanned: usize,
    /// Games that survived the filter.
    pub rows_matched: usize,
    /// Criteria answered by the position index instead of a scan.
    pub index_used: usize,
    /// Criteria applied in the order they were applied.
    pub applied: usize,
}

/// Fill `filter` for `records` using `criteria`, most useful first.
///
/// An indexed criterion runs first and costs its posting list, not the library; the rest are
/// tested against the surviving rows only, in ascending expected selectivity, and a row that
/// fails one test never reaches the next.
pub fn apply(
    records: &[HeaderRecord],
    index: &PositionIndex,
    criteria: &[Criterion],
    filter: &mut Filter,
) -> Report {
    filter.resize(records.len());
    let mut order: Vec<&Criterion> = criteria.iter().collect();
    order.sort_by(|a, b| {
        b.is_indexed()
            .cmp(&a.is_indexed())
            .then(a.expected_selectivity().total_cmp(&b.expected_selectivity()))
    });
    let mut report = Report { applied: order.len(), ..Default::default() };
    // `None` means "every row so far", which is the scan path; `Some` is a narrowed candidate
    // list, which is where an indexed criterion puts us.
    let mut candidates: Option<Vec<u32>> = None;
    for criterion in order {
        if criterion.is_indexed() {
            let Criterion::Name { id } = *criterion else { unreachable!() };
            let rows = index.rows_for(id);
            report.rows_scanned += rows.len();
            report.index_used += 1;
            candidates = Some(rows.map(|at| index.rows[at]).collect());
            continue;
        }
        match candidates.as_mut() {
            None => filter.retain(|row| {
                report.rows_scanned += 1;
                criterion.holds(&records[row])
            }),
            Some(list) => list.retain(|&row| {
                report.rows_scanned += 1;
                criterion.holds(&records[row as usize])
            }),
        }
    }
    if let Some(candidates) = candidates {
        filter.select(candidates.into_iter().map(|row| row as usize));
    }
    report.rows_matched = filter.matches();
    report
}

/// A date range over the two dates a game carries.
pub fn years_range(low: u16, high: u16) -> DateRange {
    DateRange {
        start: Date::new(low, 1, 1),
        end: Date::new(high, 12, 31),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::name_dict::{NameDictionary, NameKind};

    fn records(count: usize) -> Vec<HeaderRecord> {
        (0..count)
            .map(|i| HeaderRecord {
                white: (i % 97) as u32,
                black: (i % 53) as u32,
                date: Date::new(1970 + (i % 55) as u16, 1 + (i % 12) as u8, 1 + (i % 28) as u8),
                result: [GameResult::WhiteWins, GameResult::Draw, GameResult::BlackWins][i % 3],
                eco: (i % 300) as u16,
                flags: if i % 10 == 0 { flag_bit('p') } else { 0 },
                white_rating: (1500 + (i % 1500)) as u16,
                black_rating: (1500 + (i % 1500)) as u16,
                ..Default::default()
            })
            .collect()
    }

    #[test]
    fn a_filter_holds_nothing_until_it_filters() {
        let mut filter = Filter::new();
        assert!(!filter.is_allocated());
        assert_eq!(filter.matches(), 0);
        assert_eq!(filter.get(0), State::No);
        assert!(!filter.set(0, State::Yes), "there is no row to set");
        filter.resize(10);
        assert!(filter.is_allocated());
        assert_eq!(filter.len(), 10);
        assert_eq!(filter.matches(), 0);
    }

    #[test]
    fn the_match_count_follows_every_write() {
        let mut filter = Filter::new();
        filter.resize(100);
        filter.fill(State::Yes);
        assert_eq!(filter.matches(), 100);
        assert!(filter.set(3, State::No));
        assert!(!filter.set(3, State::No), "already out");
        assert_eq!(filter.matches(), 99);
        filter.set(4, State::Both);
        assert_eq!(filter.get(4), State::Both);
        assert!(!filter.set(4, State::Yes), "both and yes are the same answer");
        filter.resize(105);
        assert_eq!(filter.matches(), 99, "new rows start out");
        assert_eq!(filter.get(104), State::No);
        filter.resize(4);
        assert_eq!(filter.matches(), 3);
    }

    #[test]
    fn retain_and_select_rewrite_the_whole_array() {
        let rows = records(60);
        let mut filter = Filter::new();
        filter.resize(rows.len());
        filter.fill(State::Yes);
        filter.retain(|row| row % 2 == 0);
        assert_eq!(filter.matches(), 30);
        filter.select((0..60).step_by(20));
        assert_eq!(filter.matches(), 3);
        assert_eq!(filter.get(20), State::Yes);
        assert_eq!(filter.get(21), State::No);
    }

    #[test]
    fn a_position_index_finds_a_names_rows() {
        let rows = records(500);
        let index = PositionIndex::build(&rows, |r| r.white);
        assert_eq!(index.len(), 500);
        let white_of_7: Vec<usize> = (0..500).filter(|&i| rows[i].white == 7).collect();
        let indexed: Vec<usize> = index.rows_for(7).map(|at| index.rows[at] as usize).collect();
        assert_eq!(indexed, white_of_7);
        assert!(index.rows_for(9999).is_empty());
    }

    #[test]
    fn the_plan_runs_the_indexed_criterion_first() {
        let rows = records(1000);
        let index = PositionIndex::build(&rows, |r| r.white);
        let mut filter = Filter::new();
        let report = apply(
            &rows,
            &index,
            &[
                Criterion::DateRange(years_range(1980, 2000)),
                Criterion::Name { id: 7 },
                Criterion::Result(GameResult::Draw),
            ],
            &mut filter,
        );
        assert_eq!(report.index_used, 1);
        assert_eq!(report.applied, 3);
        // The name holds about one in ninety seven games, so the scan never looks at the library.
        assert!(report.rows_scanned < 200, "scanned {}", report.rows_scanned);
        let expected = (0..1000)
            .filter(|&i| rows[i].white == 7 && rows[i].result == GameResult::Draw)
            .filter(|&i| years_range(1980, 2000).matches(rows[i].date))
            .count();
        assert_eq!(report.rows_matched, expected);
        assert_eq!(filter.matches(), expected);
    }

    #[test]
    fn a_scan_only_plan_still_short_circuits() {
        let rows = records(1000);
        let index = PositionIndex::build(&rows, |r| r.white);
        let mut filter = Filter::new();
        let report = apply(
            &rows,
            &index,
            &[
                Criterion::Result(GameResult::Draw),
                Criterion::Eco(3),
                Criterion::Flag('p'),
            ],
            &mut filter,
        );
        assert_eq!(report.index_used, 0);
        // The first criterion reads the library; the rest only read what its winner left.
        assert!(report.rows_scanned < 1200, "scanned {}", report.rows_scanned);
        let expected = (0..1000)
            .filter(|&i| rows[i].result == GameResult::Draw && rows[i].eco == 3 && rows[i].flags != 0)
            .count();
        assert_eq!(report.rows_matched, expected);
    }

    #[test]
    fn a_filter_combination_narrows_a_million_games_quickly() {
        let rows = records(1_000_000);
        let index = PositionIndex::build(&rows, |r| r.white);
        let mut dict = NameDictionary::new(NameKind::Player);
        dict.intern("whoever");
        let started = std::time::Instant::now();
        let mut filter = Filter::new();
        let report = apply(
            &rows,
            &index,
            &[
                Criterion::Name { id: dict.find("whoever").unwrap() },
                Criterion::DateRange(years_range(1990, 1995)),
                Criterion::WhiteRating { low: 1800, high: 2200 },
            ],
            &mut filter,
        );
        let elapsed = started.elapsed();
        assert_eq!(report.index_used, 1);
        // The name holds a tenth of the library, so the index is what keeps this off the whole file.
        assert!(report.rows_scanned < 30_000, "scanned {} of a million", report.rows_scanned);
        assert!(report.rows_matched > 0);
        assert_eq!(filter.matches(), report.rows_matched);
        #[cfg(not(debug_assertions))]
        assert!(elapsed.as_millis() < 250, "filtering took {elapsed:?}");
        let _ = elapsed;
    }
}
