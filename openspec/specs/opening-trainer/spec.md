# opening-trainer Specification

## Purpose

The opening trainer manages the user's opening repertoires and converts them into a practice loop. It maintains a repertoire tree per color, builds it from manual moves, imported games, or a book, drills repertoire lines against the engine, records where and how often the user deviates, schedules spaced repetition reviews, and exchanges repertoires as PGN including comments. Delivered in milestone M2 (change `add-analysis-and-training`).

## Requirements

### Requirement: Repertoire Tree

The app SHALL maintain a repertoire tree per color that can be built from manual moves, imported games, or a book.

#### Scenario: Building from all three sources

- **WHEN** the user adds manual moves, imports games, and imports from a book into one color's repertoire
- **THEN** the tree contains the combined move tree for that color only
- **AND** identical positions reached by different move orders merge into a single tree node.

#### Scenario: Color separation

- **WHEN** the user maintains repertoires for both colors in the same database
- **THEN** each color's repertoire tree is stored and displayed separately
- **AND** a move for the opponent's color never appears as a required user move in the other color's drill.

### Requirement: Drill Loop

The trainer SHALL drill repertoire lines against the engine and, whenever the user deviates from the repertoire, SHALL show the repertoire move and its statistics.

#### Scenario: Deviation handling during drill

- **WHEN** during a drill the user plays a move that is not in the repertoire at the current position
- **THEN** the drill pauses and displays the repertoire move together with its stored statistics
- **AND** the user can continue the drill from the repertoire move or from the played move.

### Requirement: Deviation Statistics

The trainer SHALL record where and how often the user leaves the repertoire and SHALL offer refocus suggestions derived from those statistics.

#### Scenario: Refocus suggestions

- **WHEN** the user opens the deviation statistics of a repertoire
- **THEN** every deviation is listed with its position and frequency, ordered by frequency
- **AND** the trainer proposes the most-deviated lines as the next drill focus.

### Requirement: Spaced Repetition

The trainer SHALL schedule repertoire positions for review using a spaced repetition schedule.

#### Scenario: Review queue updates

- **WHEN** repertoire positions become due for review
- **THEN** they appear in a due-review queue ordered by due date
- **AND** a successful review moves the next due date further into the future while a failed review shortens the interval.

### Requirement: Repertoire Interchange

The app SHALL import and export repertoires as PGN, including move comments.

#### Scenario: Round trip with comments

- **WHEN** a repertoire containing move comments is exported to PGN and re-imported
- **THEN** the tree structure and every comment are identical after the round trip
- **AND** comments in any imported PGN attach to their corresponding moves in the tree.
