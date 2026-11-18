## Context

See `proposal.md` — Why, and `specs/` for the requirements. The constraints that shape the approach
here are narrower than the product boundaries: the reference suite is copyleft while Spitz has no
declared license yet; the native database format is already a required, documented, third-party
readable format (`game-management`); the 100 ms list, 5 s header search, and 60 s position search
budgets already exist and are unmet by any current design; and the workspace has five crates with a
rule that `game-db` never depends on rendering or on `uci-host`.

The reference's own numbers, measured from its source: a packed header record of 48 bytes per game
(its index chunks are 65,536 entries × 48 bytes), a 24-bit material signature stored inside that
record, interned 28-bit name ids for player/event/round and 32-bit for site, a byte-per-game filter
array with a live match count, and a cross-table capped at 500 players and 60 rounds.

## Goals / Non-Goals

**Goals:**

- Give `game-db` the four primitives every already-written budget assumes: the packed header record,
  the interned name dictionary, the per-game filter, and the sort-rank cache.
- Port behaviour, not lines: every component below is written in Rust against the reference's
  observable rules, with a provenance note naming the file that was read.
- Keep every new user-visible surface derivable from local files only.

**Non-Goals:**

- No reader or writer for the reference suite's on-disk database container.
- No change to `uci-host`; none of this touches an engine, an option, or a process.
- No secondary-index work (that is `add-power-user-features`); this change supplies the filter and
  cache primitives it consumes, not the indexes.
- No online rating lists, no online books, no name-correction dictionaries fetched from anywhere.

## Decisions

**D1 — Re-express algorithms and byte layouts; translate no copyleft lines.** The reference is GPL;
one file in it, its PGN tokenizer, is MIT. A line-for-line translation is a derivative work and would
decide Spitz's license for it. Alternative considered: translate directly and accept copyleft —
rejected, the project has not chosen a license and this change must not choose one. Alternative
considered: ignore the reference and re-derive — rejected, the value is in its edge cases (partial
dates, colour-flip signatures, round collation, tie-break rules), which are cheap to adopt and
expensive to rediscover. Consequence: each ported module carries a `// Behaviour studied from
<reference file>; no source copied.` note, the MIT tokenizer keeps its notice in `NOTICE.md`, and a
license-audit task gates the change.

**D2 — No legacy container support.** Reading the reference's own database files would be the single
largest item in this change (index, name base, two game codecs, proxy codec) and would make Spitz a
reader for a container the project has ruled out. The field set it stores is already carried by the
native format, and PGN is the migration path. Consequence: users migrating from that suite re-import
PGN and lose per-game bookkeeping the native format does not share with it; if that proves to matter,
a read-only importer is a separate change behind the same `Library` opening API.

**D3 — Packed fixed-size header record in `game-db`, target 48 bytes per game.** Field set mirrors
the proven one: game offset and length, interned ids for white/black/event/round/site, ratings plus
rating kind, date and event date as partial dates, result, flags, ECO code, final material signature,
comment/variation/NAG counts, half-move count. Alternatives: one SQLite row per game (rejected — the
5 s header-search budget is met by scanning a 48 MB mmap far more predictably than by query planning,
and the format must be documented for third parties anyway); variable-length records (rejected — kills
direct lookup by game number). Consequence: the on-disk layout is versioned and documented, and adding
a field means a format version bump plus a rebuild path.

**D4 — Interned name dictionary in `game-db`, one collation for the whole app.** Names are stored
once per type with `u32` ids; a collation index gives prefix lookup. The collation is: diacritic-fold,
ASCII-case-fold, compare; rounds additionally compare a leading integer before the text; players
additionally match under surname/given rotation. Alternatives: `LIKE` queries over a text column
(rejected — no prefix index without a second structure, and the collation would be duplicated per
query site). Consequence: the collation is part of the on-disk contract, so it carries a version byte
in the dictionary header and in every sort-cache key; renaming or deleting names needs a compaction
pass, which is the same pass `add-power-user-features` already needs for its indexes.

**D5 — Byte-per-game filter with a live match count, cheapest filter first.** A byte per game rather
than a bit supports the tri-state (yes/no/both) the flag filters need and lets concurrent filters
update an atomic count without read-modify-write on shared words. 1 MB per 1,000,000 games per active
filter is affordable; a bitset would save 7/8 of it and cost the tri-state. Alternative considered and
rejected: an index-only iterator chain, which cannot express "count what is left" and therefore cannot
report rows scanned versus index used.

