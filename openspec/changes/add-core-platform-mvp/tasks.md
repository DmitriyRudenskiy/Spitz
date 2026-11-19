# Tasks: add-core-platform-mvp

## 1. Workspace and CI

- [ ] 1.1 Scaffold Rust workspace with crates `chess-core`, `uci-host`, `move-select`, `game-db`, `desktop`; rustfmt and clippy configs committed
- [ ] 1.2 CI pipeline builds and tests on Linux (reference) and macOS; perft job and budget job run per push and record artifacts
- [ ] 1.3 Tauri app boots to an empty workspace; ts-rs type generation wired into the build

## 2. chess-rules-core

- [x] 2.1 Position type with FEN parse/format; property tests prove round-trip fidelity and malformed-FEN errors. `chess-core/src/fen.rs`: nine tests cover the start FEN, a round trip over four positions, both clocks, en-passant normalisation, and thirteen malformed inputs each refused by what is wrong with them. The coverage is example-based — a property-test framework is not a dependency of this workspace yet, and adding one is its own decision.
- [x] 2.2 Legal move generation (castling, en passant, promotions, pins); perft fixtures match start position d6 and Kiwipete d5. `chess-core/src/move_gen.rs` generates pseudo-legal moves and keeps the ones `Board::apply` accepts, so legality is decided once. Six published fixtures pass to depth 3 in every build and to depth 4-5 in a release build (Kiwipete depth 5 = 193,690,690); start depth 6 = 119,060,324 runs in the ignored throughput test, which measures 29.2 M/s against a 30 M/s budget — under it, recorded in `docs/benchmarks/budget-results.md`. Finding: castling out of check was accepted by `Board::apply`; it is refused now, with a test.
- [x] 2.3 Game state detection: checkmate, stalemate, 50-move, threefold repetition, insufficient material, dead position. `chess-core/src/state.rs` keeps the two kinds apart: `outcome` reports what the position forces (checkmate, stalemate, dead position) and `claim` reports what a player asks for (the hundredth halfmove, the third repetition), so a result reaches an arbiter, a GUI or a database without anyone guessing which kind it was. `Repetition` counts position keys — placement, side to move, castling rights, capturable en-passant — and is cleared by a pawn move or a capture; ten tests cover each rule, including that a lost castling right is a different position. `dead_position` is `insufficient_material` and no more: bare kings, one minor against a bare king, same-colour bishops against a bare king, with the general mating-network proof left as a marked ceiling. Finding: the board stored en passant only when a capture was possible, which is what the position key counts but not what a FEN writes — engines name the jumped square unconditionally, so parsing and re-writing one lost the field. The board now stores the square as notated and answers `en_passant()` by asking whether a pawn stands where it could take, which keeps the key, the generator and the string each doing their own job.
- [x] 2.4 SAN and UCI codecs with disambiguation tests; PGN reader/writer with variations, comments, NAGs, multi-game streams. `chess-core/src/san.rs` writes the disambiguation the position asks for — file, rank, or the pair when neither alone identifies the mover — with the plus or hash on the end, and reads back what a score sheet writes; `uci.rs` writes long algebraic with the promotion suffix. `pgn/game.rs` decodes a game into a flat main line with each variation hanging off the move it branches from, every move carrying its glyphs, NAGs and comments, and writes the score sheet back with the move numbers recomputed from the position; `pgn::split_games` cuts a stream at the termination markers and faults on a variation that never closes, so both halves of an unbalanced variation report the line they happened on and the games before the fault still come back. Twelve PGN scenarios and four codec scenarios in `chess-core/tests/`; the codec test replays every legal move within reach of the six perft fixtures through both codecs, and the three-queen disambiguation is checked against what python-chess writes for the same moves. Findings: the lexer accepts any word as a move, so an unknown word is reported by the decoder as an unplayable move with its line rather than by the lexer as a bad token; a game whose move text never reaches a termination marker takes its result from the Result tag; a glyph writes itself tight against its move, not as a word of its own; and the en passant field follows the spec's "a double move just happened" rule, which is why it says c3 after 1.d4 Nf6 2.c4 where python-chess leaves the field empty.
- [x] 2.5 Clock primitives for none/sudden death/increment/delay with flag-fall events. `chess-core/src/clock.rs` makes the control an enum rather than a bag of optional numbers, because the rules differ: an increment arrives after a completed move, a delay charges only the time past the grace, sudden death charges everything, and `None` is a control and not a missing clock. The caller owns the stopwatch and tells the clock what the move has taken so far, twice over: `time_taken` answers what the display would show and commits nothing, `complete_move` commits the same number, credits what the control credits, and passes the turn. A flag is reported the instant a bank reaches zero, naming the player whose it was, and the clock counts nothing after it. Six scenarios in `chess-core/tests/clock_spec.rs`. Finding: the display path credited the increment until the spec test caught it, which is the same mistake as crediting it before the move is completed. Per-move time records, moves-per-period and manual correction are the endgame-lab delta to this requirement and are not built.

## 3. move-selection

