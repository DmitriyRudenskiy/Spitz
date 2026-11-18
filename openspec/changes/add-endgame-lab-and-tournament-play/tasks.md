# Tasks

## Wave 1 — Clocks, claims, and levels (`chess-core`, `move-select`, `desktop`)

- [ ] 1.1 Extend the rules-core clock with per-move time without a game clock, moves-per-period
  controls (1-3 periods, final period all-remaining), per-move spent/remaining records, and
  mode-gated manual correction; add fixture tests for FIDE-style increment-plus-period combinations.
- [ ] 1.2 Expose claim verification (threefold, fifty-move, insufficient material) as explicit
  calls returning accept/refuse with the occurrence or material evidence, and test against drawn
  repetition edge cases.
- [ ] 1.3 Implement adjudication evaluation in `move-select`: engine-margin resign with rules-core
  mateable-construction veto, per-side and off settings, standard PGN termination recording; cover
  the fortress case in a test.
- [ ] 1.4 Define the level catalog as (constraint, mechanism, calibration) triples wired to
  declared options with visible degradation, reusing the `move-selection` mapping requirement;
  add per-engine calibration fixtures for the two reference engines.
- [ ] 1.5 Implement strict game mode as a game-scoped flag read by assists, analysis, clocks, and
  hints, with audit lines in the game file and the unrated downgrade path; test the one-hint
  voiding rule end to end.
- [ ] 1.6 Wire the new controls, adjudication, claims, and strict mode into the new-game dialog and
  game window; keep every control disabled-and-explained rather than hidden mid-game.

## Wave 2 — Book policy (`game-db`, `move-select`, `desktop`)

- [ ] 2.1 Version the native book format: play-value class, gate thresholds, recent-results window
  fields, one-pass migration with backup copy and round-trip tests.
- [ ] 2.2 Implement breadth modes (tournament/standard/wide) and the never-play classes in book
  selection, with the exhausted-position report; distribution tests over a fixed book.
- [ ] 2.3 Implement the play gate (games, wins, per-colour score, last move number) and show the
  failed threshold per move in the book panel.
- [ ] 2.4 Implement the recent-results correction column with its rolling window, lifetime-versus-
  adjusted display, and window-size setting; lifetime columns stay untouched in tests.
- [ ] 2.5 Implement the classic/modern recency bias at three strengths and the 100-position replay
  distribution test proving the shift.
- [ ] 2.6 Implement import run reporting (read/added/merged/rejected-by-reason) and the per-book
  import log; test with a deliberately broken PGN fixture.
- [ ] 2.7 Add class editing to the book editor, the breadth/gate/bias controls to book options,
  and the engine-own-book deferral display in the panel header.

## Wave 3 — Endgame lab (`game-db` probe pool, `desktop`)

- [ ] 3.1 Implement the square sweep job: piece selection, legality filtering, tablebase probing,
  per-square values, EPD export with the definition comment; assert the six-piece/10-second budget on
  reference hardware and record the measured number.
- [ ] 3.2 Implement engine-backed square sweeps with the printed per-square budget and the mixed
  exact/estimated view gate.
- [ ] 3.3 Implement wildcard definitions (fixed/file/rank/board per piece, side to move) and the
  streaming enumeration job with interrupt, partial results, illegal and symmetric-duplicate
  skipping, and examined-count reporting.
- [ ] 3.4 Implement sweep statistics: per-side W/D/L counts and percentages, longest mate with
  selectable example, mutual and one-sided zugzwangs from double probes, with the double-probe rule
  tested on known zugzwang fixtures.
- [ ] 3.5 Implement the result cache keyed by material class, wildcard definition, and backend with
  checksums; a re-run and a compare must hit the cache.
- [ ] 3.6 Implement filters (result, DTM band, zugzwang class), EPD export of filtered sets with
  the definition recorded, and the reload path into batch analysis and the training centre.
- [ ] 3.7 Implement compare-two-sweeps with side-by-side statistics, value-difference listing, and
  no re-run of the first sweep.
- [ ] 3.8 Implement practice sessions from a filtered set: tablebase-perfect defence where covered,
  stated engine fallback otherwise, per-position results in the lab history keyed by material
  class.
- [ ] 3.9 Draw the sweep overlay on the annotation layer: discrete diverging scale on colour slots,
  short-form value text at the contrast minimum, built-in legend; add the theme and contrast tests.
- [ ] 3.10 Build the wildcard definition dialog with corner-arrow scope badges, text alternatives,
  and missing-tablebase material reporting that names paths and covered piece counts.

## Wave 4 — Observation and library plumbing (`uci-host`, `game-db`, `desktop`)

- [ ] 4.1 Implement the capped engine output log with position markers, keyboard scrolling, clear,
  file dump, and whole/last-position clipboard copy; test eviction at the configured cap.
- [ ] 4.2 Implement the protocol transcript tap with timestamped raw lines, panel and file sinks,
  off-by-default, and the 1 percent nps cost measurement wired into the benchmark harness.
- [ ] 4.3 Implement the PV browser: live stepping through the searching engine's line, automatic
  mode, insert-as-variation, and the no-document-mutation test.
- [ ] 4.4 Implement the self-filling move list (book/tablebase/legal modes plus automatic) with
  click-to-play, source header, and the sufficient-not-best labelling for engine answers.
- [ ] 4.5 Implement user position values: value types, storage as documented comment opcodes,
  honouring in full-game reports and lab runs with `user` provenance, and the values editor.
- [ ] 4.6 Implement engine-owned-book deferral per side with panel source reporting, and the
  setting absent for engines without the declaration.
- [ ] 4.7 Implement the EPD record browser: list, preview board, validated raw-record editing with
  unknown-opcode preservation, invalid-row listing, navigation, and hand-offs to lab, batch
  analysis, and training.
- [ ] 4.8 Implement the live game file writer (atomic rewrite within 200 ms, new-game clear) and
  unfinished-game recovery (30 s cadence, restart offer with clocks and engine settings, deletion
  rules); test torn-read safety with a concurrent reader.
- [ ] 4.9 Implement the material balance strip (off/difference/full list) with piece-set rendering
  and screen-reader text; test surplus computation on capture-heavy fixtures.
- [ ] 4.10 Implement the drag-time reply risk preview from the analysis cache within the 150 ms
  budget, strict-mode disabled, hover never starting a search; test the no-cache no-tint path.

## Across waves

- [ ] 5.1 Extend the asset manifest checks so wildcard badges, risk glyphs, legend, and material
  strip art ride the `add-power-user-features` pipeline (colour slots, license record per asset).
- [ ] 5.2 Add locale resources for every string this change introduces and run the not-a-stub
  check on the launch locale (English is the only CI-gated locale at launch; see `config.yaml`).
- [ ] 5.3 Record measured numbers (sweep budget, transcript cost, bias distribution) in the test
  reports they belong to and relax or tighten the spec numbers only with evidence.
- [ ] 5.4 Update `openspec/specs/` through the archive step for each capability this change
  modifies, keep `openspec/README.md`'s reading map and layer table current, and run
  `openspec validate --all --strict` plus the full test suite before each merge.
