# analysis-workbench Delta

## ADDED Requirements

### Requirement: Engine output log

The app SHALL keep a per-session engine output log that accumulates engine lines across positions
with a position marker between them, in a capped ring buffer (default 50,000 lines, configurable up
to 500,000, oldest evicted first), with keyboard scrolling, a clear command, an append-to-text-file
dump carrying a session header, and clipboard copy of the whole buffer or of the last position's
lines only.

#### Scenario: Scroll back through earlier positions

- **WHEN** the user has analysed 30 positions and scrolls to the top of the log
- **THEN** lines from the first analysed position are readable, position markers separate each
  position's block, and clearing the log empties the view without stopping the running analysis

#### Scenario: Dump and bounded copy

- **WHEN** the buffer is at its cap and the user dumps it to a file
- **THEN** the text file contains the session header and every buffered line, and copying the whole
  buffer places the same text on the clipboard

### Requirement: Principal variation browser

While an engine is searching a position the app SHALL offer a browser of that engine's current
principal variation that steps the board through the line move by move — first, back, forward,
last, and jump-to-current-end — with an automatic mode that follows the engine's updates, without
writing anything into the game document, and SHALL offer one command that inserts the line being
browsed into the game as a new variation at the current move.

#### Scenario: Walk a live line

- **WHEN** the browser is opened with automatic mode on while the engine keeps extending its line
- **THEN** stepping shows each ply of the current line on the board, the game document stays
  unchanged, and closing the browser returns the board to the real position

#### Scenario: Insert as variation

- **WHEN** the user inserts the browsed line into the game
- **THEN** it appears as a variation at the current move, ordered after existing variations, and a
  further engine update does not mutate the inserted text

### Requirement: Self-filling position move list

The app SHALL provide a move list panel with content modes of book moves with their statistics,
all legal moves with the engine's answer line where available, and tablebase results for every
legal move where covered, plus an automatic mode that shows book moves when the position is
booked, tablebase results when covered, and legal moves otherwise; clicking a listed move plays it.

#### Scenario: Automatic mode switches with the position

- **WHEN** automatic mode is on and the user walks from a booked position to a five-piece
  endgame to a middlegame
- **THEN** the panel shows book statistics, then tablebase results, then legal moves, each switch
  within one second and with the source named in the panel header

#### Scenario: Engine answers are sufficient not best

- **WHEN** the legal-moves mode shows an answer line for each candidate move
- **THEN** each line is labelled as the engine's sufficient answer from the normal search rather
  than a dedicated per-move verification

### Requirement: User position values

The user SHALL be able to assign a value to the current position — a win/draw/loss verdict, a
centipawn score, or a mate distance — that full-game reports, sweeps, and lab runs then use as the
position's value instead of the engine's, that every affected output marks as user-set with the
move it was set on, and that can be listed, changed, and removed from one editor.

#### Scenario: Set value steers the report

- **WHEN** a value of lost-by-2-pawns is set on move 24 before a full-game report runs
- **THEN** the report uses the set value at that position, labels the entry user-set, and the
  editor lists it with the move number and current value

#### Scenario: Removal restores engine values

- **WHEN** a user-set value is removed and the report re-runs
- **THEN** no output references the removed value and the positions involved show engine values
