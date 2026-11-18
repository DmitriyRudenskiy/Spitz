# Tasks

## Wave 1 — Input and display ergonomics

- [ ] 1.1 Add lenient notation decoding to `chess-core` (missing and redundant disambiguation, case, capture and promotion marks, both castling spellings, long algebraic) returning candidate moves, with fixture tests over the perft positions.
- [ ] 1.2 Add figurine and piece-letter display encodings behind a locale resource, with the missing-glyph fallback covered by a test.
- [ ] 1.3 Add typed notation entry to the board and notation widgets, wiring the candidate set to an ambiguity chooser so nothing is played on an unsure parse.
- [ ] 1.4 Implement one-click move entry, drag-off cancel, right-click board menu, and mouse-wheel move navigation in `desktop`; cover each with a widget test.
- [ ] 1.5 Add the clock column (time spent, remaining, flag icon) to the notation list with editing while reviewing, and update exported PGN clocks from the edited values.
- [ ] 1.6 Add board appearance settings (orientation, coordinate placement, piece labels, element colours, animation speed) persisted per theme, and the 3:1 contrast guard for custom highlight colours.
- [ ] 1.7 Implement repertoire status colouring with colour-blind-safe defaults, text or shape markers, and the status report; add the contrast assertion to the theme tests.
- [ ] 1.8 Specify-then-build the sound pack system: manifest format, ≥1 original set, per-event volumes, silent profile, device-missing fallback; keep activation latency at or below 250 ms.

## Wave 2 — Document richness

- [ ] 2.1 Define the annotation element model (arrow, labelled square, square group, text label) with move or position anchoring, colour, and order, and its sidecar serialization inside PGN comments.
- [ ] 2.2 Draw the elements in `board-ui` with per-class visibility toggles and per-square clearing, and keep drawing responsive while engines search.
- [ ] 2.3 Implement the annotation symbol picker with NAG mapping, custom symbols, and round-trip tests through PGN.
- [ ] 2.4 Add audio recording per move with local file storage, content hash, replace-take confirmation, and device-failure handling.
- [ ] 2.5 Implement the commentary anchor timeline with editable offsets, automatic board advance, and the 250 ms sync tolerance test.
- [ ] 2.6 Add publication exports: animated image, diagram sheet, and media package with manifest; assert the size, frame-rate, and manifest contents in tests.
- [ ] 2.7 Apply the active view's visibility filters to every image export path in `game-management` so diagrams match the screen.

## Wave 3 — Library scale

- [ ] 3.1 Implement game tags with bulk apply and undo, the distinct-tag cap, rename and delete with counts, and tag search and sorting.
- [ ] 3.2 Move automatic pass annotations into run-owned namespaces with replace-on-rerun semantics, keeping handwritten annotations intact.
- [ ] 3.3 Build the checksummed secondary indexes (player, event, tag, ECO, comment text, material signature, position) with size and freshness reporting and a rebuild pass.
- [ ] 3.4 Report the index used or the rows scanned for every search filter, and add the 1M-game search budget tests.
- [ ] 3.5 Add material-and-turn fragment search with a user-specified material specification, and the fragment import path with duplicate and legality reports.
- [ ] 3.6 Implement library save, restore, and single-file archive over one component manifest with per-component checksums and rollback on failure.
- [ ] 3.7 Add the player rating list (games, highest, average Elo) with completion during search and game entry, and the outlier confirmation over 250 Elo.
- [ ] 3.8 Extend the book panel: ECO and Elo-band columns, strongest-move marker, column toggles and sorting, next and previous book position navigation.
- [ ] 3.9 Add book maintenance operations: remove entries by source or game, Elo limit on import, engine-analysis import with weights, and book-full behaviour with an explicit eviction report.
- [ ] 3.10 Support multiple book sources with configured order, per-context sets, source labelling, and single-source resolution for engine-side book queries.

## Wave 4 — Analysis, measurement, and position training

- [ ] 4.1 Implement position statistics in `chess-core` (material, structure, piece activity, king exposure, density, volatility, phase) with determinism tests, a 2 ms p95 budget, and versioned formulas.
- [ ] 4.2 Build the evaluation profile graph with error and book markers, time and depth overlays, click-to-jump, gap rendering for unanalyzed ranges, and image copy.
- [ ] 4.3 Implement the deep positional pass producing traceable summaries, with a per-sentence source rule and cost shown before the run.
- [ ] 4.4 Implement the blunder check write-back pass: symbols, run comment with engines and settings, run tag, replace-on-rerun, and clean-export support.
- [ ] 4.5 Add notation log columns (evaluation, depth, time, expected move) with documented storage and analysis-free export.
- [ ] 4.6 Add the position brief and position temperature views to the coach panel, with the quiet-versus-sharp calibration fixture.
- [ ] 4.7 Implement per-engine scheduling priority and CPU affinity with applied-versus-refused reporting and the UI latency guard under full engine load.
- [ ] 4.8 Implement permanent analysis with search-state reuse, a move request that always wins, and bounded memory.
- [ ] 4.9 Implement hash lifecycle controls and surface hash-full, tablebase hits, and resource statistics in the engine manager.
- [ ] 4.10 Build the hardware benchmark run, its machine-readable report, run history, and 10 % regression flags.
- [ ] 4.11 Implement test position sets: EPD import with expected-move opcodes, solved and unsolved results per engine, run tags written to the library, and the 95 % reproducibility check.
- [ ] 4.12 Implement position-set matches from a position list with both colours per position, no book by default, per-position scores, and PGN persistence.
- [ ] 4.13 Implement sparring from a position, including reaching the position by playing the line, position-list sessions with resume, and post-loss retraining with attempt records.
- [ ] 4.14 Add the move-by-move post-game commentary and its export, plus the assist recording that feeds it.
- [ ] 4.15 Implement editable and importable style profiles with the 70 % consistency suite and identical-selection tests across installations.
- [ ] 4.16 Implement exercise collections import with discipline and level filters, media-missing reporting, and the 50,000-position import budget.
- [ ] 4.17 Implement the single-discipline training reset with an export-first confirmation and undo from the snapshot.

## Across waves

- [ ] 5.1 Fix inherited requirement subjects so the app, not the engine, owns behaviour, in every requirement this change touches.
- [ ] 5.2 Add the asset pipeline: SVG-first import, colour-slot theming, four original piece sets, icon grid rule, and a license record per asset enforced at package time.
- [ ] 5.3 Add locale resources for the strings, piece letters, and figurine glyphs this change introduces, and run the not-a-stub check over them.
- [ ] 5.4 Update `openspec/specs/` through the archive step for each capability this change modifies, and keep `openspec/README.md`'s layer table current.
