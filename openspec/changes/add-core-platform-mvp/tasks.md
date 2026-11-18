# Tasks: add-core-platform-mvp

## 1. Workspace and CI

- [ ] 1.1 Scaffold Rust workspace with crates `chess-core`, `uci-host`, `move-select`, `game-db`, `desktop`; rustfmt and clippy configs committed
- [ ] 1.2 CI pipeline builds and tests on Linux (reference) and macOS; perft job and budget job run per push and record artifacts
- [ ] 1.3 Tauri app boots to an empty workspace; ts-rs type generation wired into the build

## 2. chess-rules-core

- [ ] 2.1 Position type with FEN parse/format; property tests prove round-trip fidelity and malformed-FEN errors
- [ ] 2.2 Legal move generation (castling, en passant, promotions, pins); perft fixtures match start position d6 and Kiwipete d5
- [ ] 2.3 Game state detection: checkmate, stalemate, 50-move, threefold repetition, insufficient material, dead position
- [ ] 2.4 SAN and UCI codecs with disambiguation tests; PGN reader/writer with variations, comments, NAGs, multi-game streams
- [ ] 2.5 Clock primitives for none/sudden death/increment/delay with flag-fall events

## 3. move-selection

- [ ] 3.1 Selection record type: chosen move plus reason (book, level cap, epsilon sample, handicap, resign, draw offer) and the seed, persisted into game metadata
- [ ] 3.2 Epsilon sampler over MultiPV scores with seeded RNG; unit tests prove determinism for a fixed seed and monotonic error rate as epsilon rises
- [ ] 3.3 Option mapper: translate a level into MultiPV/depth/nodes/`UCI_Elo`/`UCI_LimitStrength` when declared, degrade to depth and node caps when not
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
- [ ] 7.3 Native database format (documented) with writer/reader; 1M-game header index builds ≤ 10 min on reference hardware
- [ ] 7.4 Search: header filters < 5 s and position search < 60 s over 1M games
- [ ] 7.5 Virtualized game list: sort/filter interactions < 100 ms at 100k rows; duplicate detection with merge guidance
- [ ] 7.6 Print and PDF export with diagrams; clipboard copy of PGN, FEN, and diagram image

## 8. Packaging and budgets (performance-budgets delta)

- [ ] 8.1 Installers: AppImage and .deb (reference), DMG where a runner exists; cold start to first interactive board ≤ 3 s
- [ ] 8.2 `bench` subcommand records perft throughput, cold start, and resident memory in CI artifacts; budgets from the delta enforced as CI gates
- [ ] 8.3 CI asserts the binary opens no network listener and makes no outbound connection during a scripted session (local-only guarantee)
