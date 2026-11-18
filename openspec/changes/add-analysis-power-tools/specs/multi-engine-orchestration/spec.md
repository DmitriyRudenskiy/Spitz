# multi-engine-orchestration Delta

## ADDED Requirements

### Requirement: Up to 10 engines simultaneously

The analysis workbench SHALL run up to 10 engines simultaneously on the same position, each in its own panel with per-engine thread and hash assignment, positions synchronized across all engines; total resource use SHALL be capped at the available cores by default.

#### Scenario: Ten engines on one position

- **WHEN** the user adds 10 engines and assigns one thread each
- **THEN** all 10 panels evaluate the same position synchronously and total thread use does not exceed the core count by default

### Requirement: Arbiter module

The system SHALL provide an arbiter module that lets two engines analyze a position while a third component decides which move alternative to follow, publishing a confidence value between 0 and 100 percent in its own window, during analysis and play.

#### Scenario: Arbitrated choice

- **WHEN** two engines propose different best moves in a position and the arbiter module is enabled
- **THEN** the arbiter window shows the chosen move with a confidence percentage and the engines' individual evaluations

### Requirement: WinBoard engine hosting

Engine orchestration SHALL host engines speaking the WinBoard protocol alongside UCI engines, with the same engine panels and option handling, verified by a conformance test set.

#### Scenario: WinBoard engine added

- **WHEN** the user registers a WinBoard protocol engine from the conformance set
- **THEN** it appears in the engine list, analyzes positions, and reports moves through the same interface as UCI engines

### Requirement: Engine match runner

The system SHALL provide an engine match runner configured in one dialog: engines for both sides, per-side opening books with use toggles, hash sizes, time control with optional secondary time, number of games, book learning, and pondering; all games SHALL be saved to a PGN database and the match is stoppable.

#### Scenario: Configured match

- **WHEN** the user configures a 100-game match with different books per side and starts it
- **THEN** games run automatically under the chosen time control, each game is saved to the PGN database as it finishes, and a live scoreboard shows the running score

### Requirement: Engine tournament runner

The system SHALL provide an engine tournament runner playing a round-robin among up to 10 engines with the same per-engine options, a live cross-table showing rank, score, and performance rating, and stop/resume support.

#### Scenario: Live cross-table

- **WHEN** a round-robin tournament with 10 engines is running
- **THEN** the cross-table shows every played game's result and updates rank, points, and performance rating after each game

#### Scenario: Resume after stop

- **WHEN** the user stops a running tournament and restarts the app
- **THEN** the tournament can be resumed from the played-game state without replaying finished games

### Requirement: Gauntlet format and opening batching

The tournament runner SHALL offer a gauntlet format in which the first listed engine plays every
other engine, with a batch size that plays all games against the next N opponents before moving on,
and SHALL shuffle the order in which games sharing a start position are played so that no two
consecutive games use the same opening when at least three distinct start positions are configured.

#### Scenario: Gauntlet against nine opponents in batches of three

- **WHEN** a gauntlet tournament with 10 participants runs with batch size 3
- **THEN** all games against opponents 1-3 finish before any game against opponent 4, and the
cross-table marks the format and the batch size on its header

#### Scenario: Openings do not cluster

- **WHEN** a tournament plays 30 games drawn from 10 start positions
- **THEN** no two consecutive games share a start position and the produced order is recorded so
the tournament can be replayed exactly

### Requirement: Runner start positions

The match and tournament runners SHALL take an optional saved position set (EPD file or library
selection) as the start positions of games, cycling through the set in a recorded order, and SHALL
turn opening books off for the games of such a run when the user asks for book-free play.

#### Scenario: Match from an EPD set

- **WHEN** a 20-game match is configured from an EPD file of 10 positions with books off
- **THEN** each position starts exactly two games, one per colour assignment, and every game records
its start position and the absence of book play in its header

### Requirement: Legacy autoplayer compatibility

The system SHALL support automated matches between two computers over the legacy autoplayer protocol (auto232-compatible) used by Windows chess programs.

#### Scenario: Two-PC match

- **WHEN** two computers are connected and both sides enable the autoplayer interface
- **THEN** moves are exchanged automatically and the match completes with games saved locally on both machines
