# board-ui Delta

## ADDED Requirements

### Requirement: Material balance strip

The board area SHALL show a material balance strip computed from the rules core with three modes —
off, difference only, every captured piece — where the difference names the surplus (for example a
rook for a bishop and knight), the captured-piece mode renders the pieces taken from each side in
the active piece set, and every mode has a text alternative readable by a screen reader.

#### Scenario: Difference and full list agree

- **WHEN** a game reaches a position where White has given up a bishop and knight for a rook and
  one pawn
- **THEN** difference mode shows "White: +1 pawn" style surplus and full-list mode shows exactly the
  four captured pieces under the correct sides, from the current piece set

#### Scenario: Strip is off cheaply

- **WHEN** the user sets the strip to off
- **THEN** no strip is laid out, the board and notation panels reclaim the space without reflow
  flicker, and the choice persists

### Requirement: Drag-time reply risk preview

While the user drags a piece, the board SHALL be able to mark the dragged-to square with a risk
tint — visually distinct warning levels for a move a cached shallow engine evaluation calls
suspicious or clearly bad — computed only from analysis the app already holds within a 150
millisecond budget, never blocking the drag, never playing a move the user has not committed, and
disabled while strict game mode is on.

#### Scenario: Cached evaluation tints the drop square

- **WHEN** the user drags a queen into a square where the app's cached evaluation of the move has
  dropped by more than two pawns
- **THEN** the drop square carries the highest warning tint and releasing on it still plays the
  move normally

#### Scenario: No cached value no tint

- **WHEN** no cached evaluation exists for the hovered destination or the budget is exceeded
- **THEN** no tint appears, the drag keeps tracking the pointer at full frame rate, and no engine
  search is started by hovering
