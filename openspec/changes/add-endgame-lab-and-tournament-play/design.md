# Design

## Context

The surface this change adds sits behind the loop the first milestones specify. It assumes:
tablebase files exist on disk and the app can probe them itself (`add-analysis-power-tools`);
engines are hosted processes exposing only the options they declare (`engine-protocol`); the game
document, library, and book are local files with documented formats (`game-management`,
`opening-book`); and the board has an overlay layer with theme colour slots
(`add-power-user-features`).

The feature inventory this change was diffed from is a public user manual of an engine-hosting
reference product; the notes stayed outside the repository, and nothing from it — names, formats,
content — enters these specs. Local-only and GUI-only keep biting exactly where the manual's
product reached for hosted books, hosted endgame servers, and a proprietary compressed tablebase
format.

Dependencies: `add-analysis-power-tools` (tablebase probing pool, runners, endgame explorer),
`add-power-user-features` (overlay layers, colour slots, expected-move preview), `chess-rules-core`,
`engine-protocol`, `game-management`, `opening-book`, `play-vs-computer`, `analysis-workbench`,
`board-ui`.

## Goals

- Endgame research that answers questions about whole families of positions, not one position at
  a time, from tablebase data alone where it exists.
- Games that can be played to tournament rules: period clocks, claims, adjudication that survives
  a fortress, a strict mode with an audit trail.
- Book behaviour the user can state in one sentence and the panel can prove on screen.
- Engine activity that is fully observable: log, transcript, PV walk, move list, user-set values.

Deliberately out of scope, with the reason each time:

- The reference product's proprietary compressed tablebase format — a closed container of unknown
  provenance; the fast open format plus Nalimov/Syzygy already specified in `tablebase-suite`
  covers the workflow, and this change consumes whatever backend is configured.
- Hosted opening books and hosted endgame lookups — the product binds no ports.
- Proprietary book and game container formats — EPD and PGN plus the documented native formats
  cover the same workflows.
- Mobile-device book exports — a second product's format, not ours.
- Spoken move and evaluation announcement — already specified as spoken commentary in
  `game-commentary`; a second text-to-speech path would be a duplicate subsystem.
- 3D board camera manipulation beyond the existing `board-ui` 3D requirement — polish milestone
  material, not this change.
- Any engine-side work (a stronger "level" is an option mapping and a calibration, never a claim
  about absolute strength).

## Decisions

### D1 — The lab probes tablebases through the app, never through an engine

Sweeps need the value of millions of positions, one per combination; an engine answers one search
at a time and returns lines, not a truth table. The lab therefore uses the app's own probe pool
from `tablebase-suite`, with the same backend selection and missing-file reporting.

**Rejects**: driving an engine per combination, and asking an engine to "guess" family statistics.
**Closes**: a research tool whose numbers change with engine settings and cannot be re-proved.

### D2 — Sweeps are streaming jobs with a resumable cache

Enumeration runs as a background job on the shared probe pool, streaming values into the view,
interruptible with partial results kept, and caching results per (material class, wildcard
definition, backend) with a checksum so a re-run is instant and a compare reuses the first run.

**Rejects**: computing the complete set before showing anything (11M positions on a five-piece
family is minutes of dead UI), and no caching (every comparison re-probes the world).
**Closes**: an interrupting user losing all work, and a compare view that doubles the cost.

### D3 — Exact and estimated values never share a view unmarked

Square sweeps on covered material show tablebase-exact values. Where material is uncovered, the
sweep falls back to a labelled engine estimate with its budget printed, and exact-plus-estimated
values appear together only when the user turns the mixed view on.

**Rejects**: one number column with mixed provenance. **Closes**: an estimate being quoted as a
theoretical truth in analysis write-ups.

### D4 — Zugzwang and longest-mate statistics come from double probes only

A position counts as mutual zugzwang only when probing it with each side to move gives the
recorded pattern; longest mates name both sides to move. Statistics with one probe per position
would invent zugzwangs and are forbidden by the requirement.

