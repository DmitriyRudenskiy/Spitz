# Change: add-core-platform-mvp

## Why

The repository is greenfield: no code exists. Analysis, training, and database features all depend on a verified rules core, reliable hosting of third-party chess engines, a playable board UI, an application shell, and game/database management. This change establishes that foundation (Milestone M1) and produces the first installable product.

Spitz does not implement a chess engine. Playing strength comes from third-party UCI engines (Stockfish and peers); everything the product adds on top of an engine — strength levels, style, human-like choice, resign/draw behavior — is implemented in a move-selection layer inside the app (see the `move-selection` baseline spec and the `add-analysis-power-tools` change).

## What Changes

- Implements the baseline capabilities `chess-rules-core`, `engine-protocol`, `board-ui`, `app-shell`, `game-management` (requirements already seeded in `openspec/specs/` as the spec-first baseline — no capability deltas needed for them).
- Implements the `move-selection` baseline: the layer that turns engine output plus UCI options into the opponent's move (levels, epsilon sampling, seeded reproducibility).
- Adds one new engineering capability via delta: `performance-budgets` (rules-core throughput, cold start, and memory budgets that CI enforces) — see `specs/performance-budgets/spec.md` in this change.
- Creates the Rust workspace layout, CI, and Linux packaging used by later milestones (M2: analysis/training; M2.5: analysis power tools).

## Impact

- Affected specs: new baseline capabilities chess-rules-core, engine-protocol, board-ui, app-shell, game-management, move-selection; new capability performance-budgets (delta in this change).
- Affected code: all of it (greenfield). Introduces crates: `chess-core`, `uci-host`, `move-select`, `game-db`, `desktop` (Tauri shell). No engine crate, no server crate.
- First-run experience: the app detects common engines on `PATH` and in standard install directories and offers to register one; without any registered engine, browsing and editing games stay available and analysis/play are disabled with an actionable message.
