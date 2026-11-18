# chess-engine Specification

## Purpose

The chess-engine capability is the built-in engine used for playing, analysis, and training. It provides a configurable search pipeline, a pluggable evaluation, a graded strength ladder from beginner to full strength, single- and multi-line analysis modes, shared-memory parallel search, mate solving, and optional Syzygy tablebase probing, and it is exposed to the rest of the app through the engine-protocol capability. Delivered in milestone M1 (change `add-core-platform-mvp`).

## Requirements

### Requirement: Iterative deepening search

The engine SHALL search with iterative deepening alpha-beta combined with quiescence search, a transposition table, and aspiration windows, and SHALL honor a stop request within 200 milliseconds by returning the best move found so far.

#### Scenario: Stop responsiveness

- **WHEN** a search is running at any depth and a stop request arrives
- **THEN** the engine returns the best move found so far within 200 milliseconds of the request
- **AND** the returned move is always legal in the searched position

#### Scenario: Search pipeline in operation

- **WHEN** a search runs to its depth or time limit on a suite position
- **THEN** iteration-by-iteration scores and principal variations are reported progressively, and transposition table entries are reused across iterations

### Requirement: Pluggable evaluation

The engine SHALL evaluate positions through an Evaluator interface whose default implementation is a neural (NNUE-style) evaluator and which also accepts a classical fallback evaluator, reporting all scores on a centipawn scale with positive values favoring White.

#### Scenario: Evaluator substitution

- **WHEN** the classical fallback evaluator is selected instead of the neural evaluator
- **THEN** search and analysis complete normally with no changes to the search code
- **AND** the active evaluator is reported to the caller

#### Scenario: Score sign convention

- **WHEN** a quiet position where White is one rook up is evaluated, and then the color-mirrored position where Black is one rook up is evaluated
- **THEN** the first score is positive and the second score is negative, both on the centipawn scale

### Requirement: Strength ladder

The engine SHALL provide at least 20 distinct strength levels spanning beginner to maximum strength, configured through depth limits, time limits, evaluation noise, and material handicap, such that in a fixed 1000-game match each level scores at least 55% against the adjacent lower level.

#### Scenario: Ladder span

- **WHEN** the strength ladder is enumerated
- **THEN** it contains at least 20 distinct levels, the lowest level limits depth and time and adds evaluation noise, and the highest level runs at full strength with no handicap
- **AND** levels using material handicap concede exactly the defined material at the start of every game

#### Scenario: Adjacent level separation

- **WHEN** each adjacent pair of levels plays the fixed 1000-game match with colors alternating between games
- **THEN** the higher level scores at least 55% of the available points against the lower level

### Requirement: Analysis modes

The engine SHALL support infinite analysis, fixed-depth analysis, fixed-time analysis, and MultiPV analysis reporting up to 5 lines simultaneously.

#### Scenario: Bounded analysis

- **WHEN** analysis is started with a fixed depth or a fixed time limit
- **THEN** the engine stops exactly at the requested depth or after the requested time and reports the best line and score

#### Scenario: MultiPV

- **WHEN** analysis runs with MultiPV set to 5 on a position with at least 5 legal moves
- **THEN** 5 distinct principal variations with independent scores are reported and updated as analysis progresses
- **AND** on a position with fewer than 5 legal moves, exactly one line per legal move is reported

### Requirement: Lazy SMP parallel search

The engine SHALL search with Lazy SMP across a configurable thread count such that nodes per second at 4 threads reaches at least 1.6 times the single-thread rate on the reference hardware.

#### Scenario: Scaling benchmark

- **WHEN** the same fixed-time search benchmark runs with 1 thread and with 4 threads on the reference hardware
- **THEN** the 4-thread nodes-per-second rate is at least 1.6 times the single-thread rate

### Requirement: Mate solving

The engine SHALL solve at least 95% of a fixed 100-position mate-in-2 suite within 5 seconds per position.

#### Scenario: Mate-in-2 suite

- **WHEN** the fixed 100-position mate-in-2 suite runs with a 5-second limit per position
- **THEN** at least 95 positions are solved with the verified mating move
- **AND** every reported solution is checked against the suite's reference answers

### Requirement: Syzygy tablebase probing

The engine SHALL optionally probe Syzygy tablebases in play and analysis for positions with 7 or fewer pieces on the board, reporting exact win, draw, and loss values and following distance-to-zeroing best play.

#### Scenario: Exact value and DTZ play

- **WHEN** play or analysis reaches a covered position with at most 7 pieces and the Syzygy files are configured
- **THEN** the engine reports the exact win/draw/loss value and plays or suggests moves that minimize distance to the next zeroing move
- **AND** with no tablebase configured the engine searches the same positions normally
