# Tasks: add-analysis-and-training

## 1. analysis-workbench

- [ ] 1.1 Infinite analysis panel: eval, PV, MultiPV ≤5, nps, nodes, hash usage; attachable to any board; incremental UI updates
- [ ] 1.2 Full-game report job: move classification (0.5/1.0/2.0-pawn configurable thresholds), accuracy %, key moments; streams results
- [ ] 1.3 Tactical fast pass: blunder-only, ≥10x faster than the full report at defaults
- [ ] 1.4 Monte Carlo analysis with playout budget, caching, and merge across re-runs
- [ ] 1.5 Engine comparison view: ≥2 engines synchronized on one position with disagreement highlighting
- [ ] 1.6 Key-moments navigator ordered by eval swing; report-to-annotations export

## 2. opening-book

- [ ] 2.1 Book format (documented) with count/score%/avg-Elo/last-used per move; writer and reader
- [ ] 2.2 Import from PGN database (≥1M positions) with duplicate merge; progress reporting
- [ ] 2.3 Book panel with statistics during play and analysis; weighted selection modes (frequency/Elo/performance)
- [ ] 2.4 Book sets are plain local files: import, export, copy between machines by hand; each set carries a content hash recorded in games that used it
- [ ] 2.5 Book editor: add/rename/penalize moves, comments, transposition awareness

## 3. play-vs-computer

- [ ] 3.1 New-game dialog covering opponent/level/time control/color/handicap in one place
- [ ] 3.2 Adaptive strength with ±150 Elo bands and ≤1 level drift per 20-game window
- [ ] 3.3 Time-control presets (1+0 … 90+30) plus custom; flag fall integration
- [ ] 3.4 Assists: eval bar, hints, takeback count, blindfold mode
- [ ] 3.5 Post-game summary linked into the full report and training suggestions

## 4. style-engine

- [ ] 4.1 Style axes (aggression, risk, exchange orientation, positional bias) applied as selection weights in `move-select`, each with a measured effect size on the calibration suite
- [ ] 4.2 ≥12 named presets selectable per game or session
- [ ] 4.3 ≥5 personas derived from public-domain corpora with neutral configurable names, style vectors, phrase banks (≥50 phrases each)
- [ ] 4.4 Live reactions on eval swings ≥1.5 pawns and tactical shots, rate-limited
- [ ] 4.5 Archetype behaviors: attacker, coward, swindler, endgame wizard

## 5. training-center

- [ ] 5.1 Hint ladder (theme → candidates → full explanation), never auto-playing the solution
- [ ] 5.2 Tactics generator from user games + themed sets; ≥100 exercises per theme from a 1000-game database
- [ ] 5.3 Calculation training modes (assisted/evaluated/tactical) with per-mode rating 800–2800
- [ ] 5.4 Endgame course: 27 modules with drill loop and pass criteria
- [ ] 5.5 See&Click 60-second bursts with score history
- [ ] 5.6 Blitz & Train PDF worksheets from blitz games (exercises + solutions page)
- [ ] 5.7 Blindfold trainer with progressive piece hiding; progress ratings, charts, streaks

## 6. opening-trainer

- [ ] 6.1 Repertoire tree per color, built from moves/games/book
- [ ] 6.2 Drill loop vs engine with deviation feedback and repertoire statistics
- [ ] 6.3 Deviation statistics and refocus suggestions; spaced-repetition scheduling
- [ ] 6.4 PGN-based import/export of repertoire

## 7. local-only-operation delta

- [ ] 7.1 Feature matrix test: analysis, play, books, tablebases, training all pass with no network route
- [ ] 7.2 Process monitor asserts no listening socket and no outbound connection during a scripted full-feature session
- [ ] 7.3 Search the codebase and packaging for telemetry, update-check, and crash-report code paths; none may ship
