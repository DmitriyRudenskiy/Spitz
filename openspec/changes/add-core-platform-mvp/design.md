# Design: Core Platform MVP

## Context

Greenfield project with a fixed stack (see `context` in `openspec/config.yaml`): Rust core, Tauri desktop shell, TypeScript + React GUI, Linux as the primary target. M1 must deliver a usable product — play against a registered engine and analyze locally, edit games, manage a database — with no network access at all.

## Goals / Non-Goals

- Goals: perft-verified rules core; UCI hosting of third-party engines with a real move-selection layer on top; first-class 2D board UI; PGN interchange and a native game database; Linux installer.
- Non-Goals: writing a chess engine (out of scope permanently — see `openspec/backlog/specs/chess-engine/`), 3D board polish (completes in M2), any online or account feature, training content, voice.

## Architecture

Rust workspace, one crate per concern:

- `crates/chess-core` — position, move generation, legality, game state, FEN/SAN/PGN/UCI codecs, clocks. No I/O dependencies; perft and property tests are release gates.
- `crates/uci-host` — UCI client: spawn engine processes, option synchronization, async `info` parsing, cancellation, per-session logs. The app never links an engine; it only ever speaks UCI.
- `crates/move-select` — the opponent's move is chosen here, not by the engine. Inputs: MultiPV output, engine option set, configured level/style/persona, opening book hit, seeded RNG. Outputs: the move to play plus the recorded reason (book, ladder cap, epsilon sample, handicap constraint).
- `crates/game-db` — native, documented open game-database format (header index + position index), streaming PGN import/export, search.
- `app/desktop` — Tauri shell; UI in `app/ui` (React + TypeScript, Canvas 2D board; WebGL reserved for the 3D board).

Data flow: the UI talks to the local Rust core via typed Tauri commands (TS bindings generated with ts-rs). Engines run on worker threads; the UI receives incremental info events over a command/event bridge.

## Key Decisions

1. **No engine crate; engines are external processes.** Strength, evaluation quality, and tablebase probing are the engine's job and are configuration targets for us, not features we build. This removes the largest schedule risk in the original plan (NNUE training) and makes every "engine capability" requirement into an option-mapping requirement.
2. **Move selection is a first-class layer.** Stockfish gives us MultiPV, scores, `MultiPV`, `UCI_Elo`/`UCI_LimitStrength`, `Skill Level` where present, depths and node caps. It does not give us 20 calibrated levels, human-like error shapes, or style vectors. Those are produced by choosing among the reported lines (epsilon sampling with a seeded RNG) and by gating the opening through the book, so the behavior is testable without touching the engine.
3. **Everything the opponent does is reproducible.** The seed, level, style vector, book file version, and engine id are written into the game as metadata, so a 1000-game ladder regression match and any reported "the opponent blundered" case can be replayed exactly.
4. **Own open database format** instead of reverse-engineering proprietary binary formats; PGN remains the interchange standard.
5. **ts-rs code generation** for Rust-to-TypeScript types — no hand-written bindings to drift.
6. **Perft fixtures as release gates** — start position depth 6 = 119,060,324 nodes and Kiwipete depth 5 = 193,690,690 nodes run in CI on every push.

## Risks / Tradeoffs

- **Engine dependence.** The product needs at least one registered engine to play or analyze. Mitigation: engine autodetection at first run with a download link (a human action, not app traffic), clear disabled states, and a compliance test suite that runs against several real engines in CI.
- **Option coverage varies by engine.** `UCI_Elo`, `Skill Level`, and contempt-style options are not universal. Mitigation: `move-select` degrades to depth/node caps plus epsilon sampling when an option is absent, and the ladder calibration is measured per registered engine (see `move-selection` spec).
- **Calibration cost.** A 20-level ladder with a ≥55% adjacent separation gate is 19 x 1000 games per engine build. Mitigation: nightly job, not per-push; per-push CI runs a 100-game smoke ladder with a looser band.
- **WebView differences across OS (Tauri).** Linux is the reference platform; other platforms run the same smoke suite where a runner exists.

## Migration / Rollout

Not applicable (greenfield). Packaging: AppImage and .deb for Linux (reference), other platforms later. Cold start and memory budgets are tracked by the `performance-budgets` delta in this change.
