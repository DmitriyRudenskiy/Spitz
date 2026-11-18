# Change: add-analysis-and-training

## Why

With the M1 platform in place, the product must deliver the reason people choose a chess suite: deep analysis and a serious training system, plus a rich playing experience against styled, human-like opponents.

## What Changes

- Implements the baseline capabilities `analysis-workbench`, `opening-book`, `play-vs-computer`, `style-engine`, `training-center`, `opening-trainer` (requirements already seeded in `openspec/specs/`).
- Adds one delta capability: `local-only-operation` — the guarantee that nothing in the product needs a network, an account, or a service — see `specs/local-only-operation/spec.md` in this change.
- Delivers style axes, presets, personas, and archetypes as behavior of the move-selection layer (line selection plus repertoire), not as engine internals, and completes the 3D board polish deferred from M1.

## Impact

- Affected specs: six new baseline capabilities listed above; new capability local-only-operation (delta); `opening-book` loses its server-synced live book; `style-engine` style axes are realized through `move-selection`.
- Affected code: `move-select` (style vector and persona application), UI (new workspaces, 3D board), new training/exercise store (local SQLite).
