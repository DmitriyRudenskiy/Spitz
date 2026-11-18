# Design: Analysis Power Tools

## Context

The M1/M2 baseline delivers the core suite. This change ports the deep-analysis and humanized-play powers of the analysis-focused reference product, named neutrally throughout: the reference's multi-engine referee becomes the **arbiter module**, its proprietary endgame bases become the **fast tablebase format**, and its book learning is a book feature like any other.

The constraint that shapes every decision here: the engine is a third-party UCI process. Anything the original plan assigned to "the engine" either becomes an option we set, a filter we apply around the engine, or it goes out of scope.

## Key Decisions

1. **Variants first**: `chess-core` gains a `Variant` enum (Standard, Chess960). Castling logic is generalized to position-dependent king/rook squares; FEN serialization uses the extended castling encoding (files, not letters). All higher layers (play, matches, analysis) read the variant from the position, so variant support is automatic above the rules core. Engines join through the `UCI_Chess960` option and are refused with a clear message when they do not declare it.
2. **Humanized opponent lives in `move-select`, not in the engine**: reduced levels combine the level's sampling width, a separate opening-knowledge knob (how much of the book the opponent may use), and, where the engine declares it, `UCI_Elo`/`UCI_LimitStrength`. We do not pretend to inject evaluation noise into a foreign engine — the human shape comes from which of the reported lines get chosen, with within-game variance and no long streaks of best play. Draw offers and resignations are driven by reported evaluation and material thresholds with cooldowns so behavior stays believable.
3. **Coach aids** (opt-in): after each user move a short shallow search runs; a serious blunder triggers a warning with a better move and a takeback offer. Hint arrow and safe-square dots reuse the board overlay layers from board-ui. The check is one extra search per move, so its depth and frequency are user-capped.
4. **Selected-moves analysis is engine-agnostic by construction**: UCI has no standard root-move restriction, so the toolkit analyses a chosen set by running one search per selected move from the position after that move and negating the score, in parallel up to the user's engine budget. When an engine happens to declare a root-move-restriction option the toolkit uses it instead, because it is faster. Excluded moves are filtered from the reported lines. The command is reachable from the menu and toolbar and documented in the manual — a hidden shortcut would not satisfy the discoverability requirement.
5. **Histogram, shootout, batch analysis** are jobs over engines we already host: the histogram runs iterative sweeps over the game (depth grows per pass) until stopped, storing per-move eval/time/depth; shootout spawns a self-play worker that writes a variation into the game document; batch analysis reads EPD and PGN (not proprietary container formats) and writes EPD and CSV plus a live statistics view.
6. **Multi-engine orchestration**: `uci-host` gains a WinBoard adapter and a resource governor (per-engine thread/hash assignment, up to 10 engines). The arbiter module compares candidate moves of two engines and publishes a confidence percentage (agreement of evaluations weighted by depth). Match and tournament runners are state machines persisting games to PGN and a live cross-table (rank, points, performance Elo); tournaments are stoppable and resumable. Legacy autoplayer compatibility targets the de-facto two-PC protocol used by Windows chess programs.
7. **Tablebase suite**: a management and probing abstraction with three backends (fast own format, Nalimov, Syzygy). The app owns discovery and validation and hands the path to engines that probe for themselves; the endgame position explorer probes locally and renders per-move results; an evaluation-changing move (win↔draw↔loss) is flagged by watching the WDL value across user moves. No online endgame lookup: a base the user has not installed is reported as absent, not fetched.
8. **Book authoring**: the M2 book format is already open and indexed; authoring adds create-from-scratch, weed, count, per-side assignment in runners, and book learning that updates local book weights from match results. Book sets stay files the user can copy to another machine.
9. **Engine learning is out of scope**: adjusting a foreign engine's evaluation from lost games is not something we can do to a binary we do not own, and doing it in the selection layer would mean the opponent's "knowledge" is not the engine's play at all. The local store and its reset/size controls move to `book-authoring` as book learning only. See `openspec/backlog/`.
10. **External boards**: a transport abstraction (serial/USB/HID) with a DGT protocol implementation first; legacy boards documented as best-effort. Board sync pushes positions on load and after reconnect.

## Risks / Trade-offs

- Human-like errors are hard to verify, and the ceiling is set by how different the engine's reported lines are. Acceptance uses fixed match distributions (error streak metrics) measured per registered engine, not subjective review.
- Per-move searches in selected-moves analysis cost more than a restricted root search; the budget is capped and the cost is shown before the job starts.
- The arbiter's confidence metric needs calibration; v1 publishes agreement of both engines' top choices weighted by depth.
- WinBoard engines vary widely; the adapter ships with a conformance test set and graceful degradation.
- 10 simultaneous engines can oversubscribe hardware; the resource governor caps total threads at available cores by default.

## Naming (trademark-clean)

arbiter module, fast tablebase format, histogram, shootout, pondering/background analysis, book learning: generic descriptive terms used as-is. No reference-product name appears in specs, code, or UI strings.
