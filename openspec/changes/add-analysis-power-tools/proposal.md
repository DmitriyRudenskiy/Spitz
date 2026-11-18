# Change: add-analysis-power-tools

## Why

M1 and M2 cover the core: play, analysis, training, books, and a hosted-engine GUI. The reference analysis suites this product is measured against stay beloved by correspondence and analysis users for a narrower set of powers the current specs do not cover: Chess960, game histogram and shootout, analysis of selected moves only, batch position analysis, engine matches and tournaments, an arbiter over several engines, WinBoard engines, an endgame explorer, book authoring from scratch, electronic-board integration, and a stronger in-game coach. They are all achievable in the shell and rules layers, which is exactly what Spitz builds.

## What Changes

- Seven new capabilities (deltas under `specs/` in this change):
  - `chess-variants` — Chess960 rules, random starts, variant play/matches/analysis.
  - `humanized-opponent` — strength slider with separate opening knowledge, rated games with measured rating, handicap games, human-shaped errors and behavior (draw offers, resignations), in-game coach aids. Realized through the `move-selection` layer; nothing here requires engine cooperation beyond the option set it declares.
  - `deep-analysis-toolkit` — selected-moves analysis (include/exclude), game histogram, shootout, batch position analysis (EPD/PGN → EPD/CSV), background analysis during the opponent's turn.
  - `multi-engine-orchestration` — up to 10 engines at once, arbiter module, WinBoard hosting, engine match and tournament runners (round robin and gauntlet, start positions from a set), legacy autoplayer compatibility.
  - `tablebase-suite` — multi-format tablebase management, endgame position explorer, evaluation-changing-error flags. All lookups are local files; no online endgame database.
  - `book-authoring` — create/edit/weed opening books without size limits, per-side books in matches, book learning from match results. Books stay local files.
  - `external-boards` — DGT integration and legacy electronic boards.
- One delta to the existing `training-center` capability (MODIFIED: hint ladder gains an attacked-pieces view; ADDED: daily puzzle in three difficulties, selected locally by date seed).
- Removed from scope relative to the original plan: `engine-learning` (a third-party engine's evaluation is not ours to tune; see `openspec/backlog/`). The one part that was ours — book learning from results — moved into `book-authoring`.

## Impact

- Affected specs: seven new capabilities plus training-center (modified/added requirements).
- Affected code: `chess-core` gains a variant flag; `move-select` gains handicap, error-shape, and coach hooks; `uci-host` gains a WinBoard adapter, resource governor, and arbiter service; `game-db` gains EPD/CSV I/O and book-authoring extensions; the GUI gains coach overlays, histogram, and runner dialogs; new crate for electronic-board transports.
- Sequencing: after M2 (it uses the analysis workbench, style engine, and books) and on top of `move-selection` from M1. Every strength or behavior claim in this change is calibrated per registered engine, because the engine is not ours.
