# move-selection Specification

## Purpose

Move selection is the layer that decides what the computer opponent actually plays. Spitz ships no engine: a third-party UCI engine reports ranked lines and scores, and this layer turns that report into a move that matches the configured strength, style, handicap, and time control, using only the options the engine declares and the lines it returns. Play-versus-computer, the style engine, and the humanized opponent state their behavioral requirements against this layer; the layer itself is mechanism and reproducibility. Delivered in milestone M1 (change `add-core-platform-mvp`).

## Requirements

### Requirement: Selection owns the opponent move

The opponent's move SHALL be chosen by the selection layer from the engine's reported lines and SHALL NOT be the raw bestmove of any engine when a level, style, handicap, or book setting is in force.

#### Scenario: Engine bestmove is not used directly

- **WHEN** a game runs at a non-maximum strength level and the engine reports its preferred first line
- **THEN** the move played is the selection layer's output and the game records the engine's first-ranked line separately for later comparison

#### Scenario: Reason recorded with every move

- **WHEN** the opponent completes a move at any configured level
- **THEN** the game metadata records the reason for that move (book, level cap, sampled line, handicap constraint, resign, or draw offer) alongside the engine id, level, style vector, and seed

### Requirement: Level to option mapping with degradation

The layer SHALL translate a strength level into the strongest applicable set of declared engine options — `UCI_Elo` with `UCI_LimitStrength`, depth limit, node limit, or time limit — and SHALL fall back to depth and node limits plus sampling when an option is not declared.

#### Scenario: Engine without Elo limiting

- **WHEN** a level is selected on an engine that declares no `UCI_Elo` option
- **THEN** the level is realized through depth and node caps plus sampling width, the ladder calibration still measures the level's result, and the UI names the mechanism in use

#### Scenario: Option set changes with the engine

- **WHEN** the user switches the opponent to a different registered engine mid-configuration
- **THEN** the level control re-labels itself for the mechanisms the new engine supports and the numeric level is preserved where the calibration allows it

### Requirement: Sampled line selection

The layer SHALL choose among the reported lines by weighted sampling, where the weights come from the score gap to the best line, the configured style vector, and the level's sampling width, and SHALL produce a human-shaped distribution rather than alternating between best and random moves.

#### Scenario: Sampling widens as strength drops

- **WHEN** the same position is played at a low level and at a high level with a fixed seed
- **THEN** the low-level selection distribution spreads over more of the reported lines, and the high-level distribution concentrates on the best line

#### Scenario: Deterministic for a fixed seed

- **WHEN** a position and full configuration are replayed twice with the same seed
- **THEN** both runs choose the same moves in the same order

### Requirement: Opening book gate

The layer SHALL consult the opening book before querying the engine, and the amount of opening knowledge in use SHALL be adjustable independently of search strength.

#### Scenario: Book move skips the engine

- **WHEN** the current position has a book move under the configured book setting
- **THEN** the book move is played without starting an engine search and the game records the book set's content hash

#### Scenario: Weak play from a strong book

- **WHEN** full opening knowledge is combined with the lowest search strength
- **THEN** the opening follows the book and the middlegame is produced by the low-strength selection path

### Requirement: Ladder calibration harness

The layer SHALL ship a calibration harness that measures strength levels by match play, so that "level 8" means the same measured thing across engine builds: at least 20 distinct levels, each scoring at least 55% of available points against the adjacent lower level over a fixed 1000-game match with colors alternating.

#### Scenario: Nightly ladder gate

- **WHEN** the nightly job runs the ladder matches for a registered engine
- **THEN** every adjacent pair meets the 55% threshold or the build fails the gate and the offending pair is reported with its score

#### Scenario: Per-push smoke ladder

- **WHEN** a normal CI push runs the 100-game smoke ladder
- **THEN** the ordering of levels is preserved (no level scores below 50% against the level below it over the smoke sample) within the run-time budget

### Requirement: Selection overhead budget

The layer SHALL add no more than 50 ms at p95 between the engine reporting a line and the move being made, and SHALL NOT hold the UI thread while it decides.

#### Scenario: Decision latency

- **WHEN** 1000 selections run on the reference hardware at level 10 against a locally registered engine
- **THEN** the measured selection overhead is at most 50 ms at p95 and the UI stays responsive at 60 fps throughout
