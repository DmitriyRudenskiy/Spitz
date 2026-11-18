# Tasks: add-analysis-power-tools

## 1. chess-variants

- [ ] 1.1 `Variant` enum through chess-core; generalized castling (position-dependent king/rook squares); FEN extended castling encoding round-trip
- [ ] 1.2 Chess960 start-position generator: exactly 960 valid positions, seedable, uniform
- [ ] 1.3 Play vs computer and training from Chess960 starts; perft fixtures for Chess960 positions
- [ ] 1.4 Engine matches/tournaments and analysis tools accept Chess960; UCI_Chess960 option negotiation with graceful rejection

## 2. humanized-opponent

- [ ] 2.1 Strength slider (beginner→grandmaster) + separate opening-knowledge knob + full-force option, expressed as move-selection settings and declared engine options
- [ ] 2.2 Rated and casual games; Elo measurement with rating history ("my results")
- [ ] 2.3 Opponent selection: fixed level, "opponent with my strength"
- [ ] 2.4 Auto-adjust strength per game result (bounded, disableable)
- [ ] 2.5 Handicap games: material odds, time odds, combined options
- [ ] 2.6 Human-shaped selection: within-game variance, no perfect-play streaks; calibration suite per registered engine
- [ ] 2.7 Draw offers and resignations by thresholds with cooldowns; continue-after-win dialog
- [ ] 2.8 Coach aids: blunder warning + takeback offer, hint arrow, safe-square dots (shallow search, user-capped)

## 3. deep-analysis-toolkit

- [ ] 3.1 Selected-moves analysis: Ctrl+move include, Shift+move exclude; menu/toolbar command; per-move search jobs with cost shown before start; engine fast path when the option is declared
- [ ] 3.2 Histogram: iterative full-game sweeps, stoppable; show move times/search depths; copy to notation
- [ ] 3.3 Shootout: self-play continuation from chosen position into a variation
- [ ] 3.4 Batch position analysis: EPD/PGN inputs; live stats; EPD+CSV export; stoppable
- [ ] 3.5 Background analysis during opponent's turn with resource cap and disable

## 4. multi-engine-orchestration

- [ ] 4.1 Up to 10 engines on one position; per-engine thread/hash assignment; synchronized updates
- [ ] 4.2 Arbiter module: two engines + confidence 0–100% in its own window
- [ ] 4.3 WinBoard adapter with conformance test set
- [ ] 4.4 Engine match runner: per-side books, hash, time control, games count, book learning, background analysis; PGN persistence; live scoreboard
- [ ] 4.5 Engine tournament runner: round-robin ≤10 engines; live cross-table (rank/points/perf Elo); stoppable/resumable
- [ ] 4.6 Legacy autoplayer protocol compatibility (two-PC matches)

## 5. tablebase-suite

- [ ] 5.1 Tablebase management and probing abstraction: own fast format + Nalimov + Syzygy backends; path handed to engines that declare probing
- [ ] 5.2 Endgame position explorer: result after every legal move, winning/drawing continuations, distances
- [ ] 5.3 Evaluation-changing-error flagging (WDL delta watch)
- [ ] 5.4 Endgame oracle search: side to move, result filters incl. mate-in-N, ignore-symmetric option

## 6. book-authoring

- [ ] 6.1 Create book from scratch; add positions/moves from board or games; ≥1M-position books
- [ ] 6.2 Edit/extend/penalize/delete moves; import games/current game; export; Weed and Count utilities
- [ ] 6.3 Per-side book assignment with Use-Book toggles in match/tournament runners
- [ ] 6.4 Selection parameters (frequency/Elo/performance) applied per book
- [ ] 6.5 Book learning during matches/tournaments (local weights updated by results), visible in the book editor

## 7. external-boards

- [ ] 7.1 Transport abstraction (serial/USB/HID) + DGT protocol implementation; setup dialog with port detection
- [ ] 7.2 Two-way move flow: board→app input, app→board display; position push on load and reconnect
- [ ] 7.3 Legacy board support documented as best-effort

## 8. training-center delta

- [ ] 8.1 Attacked-pieces view for puzzles (MODIFIED hint ladder)
- [ ] 8.2 Daily puzzle in three difficulties, selected by date seed from local stores (ADDED)

## 9. Validate

- [ ] 9.1 openspec validate add-analysis-power-tools --type change --strict
- [ ] 9.2 Full test suite green (Rust + Web)
