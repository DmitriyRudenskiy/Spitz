# opening-book Delta

## MODIFIED Requirements

### Requirement: Book Panel

The book panel SHALL show, for each legal move in the current position, the move, its weight, the number of games and the win/draw/loss percentages behind it, an ECO code and opening name where the entry carries them, an Elo-band label, and a marker for the strongest move in the selected band, and SHALL support toggling each column, sorting by each column, and navigating to the next or previous position of the book. The full book move count SHALL be shown for the position.

#### Scenario: Move probabilities

- **WHEN** the book contains the current position
- **THEN** each listed move shows its weight and the observed win/draw/loss percentages and the total number of games behind it

#### Scenario: Columns and ECO labels

- **WHEN** the user enables the ECO and Elo-band columns and sorts by number of games
- **THEN** the panel re-sorts without losing the current position or the selection, each row is labelled with its ECO code and opening name when present, and the strongest move in the selected Elo band is marked

#### Scenario: Elo bands

- **WHEN** the panel is set to an Elo band whose games disagree with the full book
- **THEN** the moves, percentages, and strength marker reflect that band only, and positions without games in that band say so rather than showing the unfiltered figures

#### Scenario: Book navigation

- **WHEN** the user asks for the next or previous book position from the panel
- **THEN** the board steps to the neighbouring position in the book along the current line and the panel follows without a reload

#### Scenario: Statistics at the board

- **WHEN** the current position exists in the loaded book
- **THEN** the panel lists the booked moves with play count, score percentage, average opponent Elo, and last-used date
- **AND** when the position is not in the book the panel shows an explicit no-data state instead of an empty list.


### Requirement: Book Editing

The app SHALL let the user add, edit, and delete book entries in a book opened for editing, and record every change in an undoable edit log; it SHALL additionally support, on a local book only, deleting all entries created from one imported source or one game, setting an Elo limit below which imported games are ignored, and importing a line already analyzed with weights taken from the recorded analysis.

#### Scenario: Add and delete entries

- **WHEN** the user adds a move with weight 30 in edit mode and then deletes it
- **THEN** the panel reflects each change and the engine's book query reflects the final state

#### Scenario: Undoable

- **WHEN** the user presses undo after three edits
- **THEN** the book returns to the state before those edits and no stale entry remains in any open book

#### Scenario: Removing an imported source

- **WHEN** the user asks to delete the entries contributed by one imported game or one import batch
- **THEN** only the entries and weights contributed by that source are changed, positions that other
  sources also reached keep their remaining weight, and the count of modified entries is reported

#### Scenario: Elo limit on import

- **WHEN** an import runs with an Elo limit of 1800
- **THEN** games below the limit are not added, the report lists how many were skipped and why, and
  the limit is recorded in the book metadata

#### Scenario: Transposition-aware edits

- **WHEN** the user penalizes a move or adds a comment in one position that also occurs in the book under a different move order
- **THEN** the penalty and the comment appear on every transposed occurrence of that position and move
- **AND** added and renamed moves remain consistent across all transpositions of the affected position.


### Requirement: Book Compilation

The app SHALL compile a book from a PGN database of at least 1,000,000 positions, merging duplicate occurrences of the same position into a single entry with aggregated statistics. Compilation SHALL accept an Elo band, a date range, and a source filter, SHALL store per-band statistics for a position, SHALL mark a move playable only at or above a configurable minimum game count, SHALL abort with an error when the number of distinct positions exceeds 2,000,000, and SHALL emit progress with a percentage during compilation.

#### Scenario: Large import with duplicate merging

- **WHEN** the user compiles a book from a PGN database containing at least 1,000,000 positions with repeated games and transpositions
- **THEN** compilation completes and produces one entry per unique position with aggregated move statistics
- **AND** each duplicate occurrence increments the play counts instead of creating a duplicate move entry

#### Scenario: Compilation with filters

- **WHEN** a compilation runs over 200k source games filtered to an Elo band, a date range, and one database
- **THEN** every reached position has its move weights and win/draw/loss stats for that band, and positions at or above the configured minimum game count are marked as playable

#### Scenario: Band boundary

- **WHEN** a position exists in multiple Elo bands
- **THEN** each band's stats are stored and queried separately and a query with no band specified aggregates them with the aggregation rule documented

#### Scenario: Too many positions

- **WHEN** a compilation would produce more than 2,000,000 positions
- **THEN** it aborts with an error stating the counted positions and the applied filters, and writes no partial book

## ADDED Requirements

### Requirement: Multiple book sources

The app SHALL let more than one book be active for a game or an analysis session, SHALL resolve a
position by consulting the sources in the configured order, and SHALL label the move it plays with
the source it came from.

#### Scenario: Source order decides

- **WHEN** two active books contain the same position and the user's own book is listed first
- **THEN** the move and the statistics shown come from the first source that has the position, and
  the panel names the source used

#### Scenario: A source becomes unreadable

- **WHEN** an active book's file is moved or becomes unreadable during a session
- **THEN** the remaining sources keep working, the broken source is listed as unavailable with its
  path, and no move is played from a source that failed to load

#### Scenario: Per-context book sets

- **WHEN** the user configures one book set for play against the computer and another for analysis
- **THEN** starting a game or an analysis session uses the set configured for that context, and the
  game header or session report names the books in use

#### Scenario: Engine-side book queries stay single-source

- **WHEN** an engine is configured to consult a book itself
- **THEN** it receives exactly one book path, chosen by the source order, and any other active
  sources are used by the app's own panel only, so the two never disagree silently