- [x] 3.1 Selection record type: chosen move plus reason (book, level cap, epsilon sample, handicap, resign, draw offer) and the seed, persisted into game metadata. `move-select/src/lib.rs` carries the move, the reason, which line of the report it was, the engine's own first line beside it for later comparison, the seed, the level, the engine id and the book set's hash. A resignation and a draw offer are decisions without a move, so the move is an option and those two constructors still take the report — the record of a resignation should still show what the engine thought when its side gave up. A book move has no engine line, because no engine was asked, and that is recorded as an absence rather than as the book move pretending to be the engine's preference. Writing the record into a game's metadata is game-management's write path and is not built.
- [x] 3.2 Epsilon sampler over MultiPV scores with seeded RNG; unit tests prove determinism for a fixed seed and monotonic error rate as epsilon rises. One integer coin per decision, drawn only when there is a choice to make, and when it says look away a weighted pick over the report with weight `temperature / (temperature + loss)` — the best line the most likely, a near-equal line nearly as much, a blunder thin, which is the difference between a weak opponent and a random one. All of it integers: the level tables are frozen in the source as basis points and centipawns, so no float decides a move. The generator is SplitMix64 owned by this crate, because a ladder match replayed next year has to reproduce move for move and a dependency is free to retune its algorithm in a point release; `SeedStream` hands out one seed per decision from the game's seed, so a game replays by replaying its stream. Twelve scenarios in `move-select/tests/move_select_spec.rs`: the same seed replays the same moves in the same order, the error rate rises strictly across all twenty levels, and the same blunder thins out as the level rises. Findings: the first spread test claimed the weak level reaches more distinct lines, which with a four-line report is the same four buckets at any level — the honest measure is the rank the choices average; and the weakest level plays the report's worst line about one decision in eight, which is a tuning question for the ladder harness and not a shape question, so the test asserts the shape and leaves the rate to calibration.
- [x] 3.3 Option mapper: translate a level into MultiPV/depth/nodes/`UCI_Elo`/`UCI_LimitStrength` when declared, degrade to depth and node caps when not. `EngineOptions` is built from the handshake rather than from a list of engines this crate would have to keep up with; `plan_for(level)` returns the settings to send and the `Mechanism` they realize — Elo-capped, depth and nodes, or sampling alone when the engine declared nothing that slows it down. Elo is clamped into the range the engine declared, so an engine that refuses 3200 is asked for the most it accepts and the mechanism reports what was asked rather than what the level wanted; the two are never mixed, because an Elo cap and a search cap together answer the question twice. `MultiPV` width comes from the level, wide at the weak end and one line at the strong end where nothing is sampled, and is left at one when the engine has no MultiPV — which leaves the sampler one line and the mechanism says so.
- [ ] 3.4 Book gate: book lookup runs before the engine and can be limited independently of strength
- [ ] 3.5 Strength ladder: ≥20 levels; nightly 1000-game adjacent-level matches prove ≥ 55% separation per registered engine; per-push 100-game smoke ladder
- [ ] 3.6 Material handicap and evaluation-noise modes wired into selection and recorded in game metadata

## 4. engine-protocol

- [ ] 4.1 UCI handshake compliance test suite passes against 3 third-party engines in CI
- [ ] 4.2 Engine manager: add/remove executables, option synchronization, health check, crash auto-restart restoring position and options, per-session logs
- [ ] 4.3 Engine autodetection scans `PATH` and common install directories, offers registration, and reports engines that fail the handshake as unusable
- [ ] 4.4 First run with no registered engine: play and analysis controls disabled with an actionable message; browsing, editing, and database features work

## 5. board-ui

- [ ] 5.1 2D board component: 4 piece sets, 8 themes, coordinates toggle, DPI-resilient rendering
- [ ] 5.2 Input: drag-and-drop and click-click, legal-move hints, last-move and check highlights, premove buffer
- [ ] 5.3 Annotation overlay: arrows, square highlights, move glyphs persisted in the game document
- [ ] 5.4 Accessibility: full keyboard play, ARIA live move announcements, colorblind-safe palettes
- [ ] 5.5 Sounds with per-event volumes and master mute; move animation ≤ 180 ms

## 6. app-shell

- [ ] 6.1 Docking system: drag/resize/pin/float panels; 5 named layout slots saved and restored across restarts
- [ ] 6.2 Light/dark themes with custom accent; i18n catalogs for EN/DE/RU with hot locale switch
- [ ] 6.3 Settings model: persistence, per-section reset, import/export; full shortcut remapping with cheat-sheet overlay
- [ ] 6.4 Multi-board tabs (≥8) with isolated state; window layout restore

## 7. game-management

- [ ] 7.1 Game document model: moves, nested variations, comments, NAGs, glyphs, embedded diagrams, unlimited undo/redo
- [ ] 7.2 Streaming PGN import/export with variation/comment round-trip tests on 100-game fixture files
- [ ] 7.3 Native library format (documented) with writer/reader; 1M-game header index builds ≤ 10 min on reference hardware
- [ ] 7.4 Search: header filters < 5 s and position search < 60 s over 1M games
- [ ] 7.5 Virtualized game list: sort/filter interactions < 100 ms at 100k rows; duplicate detection with merge guidance
- [ ] 7.6 Print and PDF export with diagrams; clipboard copy of PGN, FEN, and diagram image

## 8. Packaging and budgets (performance-budgets delta)

- [ ] 8.1 Installers: AppImage and .deb (reference), DMG where a runner exists; cold start to first interactive board ≤ 3 s
- [ ] 8.2 `bench` subcommand records perft throughput, cold start, and resident memory in CI artifacts; budgets from the delta enforced as CI gates
- [ ] 8.3 CI asserts the binary opens no network listener and makes no outbound connection during a scripted session (local-only guarantee)
