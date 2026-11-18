# analysis-workbench Delta

## MODIFIED Requirements

### Requirement: Infinite Analysis Panel

The app SHALL provide a continuously running analysis panel that displays the evaluation, the principal variation, MultiPV lines for up to 5 moves, nodes per second, total node count, and hash table usage, updating live while attached to any board, driving one or more third-party engines rather than an engine built into the app.

#### Scenario: Attach to any board

- **WHEN** the user opens the analysis panel on any board in the app
- **THEN** analysis starts for that position and the evaluation, principal variation, MultiPV lines, nodes per second, node count, and hash usage refresh at least once per second while the panel is running
- **AND** switching to another board or another position re-targets the panel without a manual restart of the analysis.

#### Scenario: MultiPV configuration

- **WHEN** the user sets the MultiPV count to any value from 1 to 5
- **THEN** the panel displays exactly that number of ranked candidate lines, each with its own evaluation
- **AND** changing the count takes effect on the running analysis without restarting the engine.

#### Scenario: Panel under engine load

- **WHEN** every configured engine is searching at full thread count while the panel is open
- **THEN** interface input latency stays at or below 50 ms at p95
- **AND** scrolling the notation, editing a comment, or drawing an annotation updates the view within one frame.

### Requirement: Tactical Analysis Pass

The app SHALL provide a tactical analysis mode that performs a blunder-only fast pass and completes at least 10 times faster than the full report at default settings, measured on the same hardware over a fixed 100-game test suite, and SHALL report the classifications, the threshold in force, and the coverage of the pass per side.

#### Scenario: Blunder-only fast pass

- **WHEN** tactical analysis and a full report are run on the same game set at default settings
- **THEN** the tactical pass reports only blunder-classified moves
- **AND** the tactical pass wall-clock time is at most one tenth of the full report wall-clock time across the suite.

#### Scenario: Classification summary and threshold

- **WHEN** a tactical pass finishes on a 40-move game with two sides
- **THEN** the summary lists the classified moves and the counts per class and per side
- **AND** the report names the engine, its settings, and the miss threshold that produced them.

#### Scenario: Partial results and cancellation

- **WHEN** a pass is cancelled halfway through a long game
- **THEN** the moves analyzed so far keep their classification, the coverage is stated as a move count, and resuming continues from the first unanalyzed move.

### Requirement: Report-to-Annotations

The app SHALL write report classifications and report comments back into the game document as move annotations without changing the moves themselves. A write-back SHALL be identified by a run name recording the engines and settings used, SHALL be replaceable as a unit by re-running that named pass, and SHALL leave annotations the user wrote by hand untouched.

#### Scenario: Write-back with undo

- **WHEN** the user confirms writing a completed report into a game document
- **THEN** every classified move receives its classification and comment as an annotation on that move
- **AND** the move sequence remains unchanged and the write-back can be undone.

#### Scenario: Named pass is replaced not duplicated

- **WHEN** the same named pass is run twice on one game and the game is saved and reopened
- **THEN** only the most recent set of that pass's symbols and comments is present
- **AND** symbols and comments the user added by hand are unchanged.

#### Scenario: Written findings survive export

- **WHEN** a game carrying written findings is exported to PGN and re-imported
- **THEN** the symbols, the run comment naming the engines and settings, and the run tag are present after the round trip
- **AND** an export requested without analysis content contains none of them.

## ADDED Requirements

### Requirement: Evaluation profile graph

The app SHALL draw an evaluation profile of a game, one point per half-move from White's point of view, SHALL mark classified errors and book moves on the graph, SHALL offer optional overlays of time spent and search depth per move, and SHALL jump to the matching move when a point is clicked.

#### Scenario: Profile of a played game

- **WHEN** analysis data exists for a 60-move game
- **THEN** the graph shows 120 half-move points on a fixed vertical scale in centipawn or mate units
- **AND** each classified blunder and mistake carries its symbol at its point.

#### Scenario: Unanalyzed range is a gap

- **WHEN** the profile is open while analysis is still running
- **THEN** analyzed points appear without a manual refresh, unanalyzed points are drawn as a gap rather than as an evaluation of zero, and the covered move range is labelled.

#### Scenario: Profile as an image

- **WHEN** the user copies the profile to the clipboard or exports it as an image
- **THEN** the image contains the graph with its axis labels and error markers and stays readable at 400 px width.

### Requirement: Deep positional pass

The app SHALL provide a deep analysis mode over a selected range of moves that ends in a written positional summary covering the phase reached, the material and pawn structure, the standing of each side's pieces, king safety, the plans the analysis supports, and the decisive moments, with every statement traceable to the position statistics and engine lines it came from.

#### Scenario: Positional summary is produced

- **WHEN** the deep pass runs over a 40-move game with two engines and local tablebases
- **THEN** the summary names the moves after which the evaluation changed permanently and states the structural reason recorded for each.

#### Scenario: No claim without data

- **WHEN** a candidate statement has no supporting position statistic or engine line
- **THEN** it is omitted or marked as the app's inference instead of being stated as a fact.

#### Scenario: Cost stated before the run

- **WHEN** the user opens the deep pass dialog
- **THEN** the dialog states the per-move budget and the estimated total time before the run starts
- **AND** a run over a 40-move game at the default budget finishes within 10 minutes on the reference hardware.

#### Scenario: Engine lost during the pass

- **WHEN** an engine stops in the middle of a deep pass
- **THEN** the pass reports the move range it completed, keeps the classifications already computed, and offers to resume from the first unfinished move.

### Requirement: Notation log columns

The app SHALL offer optional notation-list columns recording per move the evaluation and depth after the move, the time spent, and the move the app expected when the side deviated from its line, SHALL store those columns in the saved document in a documented form, and SHALL omit them from an export requested without analysis content.

#### Scenario: Columns recorded and displayed

- **WHEN** the evaluation and expected-move columns are enabled while a game is played with a running engine
- **THEN** each completed move row shows its evaluation, depth, time spent, and the expected move when it differs from the played move
- **AND** the figures match the search results recorded for that move.

#### Scenario: Clean export

- **WHEN** the same game is exported with analysis content disabled
- **THEN** the exported PGN contains no evaluation comments, no expected-move markers, and none of the analysis symbols, while the local copy keeps them.
