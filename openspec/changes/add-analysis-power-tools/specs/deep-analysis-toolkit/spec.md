# deep-analysis-toolkit Delta

## ADDED Requirements

### Requirement: Selected-moves analysis

The analysis workbench SHALL analyze a user-selected set of moves in the current position — including any number of moves with a modifier-key move entry command and excluding moves with a second modifier command — by searching one selected move at a time and reporting a full-depth verdict for each, and SHALL show the expected cost (moves × depth) before the job starts. Where a registered engine declares a root-move restriction option, the app SHALL use it as a fast path instead of one search per move. The feature SHALL be exposed as a first-class menu and toolbar command.

#### Scenario: Included moves only

- **WHEN** the user adds three moves to the include set with the modifier command and starts analysis
- **THEN** the workbench reports one full-depth line per included move, each labeled with the move it analyses
- **AND** moves outside the set are not analyzed

#### Scenario: Cost is stated up front

- **WHEN** the user starts selected-move analysis on a position with 30 legal moves and 12 moves in the include set
- **THEN** the job reports the number of searches and the per-move depth before starting, and the user can cancel it

#### Scenario: Excluded move

- **WHEN** the user adds the engine's best move to the exclude set
- **THEN** the reported ranking contains only continuations that avoid the excluded move

#### Scenario: Engine offers a fast path

- **WHEN** the registered engine declares support for root-move restriction
- **THEN** a single search covers the whole include set and produces the same verdicts as the per-move path within one depth of each other

### Requirement: Game histogram

The analysis workbench SHALL build a histogram — a graph of the evaluation progress of an entire game — through iterative analysis passes that continue until the user stops, with each pass deeper than the last; the histogram SHALL display per-move evaluation and SHALL offer move-time and search-depth overlays and a copy-to-notation action.

#### Scenario: Iterative passes

- **WHEN** the user starts the histogram on a 60-move game and lets it run for two passes
- **THEN** every move has an evaluation after the first pass and refined evaluations after the second, with no gaps

#### Scenario: Copy to notation

- **WHEN** the user activates copy-to-notation on a completed histogram
- **THEN** the evaluations are written into the game notation as numeric annotations

### Requirement: Shootout continuation

The analysis workbench SHALL run a shootout — engine self-play from a chosen position — and record the resulting continuation as a new variation in the game document; the starting position can be picked directly from the histogram.

#### Scenario: Shootout from histogram position

- **WHEN** the user picks a position on the histogram and starts a shootout
- **THEN** the engines play out a continuation to a decisive or drawn end and the continuation appears as a new variation rooted at the chosen move

### Requirement: Batch position analysis

The system SHALL analyze all positions in EPD or PGN files as a background job, show live per-position results (solution move, time, evaluation), produce statistics on completion, and export results as EPD and CSV files; the job is stoppable.

#### Scenario: EPD suite run

- **WHEN** a 100-position EPD test file is analyzed with a per-position time limit
- **THEN** a live table shows each position's result as it completes and the final export produces an EPD file and a CSV file with solution, time, and evaluation columns

### Requirement: Background analysis during opponent turn

During play against the computer the app SHALL optionally analyze the current position in the background while the opponent thinks, within a configurable resource cap, and the option SHALL be disableable.

#### Scenario: Thinking on opponent time

- **WHEN** background analysis is enabled and the computer opponent is thinking
- **THEN** the analysis panel updates evaluations for the user's position without delaying the opponent's reply beyond its time budget