**D6 — Sort-rank cache in `game-db`, background-built, refcounted, interruptible.** Stores the
permutation plus a partial key hash so an edit re-places one row. The reference does the same with a
worker thread and an interrupt flag; the 100 ms list budget is unreachable without it. Consequence:
the ceiling is 8 bytes per row, so a 1,000,000-row view costs 8 MB, and a cache is dropped when its
view closes rather than kept warm across sessions.

**D7 — Cross-table is pure data in `game-db`; the three output forms are emitters over one
structure.** Mode detection, tie-breaks, and clash storage are rules work and are testable without a
UI; the 500×500 clash matrix is what forces the player cap (500 × 500 clash vectors is already the
dominant allocation). Alternative: compute in the front end — rejected, the numbers must be identical
across plain, HTML, and typeset output, which is a requirement.

**D8 — Binary book interchange reads into the native store; the source file is never written.** The
interchange key is a published 781-entry position-key table, so verifying a book means reproducing that
table exactly as data, not recomputing a Spitz-internal hash. Consequence: the table ships as a
versioned data file with a provenance note, and the golden-fixture test that every key of a real book
verifies is the gate on this decision. Weights map to play count; the native fields the format does not
carry stay unknown, which is why the spec forbids rendering them as zero.

**D9 — ECO classification as a half-move prefix trie in `game-db`.** Built from the user's code file at
first use, longest match wins, code stored in the header record. Alternative: store the code file as a
PGN and re-parse per game — rejected, a 1,000,000-game pass must finish in 10 minutes.

**Crate placement.** `chess-core`: partial dates, material and home-pawn signatures, PGN token set,
position keys. `game-db`: header record, name dictionary and collation, filter, sort cache, library
sessions and locking, ECO trie, cross-table, binary book reader and writer. `desktop`: cross-table
panel, completion in filter and game-entry fields, ECO column. No new workspace crates.

## Risks / Trade-offs

- Position-key table for book interchange does not match the published table → the first task is a
  fixture book with golden keys; if any key mismatches, the requirement is dropped rather than shipped
  half-verified.
- Collation is a hidden contract: a change silently invalidates cached sort orders and dictionary
  lookups → version byte in the dictionary header and in the sort-cache key, mismatch forces a rebuild.
- Cooperative locking is not enforcement on network filesystems → the app never opens the same path
  twice in one process, the lock file records host and pid, and the refusal message names the path so
  the user can resolve it.
- Saturating material counts: a promoted army can exceed 4-bit pawn or 2-bit piece fields → saturate
  the field and mark the signature inexact rather than wrapping; material search then falls back to a
  position scan for that game.
- Partial dates that match a range the user did not intend (a `1999.??.??` game shown in a March–
  November filter) → the list always shows the raw date text, and the filter row states that unknown
  components match.
- 500-player cross-table cap silently drops games → the report header states the excluded counts, per
  the spec.

## Migration Plan

No data migration: the native format has no shipped data yet. If the header record layout changes
while the change is in flight, bump the format version and rebuild the index rather than migrate.
Rollback is per feature flag (`library-insight`, `binary-book-import`, `eco-classification`); the
header record, filter, and sort cache are infrastructure and roll back only as a unit with the native
format version.

## Sequencing

While `add-core-platform-mvp` has not fixed the native library file format, this change proceeds
only on self-contained structures - dates, PGN lexing, material signatures, the packed header
record, the name dictionary, collation, sessions as data - each landing with its own tests and no
reader or writer wiring to a data file. Anything that stores or resolves byte offsets into a game
data file waits for the M1 format task (7.3), because an integration layer written against an
undefined format is written twice. The already-written budgets and the format version in this
change's roll-back notes assume that order; violating it is a design-review finding, not a
shortcut.

## Open Questions

- The rating-list file format: adopt the reference's own list layout or define a documented CSV with
  the same columns. Decide when the parser task starts; neither answer changes a requirement.
- Whether the sort-rank cache persists between sessions for a saved view. Deferred until the
  `add-power-user-features` index files exist, since they will already carry a freshness record to
  reuse.
