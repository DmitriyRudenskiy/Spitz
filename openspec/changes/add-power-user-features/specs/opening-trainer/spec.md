# opening-trainer Delta

## MODIFIED Requirements

### Requirement: Repertoire Tree

The app SHALL maintain a repertoire tree per color that can be built from manual moves, imported games, or a book, or built from the games already in the library as a ranked list of candidate moves per position with the number of games, the score, and how often the user played the move. The tree SHALL label each move with its repertoire status, and any colour used for a status SHALL stay distinguishable with the colour-accessibility option enabled.

#### Scenario: Building from all three sources

- **WHEN** the user adds manual moves, imports games, and imports from a book into one color's repertoire
- **THEN** the tree contains the combined move tree for that color only
- **AND** identical positions reached by different move orders merge into a single tree node.

#### Scenario: Color separation

- **WHEN** the user maintains repertoires for both colors in the same database
- **THEN** each color's repertoire tree is stored and displayed separately
- **AND** a move for the opponent's color never appears as a required user move in the other color's drill.

#### Scenario: Building from the library

- **WHEN** the user asks for a repertoire built from the open library within an Elo band and a date range
- **THEN** each position lists its candidate moves with game count, score, and the user's own frequency, ordered by the configured ranking
- **AND** choosing moves adds them to the tree as one undoable edit batch reporting how many positions were added

#### Scenario: Colour-blind-safe status

- **WHEN** the colour-accessibility option is enabled on a repertoire with every status represented
- **THEN** each status keeps at least a 4.5:1 contrast ratio against its background and carries a shape, letter, or text marker in addition to its colour

### Requirement: Drill Loop

The trainer SHALL drill repertoire lines against the engine and, whenever the user deviates from the repertoire, SHALL show the repertoire move and its statistics. The loop SHALL offer a strict mode, where the branch waits for the repertoire move or a correction, and a free mode, where any legal move is answered from the book when the book has the position and from the configured engine otherwise and the game continues to its result, and SHALL record deviations in both modes.

#### Scenario: Deviation handling during drill

- **WHEN** during a drill the user plays a move that is not in the repertoire at the current position
- **THEN** the drill pauses and displays the repertoire move together with its stored statistics
- **AND** the user can continue the drill from the repertoire move or from the played move.

#### Scenario: Free mode plays the game out

- **WHEN** the user plays a move outside the repertoire with free mode on
- **THEN** the app answers from the book or the configured engine, the game continues to a result or until the user stops it, and the deviation is stored against the repertoire position with the move played and the game's result

#### Scenario: Mode is chosen per session

- **WHEN** a drill session is started
- **THEN** the session starts in the mode configured for that repertoire, the active mode is shown during the session, and switching mode mid-session is recorded in the session log

## ADDED Requirements

### Requirement: Repertoire status report

The app SHALL report, per repertoire, how many positions are in each status — part of the repertoire,
answered correctly, answered wrongly, never drilled — SHALL list the positions with the most wrong
answers, and SHALL derive every figure from the stored drill history rather than recomputing it from
the games.

#### Scenario: Status counts and weakest positions

- **WHEN** the user opens the report of a repertoire with drill history
- **THEN** the report counts the positions in each status, lists the ten positions with the most
  wrong answers, and states the number of drills behind each figure

#### Scenario: Report follows the history

- **WHEN** a position is drilled correctly twice after three wrong answers and the report is reopened
- **THEN** the position has moved out of the wrong-answer list, the counts add up to the number of
  positions in the repertoire, and undoing the drills restores the previous figures
