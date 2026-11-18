# board-ui Delta

## MODIFIED Requirements

### Requirement: 2D board rendering

The UI SHALL render the 2D board with at least 4 piece sets and at least 8 color themes, optional
board coordinates drawn inside or outside the play area, optional piece-letter or figurine labels,
and crisp output at any board size and display DPI. Every shipped piece set SHALL be original
artwork under a redistributable free license, stored as scalable vector geometry or as raster art
prepared for at least twice the largest displayed square size, and drawn from theme colour slots so
one artwork works in every theme.

#### Scenario: Piece sets and themes

- **WHEN** the piece set and color theme pickers are opened
- **THEN** at least 4 piece sets and at least 8 color themes are offered and each choice applies to the visible board immediately

#### Scenario: Resolution independence

- **WHEN** the board is resized from the smallest window size to full screen on standard-DPI and high-DPI displays
- **THEN** pieces, squares, and coordinates stay sharp with no blurring or pixelation at every tested size

#### Scenario: Labels and figurines

- **WHEN** the user switches piece labels between off, piece letters, and figurine glyphs
- **THEN** the board re-renders within one frame and the chosen mode is remembered per theme
- **AND** the figurine mode falls back to letters for any locale or font that lacks the glyphs, with the fallback stated in the settings

#### Scenario: Assets are original and theme-independent

- **WHEN** the shipped visual assets are inspected
- **THEN** every piece set, icon, and sound file carries a license note naming its source and license, and no third-party chess program's artwork or font appears in the package

### Requirement: Move input and highlights

The UI SHALL accept moves by drag-and-drop, by click-click, by one-click destination entry when
that mode is on, and by typed notation, SHALL show legal-move hints while a move is being entered,
SHALL highlight the last move and any check, SHALL buffer a premove that is applied when the turn
changes to the player, and SHALL offer right-click board commands and mouse-wheel navigation of the
game.

#### Scenario: Hints and highlights

- **WHEN** a piece is selected for a click-click move or grabbed for a drag
- **THEN** its legal destination squares are shown as hints and, after the move, the from and to squares and any check are highlighted

#### Scenario: Premove buffer

- **WHEN** the player issues a premove while it is not their turn
- **THEN** the premove is stored and applied when the turn changes if it is still legal
- **AND** a premove that has become illegal is discarded when the turn changes

#### Scenario: One-click entry

- **WHEN** one-click move entry is enabled and the user selects a piece and then a destination
- **THEN** the move is committed on the second click without a third confirming click
- **AND** with one-click entry off, the same two clicks only select and then commit on the release, so a mis-click can be cancelled by dragging off the piece

#### Scenario: Typed notation entry

- **WHEN** the user types notation into the board or notation window while it is the app's turn to accept a move
- **THEN** a unique legal match is played, an ambiguous or illegal string is reported with the candidate moves it matched, and the board never plays a move the parser was unsure about

#### Scenario: Right-click board menu

- **WHEN** the user right-clicks a square
- **THEN** the menu offers the commands relevant to that square, including copy FEN, show piece statistics for the square's occupant, draw an annotation, and clear annotations from the square

#### Scenario: Wheel navigation

- **WHEN** the user scrolls the mouse wheel over the board or the notation list while reviewing a game
- **THEN** the displayed move steps forward or backward one ply per notch, repeats at most every 60 ms while the wheel keeps turning, and the game clock is never advanced by review navigation

### Requirement: Annotation overlay

The UI SHALL draw user-placed arrows, square highlights, labelled squares, square groups, and text
labels over the board, SHALL store the move glyphs !, ?, !!, ??, !?, and ?! with the game's moves,
and SHALL keep every drawn element attached to the position or move it belongs to.

#### Scenario: Arrows and highlights

- **WHEN** the user draws an arrow or a square highlight during a game or analysis
- **THEN** the annotation renders on the board and stays attached to the position or move it was drawn for

#### Scenario: Move glyphs

- **WHEN** the user applies any of the glyphs !, ?, !!, ??, !?, or ?! to a move
- **THEN** the glyph renders next to the move in the move list and is stored with the game across save and reload

#### Scenario: Drawing is immediate and erasable

- **WHEN** the user draws or erases an element while the engine is searching
- **THEN** the board updates within one frame, the engine keeps running, and erasing removes only the element under the cursor

## ADDED Requirements

### Requirement: Clock column in the notation list

The notation list SHALL show, per move, the time spent on the move and the remaining time, SHALL
mark flag fall and overtime, and SHALL let the user edit either value while reviewing a game.

#### Scenario: Time per move is visible

- **WHEN** a played game with clocks is opened
- **THEN** every move row shows its spent time and remaining time in the configured format and the
  totals per player equal the game's clock settings within one second

#### Scenario: Editing recorded times

- **WHEN** the user corrects the recorded time of a move during review
- **THEN** the row, the exported PGN clock annotations, and any evaluation profile overlay that
  displays time update to the corrected value

### Requirement: Board appearance settings

The app SHALL provide board appearance settings for orientation flip, coordinate visibility and
placement, piece labels, square and highlight colours, annotation element colours, and move
animation on/off with speed, SHALL persist them per theme, and SHALL apply every change without a
reload.

#### Scenario: Every setting applies live

- **WHEN** the user changes orientation, coordinates, colours, and animation while a board is open
- **THEN** all changes are visible on the next frame and no setting requires restarting the app

#### Scenario: Contrast is preserved

- **WHEN** the user picks custom square, highlight, and arrow colours
- **THEN** the app prevents a highlight or arrow colour from becoming indistinguishable from the
  square colours, keeping at least a 3:1 contrast ratio, and offers the colour-blind palette instead
