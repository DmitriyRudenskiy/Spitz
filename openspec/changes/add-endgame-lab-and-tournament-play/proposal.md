# Change: add-endgame-lab-and-tournament-play

## Why

A second feature inventory — taken from the public user manual of the engine-hosting reference
product that pioneered the UCI protocol (54 pages, harvested into reading notes kept outside this
repository) — was diffed against the twelve baseline capabilities and the five pending changes.
Most of its surface is already specified (engine matches and tournaments, an arbiter module,
multi-engine panels, Chess960, electronic boards, tablebase hosting, handicap and rated play,
opening-book authoring, game histograms). What survived the diff is the research-and-referee half
of a serious chess program: asking the endgame tables questions instead of moves, running
tournament-grade games with real time controls, claims and adjudication, keeping book policy under
the user's thumb, and seeing everything the engines are doing. All of it is local, GUI-side work.

## What Changes

- **One new capability**
  - `endgame-lab` — endgame research on top of the installed tablebases: a square sweep that takes
    one piece off the position and scores every square it could stand on; a wildcard sweep where
    each piece is pinned to a square, a file, a rank, or the whole board and every combination is
    enumerated; statistics over the enumerated set (percentages, longest mate, mutual zugzwangs)
    with filters; EPD export of any filtered subset; a compare-two-setups view; and a practice mode
    that plays the filtered positions against tablebase-perfect defence.
- **Tournament-grade play** (`play-vs-computer`, `chess-rules-core`): a catalog of named level types
  (fixed depth, nodes, exact and average time per move, Elo cap, period controls) that degrade
  honestly when an engine lacks the option; period-based and multi-period match clocks with per-move
  time records and manual clock setting; engine-threshold resign and material-checked draw
  adjudication with standard PGN termination fields; verified draw claims (threefold, fifty-move,
  insufficient material); and a strict game mode that switches every assist off and marks the game
  rated only if nothing was touched.
- **Book policy** (`opening-book`): per-move play-value classes with tournament/standard/wide
  selection breadth, play gates (minimum games, wins, per-colour score, last move number) that
  explain themselves in the book panel, a recent-results correction that demotes refuted book
  moves, a classic/modern recency bias, and import runs that report and log every rejected record.
- **Engine observation** (`analysis-workbench`, `engine-protocol`): a capped engine output log with
  cross-position scrollback, file dump and clipboard copy; a principal-variation browser that walks
  the running engine's current line move by move without touching the game document; a single move
  list that fills itself with book statistics, tablebase results, or legal moves with the engine's
  sufficient refutations; user-set position values that reports honour and mark as user-sourced; an
  on-demand raw protocol transcript; and deferral when an engine plays from its own book.
- **Library and document plumbing** (`game-management`, `board-ui`): an EPD record browser with
  preview, raw-record editing and batch hand-off to sweeps and training sets; a live PGN file kept
  current after every move plus recovery of unfinished games; a material-balance strip; and a
  risk-tinted reply preview while a piece is being dragged.
- **Pending-milestone additions** (`add-analysis-power-tools`): gauntlet tournaments with opponent
  batching and opening de-clustering, and starting matches/tournaments from a saved position set —
  two requirements appended to that change's runner delta, since the runner itself lives there.
- **Excluded on purpose** — present in the manual, out of this project: the vendor's proprietary
  compressed tablebase format and proprietary book/game containers (a fast open format plus
  Nalimov/Syzygy and EPD/PGN cover the workflow), hosted opening books and hosted endgame
  lookups, mobile-device book exports, text-to-speech move announcement (already covered as
  spoken commentary), and 3D board camera play.
  Full reasoning in `design.md` and `backlog/descoped.md`.

## Capabilities

### New Capabilities

- `endgame-lab`: wildcard-driven endgame research over tablebases — sweeps, statistics, filtered
  export, comparison, and practice against perfect defence.

### Modified Capabilities

- `play-vs-computer`: named level catalog, adjudication and draw claims, strict game mode.
- `chess-rules-core`: period and multi-period match clocks, per-move time records, manual clock set.
- `opening-book`: play-value classes and breadth modes, play gates, recent-results correction,
  recency bias, import reporting and log.
- `analysis-workbench`: engine output log, PV browser, self-filling move list, user position values.
- `engine-protocol`: protocol transcript, engine-owned opening book deferral.
- `game-management`: EPD record browser, live game file, unfinished game recovery.
- `board-ui`: material balance strip, drag-time reply risk preview.

## Impact

- **Crates**: `chess-core` (period clocks, move time records, claim verification), `uci-host`
  (transcript tap, engine-book flag), `move-select` (level catalog entries, book policy fields),
  `game-db` (book policy columns, import log, EPD browser index, live file writer), `desktop`
  (lab panels, PV browser, move list, strips and previews), plus the shared tablebase probe pool
  introduced with `add-analysis-power-tools`.
- **File formats**: native book format gains play-class, gate, and last-result-window fields with a
  documented migration; user position values serialize into PGN comments as annotated opcode text;
  sweep exports are plain EPD with the filter recorded in the comment field.
- **Assets**: wildcard badges, risk glyphs, and the material strip reuse the piece-set colour
  slots and the license manifest rules established in `add-power-user-features` — no new art
  pipeline.
- **No network code** is added; every feature reads local files and local processes.
- **Sequencing**: builds on `add-analysis-power-tools` (tablebase probing pool, engine runners)
  and reuses `add-power-user-features` overlays (colour slots, expected-move preview). Safe to
  implement after either; the lab is useless without tablebase files present and says so.
