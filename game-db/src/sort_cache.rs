//! The cached sort order: the permutation a view pages through, built in the background and
//! interruptible, at eight bytes a row.
//!
//! Re-sorting a 100,000-row result set is a lookup in [`SortCacheTable`], not a sort; the sort
//! happens once per (criteria, filter) pair and its result is kept while the view is open.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

/// The column a view is sorted by. The caller turns a row's value into a `u32` sort key, so the
/// cache itself stays eight bytes a row whatever the column is.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Column {
    Date,
    Event,
    Site,
    Round,
    White,
    Black,
    Result,
    WhiteRating,
    BlackRating,
    Eco,
    HalfMoves,
    Length,
}

/// Which way the rows run. Reversing is a flag on the read, never a rebuild.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Direction {
    Ascending,
    Descending,
}

/// What a view is sorted by: the columns in priority order.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct SortKey {
    pub columns: Vec<Column>,
}

impl SortKey {
    pub fn new(columns: impl IntoIterator<Item = Column>) -> Self {
        Self { columns: columns.into_iter().collect() }
    }
}

/// How far a background build has got.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuildStatus {
    /// A build is running.
    Building,
    /// The published order is the one asked for.
    Ready,
    /// The build was interrupted; the published order is whatever was there before.
    Interrupted,
    /// Nothing has been published yet.
    Empty,
}

/// The ceiling `SortCache` holds itself to: four bytes for the permutation and four for the
/// partial sort key, per row.
pub const BYTES_PER_ROW: usize = 8;

pub struct SortCache {
    key: SortKey,
    /// Row numbers in sorted order. Always usable: a build writes a scratch copy and swaps.
    order: Vec<u32>,
    /// The sort key of each row in `order`, so a re-placement can find its neighbourhood.
    keys: Vec<u32>,
    /// Views holding this order.
    users: usize,
    build: Option<Build>,
}

/// What a build produces: the permutation and its keys, or nothing if it was interrupted.
type Built = Option<(Vec<u32>, Vec<u32>)>;

struct Build {
    handle: Option<JoinHandle<Built>>,
    cancel: Arc<AtomicBool>,
    interrupted: bool,
}

impl SortCache {
    /// A cache with nothing published yet.
    pub fn new(key: SortKey) -> Self {
        Self { key, order: Vec::new(), keys: Vec::new(), users: 1, build: None }
    }

    pub fn key(&self) -> &SortKey {
        &self.key
    }

    /// Sort `rows` now. Used by tests, by small views, and by anything that cannot wait.
    pub fn build_now(rows: &[u32], key: SortKey, key_of: impl Fn(u32) -> u32) -> Self {
        let (order, keys) = sort_rows(rows, &key_of, &AtomicBool::new(false)).expect("not cancelled");
        Self { key, order, keys, users: 1, build: None }
    }

    /// Start a background build. The previous order stays published and usable until this one
    /// finishes; [`Self::poll`] publishes it.
    pub fn start_build(&mut self, rows: &[u32], key_of: Arc<dyn Fn(u32) -> u32 + Send + Sync>) {
        self.cancel_build();
        let rows = rows.to_vec();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_for_thread = Arc::clone(&cancel);
        let handle = std::thread::spawn(move || sort_rows(&rows, key_of.as_ref(), &cancel_for_thread));
        self.build = Some(Build { handle: Some(handle), cancel, interrupted: false });
    }

    /// Ask a running build to stop. The published order does not change.
    pub fn cancel_build(&mut self) {
        if let Some(build) = self.build.take() {
            build.cancel.store(true, Ordering::Relaxed);
            if let Some(handle) = build.handle {
                let _ = handle.join();
            }
        }
    }

    /// Where the build stands, publishing its result if it finished. Never blocks unless a build
    /// was interrupted and has not been joined yet.
    pub fn status(&mut self) -> BuildStatus {
        let Some(build) = self.build.as_mut() else {
            return if self.order.is_empty() { BuildStatus::Empty } else { BuildStatus::Ready };
        };
        if build.interrupted {
            return BuildStatus::Interrupted;
        }
        let finished = build
            .handle
            .as_ref()
            .is_some_and(|handle| handle.is_finished());
        if !finished {
            return BuildStatus::Building;
        }
        let handle = build.handle.take().expect("just checked");
        match handle.join() {
            Ok(Some((order, keys))) => {
                self.order = order;
                self.keys = keys;
                self.build = None;
                BuildStatus::Ready
            }
            Ok(None) => BuildStatus::Interrupted,
            Err(_) => BuildStatus::Interrupted,
        }
    }