**Rejects**: inferring the second side-to-move's value from the first. **Closes**: published
endgame statistics that fail reproduction — the exact claim the research features exist to make.

### D5 — Sweep values render on the board's annotation layer with a discrete diverging scale

Per-square values are drawn by the `board-ui` overlay layer: at most four tint stops plus neutral
on theme colour slots for the diverging scale, value text at the theme's minimum readable size with
at least 4.5:1 contrast against every square colour, and short forms (`W`, `D`, `L`, `+M<n>`) to
fit a square. The scale legend is part of the overlay, not a separate floating window.

**Rejects**: a continuous rainbow heat map (unreadable at square size, fights custom themes), and
tooltips-only values (nothing is visible at a glance). **Closes**: a research display that only
works in one theme and photographs badly in exports.

### D6 — Wildcard scope is a four-value enum per piece with corner-arrow badges

Each piece gets fixed / file / rank / board scope by clicking it in the definition dialog; the
board preview marks scoped pieces with a corner arrow glyph from the original icon set, in the
colours of the piece-set theme, with a text alternative naming the scope for screen readers.

**Rejects**: a placement mini-language or regex positions (untestable UI, unusable by the target
user), and colour-only scope marking (fails colour-blind users and greyscale export). **Closes**:
an ambitious editor nobody can operate, and invisible state.

### D7 — Level catalog entries are (constraint, mechanism, calibration) triples

Each named level states the constraint it enforces (depth, nodes, time, Elo option, period
budget), the declared UCI option that realises it, and the calibration corpus measured per engine
that maps "Elo 1800" to concrete option values. Levels render disabled with the reason when the
mechanism is absent — the `move-selection` degradation rule, reused, not redefined.

**Rejects**: absolute strength claims ("plays like a 1800 player") without a measured per-engine
band, and inventing weakness by random blunder moves inside a level (that is the humanized
opponent's error model, not a level). **Closes**: levels that silently stop being the level the
user picked when the engine changes.

### D8 — Adjudication is engine score plus rules-core veto

Resign and draw adjudication read the engine evaluation but must pass a rules-core material check
that a mate-able construction exists before any result is recorded; terminations use standard PGN
termination fields; nothing is retroactive across a resumed game.

**Rejects**: engine-only resignation (engines resign technically drawn fortresses and misread
opposite-coloured endings), and app-side "resign if losing for N moves" without a material check.
**Closes**: recorded losses in positions with no legal losing reason, and PGN files other tools
cannot interpret.

### D9 — Strict mode is one game-scoped flag every panel reads

Strict mode lives in the game record; assists, analysis panels, clock editing, and hint requests
all query it rather than keeping their own toggles, and every refusal appends an audit line
(facility, move number, timestamp) to the game file.

**Rejects**: per-panel settings that can silently disagree, and strict mode as a global setting.
**Closes**: "rated" games where half the table was off, and audit trails that cannot answer what
was used.

### D10 — Period clocks belong to the rules core, level budget belongs to move-select

Multi-period match controls, per-move time records, and manual corrections are rules-core
behaviour with millisecond-accurate tests. "Average time over a game budget" is a level that
consumes those records to pick a per-move time; the rules core never decides pacing.

**Rejects**: implementing clocks in the UI layer on top of wall time, and pacing logic inside the
clock. **Closes**: clocks that drift from the notation's per-move times, and untestable period
transitions.

### D11 — Book policy fields live in the native book format, versioned

Play-value class, gate thresholds, and the recent-results window migrate into the documented
native book format with a version bump and a one-pass migration; imports from tools without
classes default every entry to class `a` and say so in the import report.

**Rejects**: a sidecar policy file per book (two sources of truth that drift), and silently
assigning classes by heuristic. **Closes**: books that lose their personality because policy lived
outside them, and import runs nobody can audit.

### D12 — Recent-results correction adjusts display and weight, never statistics columns

Lifetime columns (games, score, Elo, last-used) stay exactly as compiled. The correction is a
separate computed column with its window recorded; selecting with the correction on uses the
adjusted value, and the panel shows both so the user sees the demotion.

**Rejects**: mutating stored lifetime statistics from the user's personal results (poisons the book
for everyone who receives it). **Closes**: a book that cannot be shared because it memorised one
player's losses.

