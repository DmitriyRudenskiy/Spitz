# Design

## Context

Spitz already specifies analysis, play, books, training, and a classic desktop shell. What it does
not specify is the layer that power users live in: a notation that can be typed instead of clicked,
rich and graphic annotations, documents that can be published, a library that can be filtered and
tagged at scale, engine resources the user controls and measures, and training built from
positions rather than from games.

Two hard constraints from `openspec/config.yaml` shape every decision below: the chess knowledge
lives in a third-party engine process, and the product is local-only. A third constraint is legal:
reference products and their file formats and content are not part of this project's vocabulary or
its package.

Dependencies: `add-core-platform-mvp` (shell, board, rules core, game library),
`add-analysis-and-training` (engine hosting, evaluation display, coach panel, play sessions),
`add-analysis-power-tools` (deep analysis, book authoring, tablebases, training store).

## Goals

Get the named workflows specified well enough that an implementer can build and test each one
without inventing product behaviour, and that every requirement states a result a tester can
observe.

Deliberately out of scope, with the reason each time:

- Cloud and crowd analysis, hosted engines, engine marketplaces and auction-style engine hiring —
  the product is local-only.
- Accounts, activation, licensing and seat management — there is nothing to license.
- Bundled commercial opening books, encyclopedias, lesson collections, test position sets, video
  lessons and sound banks — the user supplies such files, and this project may not redistribute them.
- Proprietary container, book and database formats, and any feature whose behaviour only exists in
  one vendor's format — the specs name open formats and the app's own documented formats.
- Online play, push notifications, ratings against other people, and community-contributed book
  moves — network features by construction.
- Voice command and dictation — separate input subsystem, separate change.
- Building a chess engine, including any engine-side training, speaking, or explaining ability —
  the engine is a third-party process.

## Decisions

### D1 — Requirements name the app, not the engine

Every requirement in this change makes the app the actor. Where a capability depends on engine
output, the requirement says what the app asks for and what it displays.

**Rejects**: keeping the older "the engine SHALL show/describe/provide" wording inherited from the
baseline specs. **Closes**: requirements nobody can satisfy without shipping a custom engine.

### D2 — Lenient notation decoding with candidate sets

The codec gains a lenient mode that accepts human input (missing or redundant disambiguation,
lower case, omitted capture and promotion marks, both castling spellings, long algebraic) and returns
the set of legal moves a string matches. Ambiguity is resolved by the caller showing candidates; the
codec never picks silently.

**Rejects**: strict SAN-only decoding, and a "guess the best move" fallback. **Closes**: typed-move
input failing on legal but informally written moves — the reported complaint behind the request for
"enter moves by one click and by typed notation".

### D3 — Static position descriptions come from position statistics plus engine lines

The position brief, positional summaries, and move-by-move reasons are generated from deterministic
facts computed by the rules core — material, pawn structure, piece activity, king exposure, capture
and check density, evaluation volatility — combined with the engine's lines. Every emitted statement
has a recorded source; a statement with no source is omitted or labelled as inference.

**Rejects**: asking an engine to explain itself; tuning engine evaluation into prose; shipping a
language model inside the app. **Closes**: a "static evaluation of the game" feature that is only
implementable by inventing our own engine, and confident prose the data does not support.

### D4 — Position temperature is a defined measurement

Temperature is computed from capture and check density in the position and the volatility of the
evaluation around it, calibrated so a fixed quiet corpus scores at most half of a fixed sharp corpus.
It is a number the tests can pin, not a label.

**Rejects**: exposing the engine's score as "how interesting this is", and an unscored heuristic
label. **Closes**: an unmeasurable goal entering the spec, and a training feature with no way to tell
good positions from noise.

### D5 — Annotation elements are anchored to a move or a position and stored as data

Arrows, labelled squares, square groups, and text labels are stored as structured records anchored to
a move or a position, with colour and order, serialized in the documented sidecar namespace inside
PGN comments. The board view draws them; every export applies the view's visibility filters.

**Rejects**: a private binary annotation container, and screen-coordinate drawing (does not survive
resize or export). **Closes**: annotations that vanish or shift on export, and the third-party
container formats this project may not reuse.

### D6 — Media is a sidecar with a manifest, never inside the game file

Spoken commentary is recorded to local audio files referenced by the game, with a content hash and an
anchor-offset table for board synchronization. A media package export writes the game, the media
files, and a manifest listing paths, hashes, durations and offsets. Missing media is reported with
its expected path.

**Rejects**: embedding audio in PGN, and any upload or hosted media. **Closes**: unbounded game files
and unplayable shared games; nothing in the spec needs a network.

### D7 — Engine passes write annotations under a run name

A write-back pass (blunder check, positional report) records a run name, the engine names, and the
settings used, and re-running the same name replaces its own annotations only. Handwritten comments
and symbols are never rewritten.

**Rejects**: appending pass output to existing comments, and clearing all comments before a pass.
**Closes**: double-written annotations after a second pass, and destroyed user commentary.

### D8 — The library keeps checksummed secondary indexes

Player, event, tag, ECO, comment-text, material-signature and position indexes are separate from the
primary store, each with a size, a freshness stamp and a checksum. A missing or mismatched index is
rebuilt, not silently ignored. Every search filter names the index it used; a filter with no index
reports the rows it scanned.

**Rejects**: an embedded SQL engine as the answer to every filter, and a single blob file that must be
scanned. **Closes**: "search feels slow at a million games" being unfixable, and hidden full scans.