    /// Interrupt a running build and keep the order already published.
    pub fn interrupt(&mut self) {
        if let Some(build) = self.build.as_mut() {
            build.cancel.store(true, Ordering::Relaxed);
            build.interrupted = true;
        }
    }

    /// Rows in the view.
    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// The whole order, ascending.
    pub fn order(&self) -> &[u32] {
        &self.order
    }

    /// One page of the view, in the requested direction. Cost is the page, not the set.
    pub fn page(&self, offset: usize, limit: usize, direction: Direction) -> Vec<u32> {
        let total = self.order.len();
        let end = offset.saturating_add(limit).min(total);
        if offset >= total || limit == 0 {
            return Vec::new();
        }
        match direction {
            Direction::Ascending => self.order[offset..end].to_vec(),
            // The descending view is the same order read from the other end.
            Direction::Descending => self.order[total - end..total - offset].iter().rev().copied().collect(),
        }
    }

    /// Where `row` sits in the order, if it is in it.
    pub fn position_of(&self, row: u32) -> Option<usize> {
        self.order.iter().position(|&candidate| candidate == row)
    }

    /// One game's sort key changed: move its row and leave the rest where they are.
    ///
    /// The row is lifted out and re-inserted where its new key sorts, which is a binary search
    /// and two memmoves. Cost is the distance it travels, not the size of the set.
    pub fn reposition(&mut self, row: u32, new_key: u32) -> Option<usize> {
        let at = self.position_of(row)?;
        self.order.remove(at);
        self.keys.remove(at);
        // Rows sharing a key keep their row order, so the search walks past the equal keys.
        let mut insert = self.keys.partition_point(|&key| key < new_key);
        while insert < self.order.len() && self.keys[insert] == new_key && self.order[insert] < row {
            insert += 1;
        }
        self.order.insert(insert, row);
        self.keys.insert(insert, new_key);
        Some(insert)
    }

    /// Bytes held for the rows, which must stay at or under [`BYTES_PER_ROW`] a row.
    pub fn memory_bytes(&self) -> usize {
        (self.order.capacity() + self.keys.capacity()) * 4
    }

    pub fn users(&self) -> usize {
        self.users
    }

    pub fn acquire(&mut self) {
        self.users += 1;
    }

    /// One view closed. Returns whether the cache is now nobody's.
    pub fn release(&mut self) -> bool {
        self.users = self.users.saturating_sub(1);
        self.users == 0
    }
}

/// The orders of one library and filter, keyed by what the view is sorted by.
#[derive(Default)]
pub struct SortCacheTable {
    entries: HashMap<SortKey, SortCache>,
    /// Full sorts performed, which is what the "no re-sort" scenario asserts on.
    sorts: usize,
}

impl SortCacheTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sorts_performed(&self) -> usize {
        self.sorts
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The order for `key`, sorting only if nothing holds it yet.
    pub fn sort_now(&mut self, rows: &[u32], key: SortKey, key_of: impl Fn(u32) -> u32) -> &mut SortCache {
        if !self.entries.contains_key(&key) {
            self.sorts += 1;
            let cache = SortCache::build_now(rows, key.clone(), key_of);
            self.entries.insert(key.clone(), cache);
        }
        self.entries.get_mut(&key).expect("just inserted")
    }

    /// The order for `key`, starting a background build if nothing holds it yet.
    pub fn order_for(
        &mut self,
        rows: &[u32],
        key: SortKey,
        key_of: Arc<dyn Fn(u32) -> u32 + Send + Sync>,
    ) -> &mut SortCache {
        let fresh = !self.entries.contains_key(&key);
        if fresh {
            self.sorts += 1;
            let mut cache = SortCache::new(key.clone());
            cache.start_build(rows, key_of);
            self.entries.insert(key.clone(), cache);
        }
        self.entries.get_mut(&key).expect("just inserted")
    }

    /// Drop orders no view is holding.
    pub fn prune(&mut self) {
        self.entries.retain(|_, cache| cache.users() > 0);
    }

    pub fn get(&self, key: &SortKey) -> Option<&SortCache> {
        self.entries.get(key)
    }