### D13 — User position values are annotations with provenance

A user-set value serializes into the game document as a documented opcode in the move's comment,
carries the value type and the setting timestamp, and is read back by reports, sweeps, and lab
runs as a terminal value marked `user`. Engine caches are never patched.

**Rejects**: rewriting engine output in memory (not reproducible across sessions), and a separate
values file per game (unsent, unshareable, lost). **Closes**: "helping the analysis" becoming a
rumour — the value is in the file, visible, removable.

### D14 — Engine log and transcript are different things

The analysis log is parsed, capped, evicting, UI-scrolled text meant for reading. The protocol
transcript is raw line-for-line, timestamped, off by default, performance-bounded, meant for bug
reports. They share no buffer.

**Rejects**: one window that is both, and unbounded buffers in either. **Closes**: "attach a log"
bug reports where the interesting lines were evicted, and a transcript dragging the engine below
its benchmark.

### D15 — Live file and recovery file are distinct writers

Live-file mode rewrites one user-chosen PGN atomically via temporary file plus rename after every
move; recovery storage is an app-owned directory with its own cadence and lifetime. The live file
is deleted on new game; recovery entries are deleted on save or discard.

**Rejects**: one autosave file serving both jobs (external readers watching a file the app also
prunes), and in-place appending (torn reads). **Closes**: half-written PGN handed to a spectator
tool, and lost games after a crash.

### D16 — Material strip and risk tints are derived views with hard budgets

The material strip recomputes only on the move that changes material; the drag-time risk tint reads
the analysis cache within a 150 ms budget and paints nothing on a miss. Neither starts an engine
search.

**Rejects**: re-scanning the game record per frame, and on-hover on-demand searches (input lag and
engine spam). **Closes**: a board UI that drops frames during drag in long games.

### D17 — Graphics reuse the M4 asset pipeline

Wildcard badges, risk glyphs, sweep legend, and the material strip are drawn from the icon grid,
piece-set colour slots, and license manifest established in `add-power-user-features`; this change
adds no new artwork dependency and no new art format.

**Rejects**: bespoke raster screenshots of other programs' UIs as a design reference or asset
source, and one-off inline drawings outside the theme system. **Closes**: unlicensed or
theme-breaking artwork arriving with the panels, and a second theming mechanism.

## Risks and trade-offs

- Probe throughput is the lab's whole cost model; the 10-second square-sweep budget must be
  re-measured once the probe pool exists, and the number relaxed with evidence rather than the
  feature removed.
- Period controls interact with the increment/delay controls in ways no public manual specifies;
  the fixture suite should include the FIDE-style combinations before the UI exposes them.
- The double-probe rule doubles the work of zugzwang statistics; on slow storage that can dominate
  a sweep, so the statistic is opt-out in the sweep dialog with the cost stated.

## Migration and rollout

- Native book format version bump with a one-pass in-place migration and a pre-migration backup
  copy; books are useless without the app, so the migration runs on first open, not at install.
- Recovery storage and the import log are new app-owned files in the existing data directory;
  nothing existing changes format.
- Clock changes are additive to the rules core API; games stored before period controls keep
  working because their controls are a subset of the new set.
- Land after the `add-analysis-power-tools` probe pool and runners exist; the observation deltas
  (log, PV browser, move list, transcript) can ship independently of the lab and the level work.

## Open questions

- Strict-mode audit lines live in the game file as comments; pick the exact opcode namespace when
  the annotation sidecar format from `add-power-user-features` is written, so the two do not
  collide.
- Whether practice sessions should feed `training-center` progress records or keep their own
  history: this change keeps its own; the merge is a tuning decision once both histories exist.