### D9 — Tags are capped and run-owned

Tags are free-form with a per-database cap (default 100,000 distinct tags), bulk operations are
single undo steps, and automatic tags created by passes live in a run-owned namespace so they can be
replaced as a set.

**Rejects**: uncapped tags, and treating automatic tags the same as user tags. **Closes**: library
search becoming unusable after repeated analysis passes, and pass-created tags surviving as litter.

### D10 — Library save, restore, and archive share one manifest

Saving a library, restoring it, and archiving it all describe the same component set — games,
repertoires, books, training store, settings, indexes — with per-component sizes and checksums. A
restore either completes or restores the previous state.

**Rejects**: copying whatever files happen to exist. **Closes**: half-restored libraries and silent
data loss.

### D11 — Engine resource control is best-effort and reported

Priority and CPU affinity are applied through the platform scheduler when starting an engine and its
children; the app itself stays at normal priority and keeps one core free. Values the OS refuses are
shown as refused with the OS error text, and the engine starts anyway.

**Rejects**: pretending the setting always works, and killing engine threads to protect the UI.
**Closes**: "the interface froze while engines were searching" being answered by a settings change
that silently did nothing.

### D12 — Hash retention is explicit

An engine's hash is retained between games of a match and between positions of one analysis session,
cleared on request and on new match, and a hash-full report from the engine is surfaced as a suggestion
with the current size. Memory is bounded by the configured hash plus a reported margin.

**Rejects**: clearing the hash on every move, and letting permanent analysis grow memory without
reporting it. **Closes**: pointless re-searching and unexplained memory growth.

### D13 — Benchmarking measures this stack, on this machine

The hardware report measures the rules core (perft), each registered engine (nodes per second at a
fixed setting), tablebase latency, and selection overhead, and writes a machine-readable report with a
machine description and a history for regression comparison. Test position sets reuse the
per-position search behaviour already decided in `add-analysis-power-tools`: UCI cannot restrict the
root move, so a position counts solved when the engine's chosen line contains an expected move.

**Rejects**: publishing advertised benchmark numbers, and a bespoke engine-side test protocol.
**Closes**: a benchmark that cannot be reproduced by the user who ran it.

### D14 — Sessions reuse the play-session runner

Sparring from a position, position-list sessions, and retraining a lost position are configurations of
the existing session runner: a normalized position record (FEN, origin game, ply, label), a configured
opponent, and a per-position result record.

**Rejects**: a second match engine. **Closes**: two places to keep in sync when timing, adjudication,
or persistence changes.

### D15 — Visual assets are vector-first and theme-driven

Shipped piece sets are SVG or 2× raster, drawn through theme colour slots, with at least four
original sets. Icons follow one grid and stroke rule. Every asset ships with a license record, and the
package build fails if an asset has no license entry.

**Rejects**: bundled artwork from other products, mixed raster sizes, and per-piece hardcoded colours.
**Closes**: blurry boards at high DPI, themes that only repaint squares, and trademark exposure.

### D16 — Sounds are files with metadata, not samples from other products

Sound sets are directories of short files plus a manifest naming each event, its duration ceiling and
its default volume; at least one original set ships. Silent mode is a profile, not a volume of zero
buried in settings.

**Rejects**: reusing a commercial product's sounds, and synthesizing everything at runtime.
**Closes**: legal exposure and unthemeable audio.

### D17 — Localized notation is a rendering concern

Piece letters, figurine glyphs, and move-list wording come from locale resources. A missing glyph set
falls back to piece letters with the fallback stated in settings, and no locale is a translation
target for another locale's tests.

**Rejects**: hardcoding English letters in the board and notation widgets. **Closes**: a UI that needs
a code change per language.

## Risks and trade-offs

The lenient decoder can turn a typo into two candidate moves; the mitigation is that the app plays
nothing it is not sure of, which costs an extra click. Secondary indexes double the write cost of
bulk imports; the import budget accounts for it, and indexes may be deferred until an import finishes.
Position statistics are a new deterministic surface — a change in the formulas shifts the coaching
output, so the formulas are versioned and the calibration corpora are fixtures. Media synchronization
drift accumulates over long recordings; the anchor table is editable and the spec pins the tolerance.
Deep positional summaries can overstate certainty; the traceability rule is testable per sentence
template and the templates are the unit of review.

## Migration and rollout

Four waves, each independently testable: (1) input and display ergonomics — notation, one-click entry,
clock column, status colouring, localized labels, sounds; (2) document richness — annotation elements,
symbols, media, publication exports; (3) library scale — tags, indexes, save/restore/archive, player
ratings, book panel and maintenance; (4) power analysis and measurement — evaluation profile, deep
positional pass, write-back passes, resource control, benchmarking, test sets, and position sessions.
No wave requires a data format break: annotation elements, tags, indexes, and media manifests are
additive to what `add-analysis-power-tools` stores, and older libraries load with them absent.

## Open questions

Which audio container ships as the default, and does the platform's decoder set need a fallback
format? Which animated-image encoder stays within the package size and licensing rules? What is the
minimum viable set of piece letters and figurine glyphs for the first two locales? Should the tag cap
be per database or per library, and what is the migration if it is lowered? Can
`sched_setaffinity` be relied on inside the packaging targets, or does the flatpak sandbox need its own
path? Should position temperature feed the tactics generator automatically, or stay a displayed value
until the calibration corpus exists?