    pub fn get_mut(&mut self, key: &SortKey) -> Option<&mut SortCache> {
        self.entries.get_mut(key)
    }
}

/// Rows of `rows` sorted by their key, ties by row number, giving up between chunks if asked.
///
/// The chunked merge is what makes the build interruptible: a single `sort_unstable_by` cannot
/// be stopped halfway, and a build that cannot be stopped holds its thread and its memory while
/// the user has already moved on.
fn sort_rows<F: Fn(u32) -> u32 + ?Sized>(
    rows: &[u32],
    key_of: &F,
    cancel: &AtomicBool,
) -> Option<(Vec<u32>, Vec<u32>)> {
    const CHUNK: usize = 4096;
    let mut runs: Vec<Vec<u32>> = Vec::new();
    for chunk in rows.chunks(CHUNK) {
        if cancel.load(Ordering::Relaxed) {
            return None;
        }
        let mut run: Vec<u32> = chunk.to_vec();
        run.sort_unstable_by_key(|&row| (key_of(row), row));
        runs.push(run);
    }
    while runs.len() > 1 {
        if cancel.load(Ordering::Relaxed) {
            return None;
        }
        let mut merged: Vec<Vec<u32>> = Vec::with_capacity(runs.len().div_ceil(2));
        let mut round = runs.into_iter();
        while let Some(left) = round.next() {
            match round.next() {
                None => merged.push(left),
                Some(right) => merged.push(merge(&left, &right, key_of)),
            }
        }
        runs = merged;
    }
    let order = runs.into_iter().next().unwrap_or_default();
    let keys = order.iter().map(|&row| key_of(row)).collect();
    Some((order, keys))
}

