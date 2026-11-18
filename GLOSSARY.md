# Spitz

Domain vocabulary for the Spitz chess workspace: a local-only GUI that plays and analyses through
externally supplied engines. Terms here are domain terms only; how they are implemented lives in
specs and design docs.

## Language

### Positions and their values

**Value**:
The outcome of a position for the side to move, always named with its source: exact (tablebase),
estimated (engine), or user-set. Calling a bare number "the value" without a source is the bug.
_Avoid_: evaluation (reserved for engine output), score (reserved for game results)

**Probe**:
A single value lookup for one position and side to move against a tablebase backend.
_Avoid_: query, lookup

**Sweep**:
Enumerating the positions of a family systematically and recording each value; the unit of endgame
research.
_Avoid_: scan, brute force

**Square sweep**:
A sweep that walks one named piece across the board, one legal square at a time.

**Wildcard sweep**:
A sweep in which every piece has a scope, enumerating the whole family.

**Scope**:
The set of squares one piece may occupy in a wildcard definition: fixed, file, rank, or board.
_Avoid_: range, zone

**Mutual zugzwang**:
A position whose recorded value worsens for a side precisely because it is that side's move —
established only by probing the position with each side to move.
_Avoid_: zugzwang (unqualified, for the statistic)

### Opponents

**Level**:
A named strength setting for playing against an engine, defined as a constraint, the declared
engine mechanism that enforces it, and a measured calibration.
_Avoid_: difficulty, strength (as a noun for the setting)

**Archetype**:
A playing personality defined by selection policy over the lines an engine reports, orthogonal to
level.
_Avoid_: persona, style preset

**Strict mode**:
A game-scoped flag that disables every assist for the game and records any refused request in the
game file; a single use downgrades the game to unrated.

**Adjudication**:
Recording a result before the game ends legally, from engine margin plus a rules veto that a
mateable construction exists; never retroactive.

### Books

**Book class**:
An entry's play-value category (a–g) deciding whether the book may play it at all.
_Avoid_: type, tag, weight (weight is the number inside a class)

**Breadth mode**:
How wide the book plays for a repertoire: tournament, standard, or wide.

**Play gate**:
Per-entry thresholds (games, wins, per-colour score, last move number) a book entry must pass to
be selectable.

### Games and files

**Library**:
A user-visible corpus of game documents with its own indexes and sessions; the thing the user
opens, searches, and closes.
_Avoid_: database (reserve for external systems and for "database" inside parser vocabulary)

**Game document**:
One game with its moves, comments, annotations, and clocks, as stored and shared.
_Avoid_: record (means the PGN object in parser contexts)

**Live game file**:
A user-chosen PGN rewritten after every move so an external reader can watch the game; deleted on
new game. Distinct from a recovery file.

**Recovery file**:
An app-owned snapshot of an unsaved game used only to offer restoration after a crash.

**Engine profile**:
An engine binary together with a saved set of option values, selectable in the engine picker.
_Avoid_: configuration, preset

### Reporting

**Reference hardware**:
The one machine recorded in `docs/benchmarks/reference-hardware.md`; every performance number in
a requirement is a promise about that machine and nothing else.
_Avoid_: target machine, benchmark rig

**Acceptance budget**:
A number written into a SHALL before it was ever measured; binding in tests as written, relaxable
only by a spec edit carrying a measured replacement.

**Provenance**:
Where a value, weight, or annotation came from — tablebase, engine (and which run), user, book.
Every derived number in a report carries one.

**Audit line**:
A timestamped game-file entry recording that a facility was refused or used in a strict game.
