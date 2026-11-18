# Descope notes

Why each item below is out of the active spec set, and what it would take to bring it back.
Nothing here is deleted: the requirement text stays as written, so restoring a capability is a
`git mv` back into `specs/` plus a scope review — not a rewrite from memory.

## Removed capabilities

### `chess-engine` — in-house engine, NNUE, training data

The project is a GUI; playing strength comes from third-party UCI engines such as Stockfish.
The capability specified an engine we would have to write, tune, and test (perft-parity
strength checks, NNUE nets and training pipelines, learning-slowdown and resignation behavior,
built-in engine exposure through the protocol layer).

**Where that need went:** engine behavior is now expressed as (a) UCI options the GUI sets and
reads, (b) the move-selection layer that chooses among the lines an engine reports, and (c)
hosting requirements in `engine-protocol`. See `specs/move-selection/spec.md` and
`specs/engine-protocol/spec.md`.

**To restore:** as a separate repository. An engine is a different product with a different
test surface (search, eval, net training) and would hold this GUI's schedule hostage.

### `cloud-sync`, `crowd-analysis`, `online-play`, `accounts-licensing`

The project talks to local processes and local files only. These four capabilities are a
hosted service: accounts and entitlements, device seats and offline grace, a cloud store with
conflict resolution, a crowd-sourced analysis corpus, and rated matchmaking against other
people's accounts.

**To restore:** a server, an operator, a privacy policy, and a decision about what a free
open-source project owes the people who run it. Explicitly: not planned.

### `voice-interface`

Not descoped for legal reasons — speech is a good fit for hands-on-board analysis. Out because
every path in the original spec ran through a cloud account. A local path (on-device recognition
for move notation and commands) is a different capability and could return as a change of its
own; its UI-safety rules (a recognized move never commits without confirmation, results never
stored as audio, offline indication) are worth keeping when it does.

## Removed requirements from kept capabilities

| Capability | Requirement | Why |
| --- | --- | --- |
| `engine-protocol` | Built-in engine exposure | There is no built-in engine. Replaced by "Engine discovery and declared capability". |
| `opening-book` | Live Book Client | Server-delivered book deltas. Replaced by local book export/import in M2. |
| `performance-budgets` (delta) | Analysis speed budget | Specified the engine's nodes-per-second, which is the engine's business. GUI budgets (rules throughput, cold start, memory, input latency) stayed. |
| `local-only-operation` (delta) | Offline degradation, no-server start, read-only mode | Written as graceful degradation of cloud features. Replaced by one requirement: the program opens no sockets. |
| `training-center` | Progress Tracking sync wording, cloud puzzle sets, online daily puzzle | Cloud sources. Daily puzzle is now selected by date seed from local exercise stores. |
| `style-engine` | Style axes as engine search/eval modifications | A foreign engine does not accept "play positionally". Axes are now selection policy over the lines the engine reports, behind a measured-effect gate. |
| `tablebase-suite` (delta) | Online endgame lookup | Remote six-piece lookup. Local multi-format probing and the endgame explorer stayed. |
| `endgame-lab` (delta) | A vendor's proprietary compressed tablebase format as a lab data source | Closed container of unknown provenance. The lab probes whatever open backends are configured; a fast open format plus Nalimov/Syzygy already cover the speed case. |
| `book-authoring` (delta) | On-demand online book query | Remote book service. Local authoring, weeding, per-side books and book learning stayed. |
| `book-authoring` (delta) | Engine learning (was its own capability) | Adjusting a third-party engine's evaluation is not possible from outside, and doing it in the selection layer would stop being the engine's play. Book learning — the part that was always ours — moved into `book-authoring`. |
| `deep-analysis-toolkit` (delta) | Selected-moves analysis "restricted engine search, faster than full" | UCI has no root-move restriction. Now one search per selected move from the position after that move, with cost stated up front, plus an optional engine-declared fast path. |
| `deep-analysis-toolkit` (delta) | Batch input in a vendor's proprietary game container format | Closed container owned by another vendor. EPD and PGN cover the same workflow. |

## Changed identity

- Product name: the working name used by the source spec set is replaced by **Spitz** throughout.
- One analysis-focused change was renamed to `add-analysis-power-tools`; its former name was a
  third-party product name, and the specs themselves already used trademark-neutral terms
  (arbiter module, fast tablebase format, histogram, shootout).
- Milestone numbering: M1 core platform (`add-core-platform-mvp`), M2 analysis and training
  (`add-analysis-and-training`), M3 analysis power tools (`add-analysis-power-tools`), M4 power-user
  surface (`add-power-user-features`), M5 endgame lab and tournament play
  (`add-endgame-lab-and-tournament-play`), plus the shell refinement change
  (`add-ui-enhancements`), which takes no milestone number. The retired online/cloud/voice
  milestone is not numbered.
- `chess-engine` is no longer part of M1, which is why M1 is now four crates plus the shell
  instead of a fifth engine crate.
