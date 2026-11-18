# game-management Delta

## ADDED Requirements

### Requirement: Position record browser

The app SHALL open a file of EPD position records as a browsable list: a preview board for the
selected record, the raw record text in an editable field that validates before loading, first /
previous / next / last / jump navigation, unknown opcodes preserved untouched on save, records that
fail validation listed with the failing text and never silently dropped, and commands to load the
selected record onto a board or hand the whole list to the endgame lab, batch position analysis, or
the training centre.

#### Scenario: Edit one opcode and load

- **WHEN** the user changes the `hmvc` opcode of the selected record and confirms
- **THEN** the preview board updates from the validated record, unknown opcodes of that record are
  unchanged, and saving writes a new file leaving the original intact

#### Scenario: Invalid record is listed not hidden

- **WHEN** an EPD file with 200 records where 7 have malformed piece placement is opened
- **THEN** the list shows all 200 rows, marks the 7 invalid with the failing text, and navigation
  skips them only when asked to

### Requirement: Live game file

The app SHALL keep a plain PGN copy of the current game at a user-chosen path when live-file mode is
on, rewriting the file atomically (write to a temporary file, rename over the target) within 200 ms
of every completed move, keeping it valid PGN for an external reader at every instant, and clearing
it when a new game starts.

#### Scenario: Reader never sees a half-written game

- **WHEN** a move completes while another program reads the live file
- **THEN** the reader sees either the previous complete game or the new one, never a truncated
  file, and the file's last-ply count matches the board within 200 ms

### Requirement: Unfinished game recovery

The app SHALL save an in-progress game to recovery storage within 30 seconds of any change, SHALL
offer every recovered game on next start with the position, clocks, and engine settings it had, and
SHALL delete a recovered game once the game is saved normally or discarded by the user.

#### Scenario: Crash mid-game

- **WHEN** the app is killed after a move was played 10 seconds ago
- **THEN** restarting offers the game at the move played, with clocks within 30 seconds and the
  same opponent settings, and accepting it continues the game with its engine sessions restarted
