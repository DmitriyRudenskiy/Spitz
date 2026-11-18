# OpenSpec

Spec-driven workflow for Spitz. Specs say **what** the product does; changes propose a delta to
those specs; tasks say **how** to build it. Everything here is authored before the code, so
implementations can be reviewed against a written promise instead of against a screenshot.

Commands: `/opsx:explore` (read-only thinking), `/opsx:propose` (new change), `/opsx:apply`
(work through tasks), `/opsx:update` (revise a change), `/opsx:sync` (fold a delta into main
specs), `/opsx:archive` (finish it). Run `openspec list` for active changes and
`openspec validate --all --strict` before committing anything under this directory.

`config.yaml` carries the project context (product boundaries, stack, language) and the authoring
rules injected into every artifact. Read it before writing a spec: the boundaries decide most
design questions, and the rules decide most review comments.

## Reading map

**Understand the product in five minutes**
`specs/app-shell` → `specs/board-ui` → `specs/engine-protocol` → `specs/move-selection`

**Play and improve**
`specs/play-vs-computer` (levels, ladders, time controls) → `specs/style-engine` (personas) →
`specs/training-center` (tactics, endgame drills, progress) → `specs/opening-trainer`

**Analyse**
`specs/analysis-workbench` (multipv, eval, engine chat) → `specs/engine-protocol` (hosting,
options, resources) → `specs/opening-book` (local books) → `specs/game-management` (documents,
PGN, the store)

**Research endgames and run rated games**
`changes/add-endgame-lab-and-tournament-play/specs/endgame-lab` (wildcard sweeps, oracle
statistics, practice against tablebase defence) → `specs/chess-rules-core` (period clocks,
claims) → `changes/add-endgame-lab-and-tournament-play/specs/play-vs-computer` (level catalog,
adjudication, strict mode)

**Work like a power user**
`changes/add-power-user-features/specs/game-commentary` (graphics, symbols, spoken commentary,
publication exports) → `specs/game-management` (tags, indexes, backup cycle) →
`changes/add-power-user-features/specs/engine-benchmarking` (hardware report, test position sets,
position-set matches) → `specs/move-selection` and `specs/style-engine` (how the app picks from
engine lines)

**Know the limits before proposing anything**
`backlog/descoped.md` — what used to be in scope, why it left, and what it would take to return.

## Where the layers live

| Layer | Rust crate / surface | Specs |
| --- | --- | --- |
| Rules | `chess-core` | `chess-rules-core` |
| Engine hosting | `uci-host` | `engine-protocol` |
| Move choice on top of engine output | `move-select` | `move-selection`, `style-engine`, `play-vs-computer` |
| Storage | `game-db` | `game-management`, `opening-book` |
| Document richness | `game-db`, `desktop` | `game-commentary` (proposed in `changes/add-power-user-features`) |
| Shell and rendering | `desktop` | `app-shell`, `board-ui`, `analysis-workbench`, trainers |
| Measurement | `uci-host`, `chess-core`, `desktop` | `engine-benchmarking` (proposed in the same change) |
| Endgame research | `game-db` probe pool, `desktop` | `endgame-lab` (proposed in `changes/add-endgame-lab-and-tournament-play`) |

Two rules fall out of that table and cause most design arguments. The engine is a foreign process:
behaviour we want is either a UCI option the engine declares or something the selection layer does
itself, so every strength claim is calibrated per engine, and any sentence the app writes about a
position comes from rules-core facts plus engine lines rather than from the engine's prose. The
network does not exist: features that would have fetched data say what they do with the local file
instead.

## Backlog

`backlog/` is not an OpenSpec artifact directory and `openspec validate` ignores it. It holds the
requirement text of capabilities that left scope (specs, and one whole milestone change) plus
`descoped.md`. Restoring something is a `git mv` back into `specs/` or `changes/` followed by a
scope review — the reasoning stays here so nobody has to re-litigate it from memory.
