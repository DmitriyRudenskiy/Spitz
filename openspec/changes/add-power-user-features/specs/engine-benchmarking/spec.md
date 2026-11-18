# engine-benchmarking Delta

## Purpose

Engine benchmarking measures the machine and the engines the user actually runs, instead of
repeating advertised numbers: a repeatable hardware report, test position sets where a position is
either solved or not, and matches driven by a position list so openings do not decide the result.

## ADDED Requirements

### Requirement: Hardware report

The app SHALL provide a single benchmark action that measures, on the current machine and without
a user-supplied engine build, the rules-core move-generation throughput on a fixed fixture set, the
nodes per second of each registered engine at a fixed search setting, tablebase probe latency when
tablebases are configured, and the move-selection overhead, and SHALL write the result as a local
report file and show it in the app.

#### Scenario: One run produces a full report

- **WHEN** the user runs the benchmark with two registered engines and installed tablebases
- **THEN** the report lists perft nodes per second, each engine's nodes per second with its name,
  version and settings, tablebase probe latency, and selection overhead in milliseconds
- **AND** the report records the machine description: CPU model, core and thread count, memory
  size, kernel version, and the app version

#### Scenario: The interface stays usable

- **WHEN** the benchmark is running at full load
- **THEN** the app keeps responding, input latency stays at or below 50 ms at p95, and the user can
  cancel the run, which stops all child engine processes within 500 ms

#### Scenario: Report is machine readable

- **WHEN** the run finishes
- **THEN** the report is written as a documented file format (JSON plus CSV export) that a script
  can parse, and re-running the benchmark appends a new run instead of overwriting the old one

### Requirement: Benchmark history and regression check

The app SHALL keep benchmark runs in a local history, SHALL compare any two runs, and SHALL flag a
difference of 10 % or more on any measured figure as a regression or improvement.

#### Scenario: Comparing two runs

- **WHEN** the user selects two runs of the same engine on the same machine
- **THEN** the app shows the per-figure delta with its percentage and marks every figure that moved
  by 10 % or more

### Requirement: Test position sets

The app SHALL import a test position set from an EPD file, reading each record's position, its
expected-move opcodes when present, and its comment, SHALL run a registered engine over the set with
a user-set depth or time budget per position, and SHALL mark a position solved when the engine's
chosen line contains one of the expected moves and unsolved otherwise.

#### Scenario: Solved and unsolved positions

- **WHEN** a 200-position EPD set with best-move opcodes is run at 20 nodes or 2 seconds per
  position
- **THEN** every position is reported as solved or unsolved with the engine's move, the expected
  moves, the depth reached, and the elapsed time
- **AND** the totals per set are shown per engine so two runs can be compared

#### Scenario: Position without expected moves

- **WHEN** a record in the set has no expected-move opcode
- **THEN** the position is reported as informational rather than counted in the solved ratio, and
  the count of such records is shown in the totals

#### Scenario: Records that cannot be used

- **WHEN** the file contains a malformed record or an illegal position
- **THEN** the run continues, the record is listed with its line number and the reason, and the
  totals exclude it

### Requirement: Benchmark result tagging

The app SHALL write the outcome of a test-set run back into the library: every position that came
from a game can be tagged with the run result, unsolved positions get a tag naming the set, and
tagged games are findable through the library search.

#### Scenario: Unsolved positions become searchable

- **WHEN** a run finishes with 40 unsolved positions taken from games in the open database
- **THEN** those games carry the run's tag, the tag appears in the tag filter, and filtering by it
  returns exactly those games

#### Scenario: Re-running a set replaces its own tags

- **WHEN** the same set is run twice with the same run name
- **THEN** the second run replaces the tags it created and leaves tags created by other runs intact

### Requirement: Position-set match

The app SHALL run a match from a position list of 1 to 64 positions in which each side plays every
position once with White and once with Black, using the engine, level, book, and time settings
configured for the match, SHALL persist every game to PGN, and SHALL report a per-position and a
total score.

#### Scenario: Each position played both ways

- **WHEN** a match is started with a list of 10 positions and two sides
- **THEN** 20 games are played, one for each position and colour, and the score table shows the
  points won by each side with White and with Black separately

#### Scenario: No opening book unless asked

- **WHEN** a position-set match is started with default settings
- **THEN** no book is consulted, so both sides must play from the tested position
- **AND** enabling a book is possible and is recorded in the match header and in every game

#### Scenario: Position lists come from files

- **WHEN** the user supplies the position list as EPD, as PGN games, or as a saved selection from
  the library
- **THEN** the match starts from those positions and reports any position it had to reject with the
  reason
