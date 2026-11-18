# Change: port-reference-database-internals

## Why

A 48k-line C++/Tcl chess database suite with two decades of field use was read end to end to find
what Spitz does not yet have and cannot get from an engine. Two findings. First, real gaps in the
spec set: nothing specifies how a game gets an ECO code, how a mistyped player name is found, how a
100,000-row list stays at 100 ms while it is being re-sorted, how a PGN date with unknown components
behaves, or how a user's existing binary opening books are read. Second, a set of proven internals —
packed header record, interned name dictionary, lazily-allocated per-game filter, sort-rank cache,
material signature, database session pool — that are the mechanism the already-written budgets in
`game-management` and `performance-budgets` silently assume. This change ports the behaviour those
internals deliver, as original Rust, and ports the algorithms and byte layouts behind them.

## What Changes

- **New capability `library-insight`** — cross-table reports over any filtered game set (all-play-all,
  Swiss, knockout, auto-detected, with documented tie-breaks and plain/HTML/LaTeX output), header-name
  completion and correction over the library's own name dictionary, and player profile attributes
  (title, federation, birth date and age at event, peak rating) from a user-supplied rating list.
- **`game-management`** (modified) — ADDED: a reusable sorted-order cache behind the virtualized list;
  multi-library session management with read-only detection and cooperative locking; ECO classification
  of every imported game from a user-supplied code file.
- **`chess-rules-core`** (modified) — ADDED: partial PGN dates (parse, compare, filter); a packed
  material signature and home-pawn signature computed incrementally; full PGN lexical token coverage
  including the comment, NAG, glyph and escape forms that real files contain.
- **`opening-book`** (modified) — ADDED: read and write interchange with the fixed-size-entry binary
  book format users already own, with key verification and weight mapping.
- Not included, on purpose: no reader or writer for the reference suite's own on-disk database
  container (its field set is already carried by the native format; the migration path stays PGN), and
  no line-for-line translation of GPL-licensed source. See `design.md`.

## Capabilities

### New Capabilities

- `library-insight`: cross-table reports, header-name completion and correction, and player profile
  attributes derived from a game library and a rating list.

### Modified Capabilities

- `game-management`: adds sorted-order caching, library session and locking behaviour, and ECO
  classification.
- `chess-rules-core`: adds partial-date handling, material/home-pawn signatures, and PGN token
  coverage.
- `opening-book`: adds binary book format interchange.

## Impact

- `chess-core`: date type with unknown components; material signature; PGN lexer token set.
- `game-db`: name dictionary with the library collation, per-game filter and filter ordering,
  sort-rank cache, library session pool, ECO classifier, cross-table builder, binary book reader and
  writer.
- `desktop`: cross-table panel, name completion in filter and game-entry fields, ECO column.
- `uci-host`: none. No engine work, no network, no new dependency beyond what the workspace already
  uses for mmap and Unicode case folding.
- Sequencing: builds on `add-core-platform-mvp` (native format, game list) and is independent of
  `add-power-user-features`; the secondary-index work there reuses the filter and sort-cache
  primitives defined here.
- Licensing: algorithms and file layouts are re-expressed in original Rust; the one MIT-licensed
  source file in the reference (its PGN tokenizer) may be ported with its notice retained.
