# analysis-workbench Specification

## Purpose

The analysis workbench provides every analysis feature as a panel attachable to any board in the app, whether the position comes from live play, game review, or training. It combines continuous engine analysis, a whole-game quality report, a fast tactical pass, Monte Carlo playout statistics, and side-by-side engine comparison. Results run as background jobs, stream into the interface while they are computed, and can be written back into the game document as annotations. Delivered in milestone M2 (change `add-analysis-and-training`).

## Requirements

### Requirement: Infinite Analysis Panel

The engine SHALL provide a continuously running analysis panel that displays the evaluation, the principal variation, MultiPV lines for up to 5 moves, nodes per second, total node count, and hash table usage, updating live while attached to any board.

#### Scenario: Attach to any board

- **WHEN** the user opens the analysis panel on any board in the app
- **THEN** analysis starts for that position and the evaluation, principal variation, MultiPV lines, nodes per second, node count, and hash usage refresh at least once per second while the panel is running
- **AND** switching to another board or another position re-targets the panel without a manual restart of the analysis.

#### Scenario: MultiPV configuration

- **WHEN** the user sets the MultiPV count to any value from 1 to 5
- **THEN** the panel displays exactly that number of ranked candidate lines, each with its own evaluation
- **AND** changing the count takes effect on the running analysis without restarting the engine.

### Requirement: Full-Game Report

The system SHALL generate a full-game report that classifies every move as best, good, inaccuracy, mistake, or blunder against the best line using evaluation-loss thresholds of at least 0.5 pawns for inaccuracy, 1.0 for mistake, and 2.0 for blunder, with all three thresholds configurable, and that computes an accuracy percentage for each player.

#### Scenario: Threshold classification

- **WHEN** the report evaluates a move whose evaluation loss against the best line is 0.6, 1.2, or 2.3 pawns at default thresholds
- **THEN** the move is classified as inaccuracy, mistake, or blunder respectively
- **AND** a move with a loss below the inaccuracy threshold is classified as best when it matches the engine's first choice and good otherwise.

#### Scenario: Background streaming

- **WHEN** a full-game report runs on a completed game
- **THEN** it executes as a background job and streams per-move classifications, per-player accuracy percentages, and the top key moments into the interface as they are computed
- **AND** the user can keep working on other boards while the report is still running.

### Requirement: Tactical Analysis Pass

The engine SHALL provide a tactical analysis mode that performs a blunder-only fast pass and completes at least 10 times faster than the full report at default settings, measured on the same hardware over a fixed 100-game test suite.

#### Scenario: Blunder-only fast pass

- **WHEN** tactical analysis and a full report are run on the same game set at default settings
- **THEN** the tactical pass reports only blunder-classified moves
- **AND** the tactical pass wall-clock time is at most one tenth of the full report wall-clock time across the suite.

### Requirement: Monte Carlo Analysis

The system SHALL provide Monte Carlo analysis that reports per-candidate-move playout statistics (win, draw, and loss percentages plus popularity), honors a user-set playout budget, and caches results so that re-runs merge with previous playout counts.

#### Scenario: Budget and merged statistics

- **WHEN** the user runs Monte Carlo analysis with a 10,000-playout budget and then re-runs it with 10,000 further playouts on the same position
- **THEN** each candidate move shows win, draw, and loss percentages and popularity after each run
- **AND** the merged statistics equal the values computed over the combined 20,000 playouts.

### Requirement: Engine Comparison

The app SHALL run at least 2 third-party engines side by side on the same position with synchronized analysis updates and SHALL highlight positions where the engines disagree on their first-ranked move or on the evaluation sign.

#### Scenario: Synchronized disagreement highlight

- **WHEN** two or more engines are attached to the comparison panel and the user changes the position or plays a move
- **THEN** every attached engine re-synchronizes to the new position and continues live analysis
- **AND** whenever the engines' first-ranked moves differ or their evaluation signs differ, the panel highlights the disagreement.

### Requirement: Key-Moments Navigator

The system SHALL provide a key-moments navigator that lists a reported game's moments ordered by evaluation swing and offers a jump to the position before and to the position after each moment.

#### Scenario: Swing-ordered jump list

- **WHEN** a full-game report has completed for a game
- **THEN** the navigator lists the key moments sorted by descending evaluation swing
- **AND** selecting a moment jumps the board to the position before the swing and then to the position after it.

### Requirement: Report-to-Annotations

The app SHALL write report classifications and report comments back into the game document as move annotations without changing the moves themselves.

#### Scenario: Write-back with undo

- **WHEN** the user confirms writing a completed report into a game document
- **THEN** every classified move receives its classification and comment as an annotation on that move
- **AND** the move sequence remains unchanged and the write-back can be undone.