fn merge<F: Fn(u32) -> u32 + ?Sized>(left: &[u32], right: &[u32], key_of: &F) -> Vec<u32> {
    let mut out = Vec::with_capacity(left.len() + right.len());
    let (mut i, mut j) = (0, 0);
    while i < left.len() && j < right.len() {
        if (key_of(left[i]), left[i]) <= (key_of(right[j]), right[j]) {
            out.push(left[i]);
            i += 1;
        } else {
            out.push(right[j]);
            j += 1;
        }
    }
    out.extend_from_slice(&left[i..]);
    out.extend_from_slice(&right[j..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A view of `count` rows whose date key runs backwards, so a sort has something to do.
    fn rows(count: u32) -> Vec<u32> {
        (0..count).collect()
    }

    fn date_key(row: u32) -> u32 {
        100_000 - row % 50_000
    }

    fn rating_key(row: u32) -> u32 {
        (row % 997) * 3
    }

    #[test]
    fn an_order_costs_eight_bytes_a_row() {
        let cache = SortCache::build_now(&rows(100_000), SortKey::new([Column::Date]), date_key);
        assert_eq!(cache.len(), 100_000);
        assert!(
            cache.memory_bytes() <= BYTES_PER_ROW * cache.len(),
            "{} bytes for {} rows",
            cache.memory_bytes(),
            cache.len()
        );
    }

    #[test]
    fn a_built_order_is_sorted_and_pageable() {
        let cache = SortCache::build_now(&rows(1000), SortKey::new([Column::Date]), date_key);
        let order = cache.order();
        assert!(order.windows(2).all(|pair| date_key(pair[0]) <= date_key(pair[1])));
        assert_eq!(cache.page(0, 3, Direction::Ascending), vec![999, 998, 997]);
        assert_eq!(cache.page(0, 3, Direction::Descending), vec![0, 1, 2]);
        assert_eq!(cache.page(1, 3, Direction::Descending), vec![1, 2, 3]);
        assert_eq!(cache.page(998, 10, Direction::Ascending).len(), 2);
        assert!(cache.page(1000, 10, Direction::Ascending).is_empty());
        assert_eq!(cache.position_of(order[7]), Some(7));
    }

    #[test]
    fn a_background_build_publishes_when_it_finishes() {
        let mut cache = SortCache::new(SortKey::new([Column::Date]));
        assert_eq!(cache.status(), BuildStatus::Empty);
        cache.start_build(&rows(20_000), Arc::new(date_key));
        assert_eq!(cache.status(), BuildStatus::Building);
        for _ in 0..1_000 {
            if cache.status() == BuildStatus::Ready {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(cache.status(), BuildStatus::Ready);
        assert_eq!(cache.len(), 20_000);
        let order = cache.order();
        assert!(order.windows(2).all(|pair| date_key(pair[0]) <= date_key(pair[1])));
    }

    #[test]
    fn an_interrupted_build_leaves_the_previous_order_usable() {
        let mut cache = SortCache::build_now(&rows(400), SortKey::new([Column::Date]), date_key);
        let before = cache.order().to_vec();
        cache.start_build(&rows(400_000), Arc::new(rating_key));
        cache.interrupt();
        assert_eq!(cache.status(), BuildStatus::Interrupted);
        assert_eq!(cache.order(), before.as_slice(), "the old order is still the view");
        assert!(cache.order().windows(2).all(|pair| date_key(pair[0]) <= date_key(pair[1])));
        // And the cache is usable again for a build that is allowed to finish.
        cache.start_build(&rows(400), Arc::new(rating_key));
        while cache.status() == BuildStatus::Building {
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(cache.status(), BuildStatus::Ready);
        assert!(cache.order().windows(2).all(|pair| rating_key(pair[0]) <= rating_key(pair[1])));
    }

    #[test]
    fn editing_one_row_moves_only_that_row() {
        let count = 100_000u32;
        let mut cache = SortCache::build_now(&rows(count), SortKey::new([Column::Date]), date_key);
        let before = cache.order().to_vec();
        let moved = before[10];
        let started = std::time::Instant::now();
        let at = cache.reposition(moved, u32::MAX).expect("row is in the order");
        let elapsed = started.elapsed();
        assert_eq!(cache.len(), count as usize, "the set did not change size");
        assert_eq!(cache.order()[at], moved);
        let key = |row: u32| if row == moved { u32::MAX } else { date_key(row) };
        assert!(cache.order().windows(2).all(|pair| key(pair[0]) <= key(pair[1])));
        let others: Vec<u32> = cache.order().iter().copied().filter(|&row| row != moved).collect();
        let before_others: Vec<u32> = before.iter().copied().filter(|&row| row != moved).collect();
        assert_eq!(others, before_others, "every other row kept its position");
        #[cfg(not(debug_assertions))]
        assert!(elapsed.as_millis() < 100, "re-placement took {elapsed:?}");
        let _ = elapsed;
        assert_eq!(cache.reposition(999_999, 1), None);
    }

    #[test]
    #[ignore = "performance budget: runs on the reference machine only, ADR-0010 and budgets.yml"]
    fn re_sorting_a_cached_view_does_not_sort_again() {
        let count = 100_000u32;
        let rows = rows(count);
        let mut table = SortCacheTable::new();
        let by_date = SortKey::new([Column::Date]);
        let by_rating = SortKey::new([Column::WhiteRating]);
        let started = std::time::Instant::now();
        table.sort_now(&rows, by_date.clone(), date_key);
        table.sort_now(&rows, by_rating.clone(), rating_key);
        let cold = started.elapsed();
        assert_eq!(table.sorts_performed(), 2);

        let started = std::time::Instant::now();
        for _ in 0..4 {
            let cache = table.sort_now(&rows, by_date.clone(), date_key);
            assert_eq!(cache.page(500, 20, Direction::Ascending).len(), 20);
            let cache = table.sort_now(&rows, by_rating.clone(), rating_key);
            assert_eq!(cache.page(500, 20, Direction::Descending).len(), 20);
        }
        let warm = started.elapsed();
        assert_eq!(table.sorts_performed(), 2, "no full re-sort of the set");
        assert!(warm < cold, "warm sorts should not cost more than the cold ones");
        #[cfg(not(debug_assertions))]
        assert!(warm.as_millis() < 400, "eight warm sorts took {warm:?}");
        let _ = warm;
    }

    #[test]
    fn an_order_is_dropped_when_its_last_view_closes() {
        let mut table = SortCacheTable::new();
        let key = SortKey::new([Column::Date]);
        table.sort_now(&rows(100), key.clone(), date_key);
        assert_eq!(table.len(), 1);
        table.get_mut(&key).unwrap().acquire();
        assert!(!table.get_mut(&key).unwrap().release());
        table.prune();
        assert!(table.get(&key).is_some(), "a view still holds it");
        assert!(table.get_mut(&key).unwrap().release());
        table.prune();
        assert!(table.get(&key).is_none());
    }
}
