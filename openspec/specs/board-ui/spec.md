# board-ui Specification

## Purpose

The board-ui capability renders the chessboard in 2D and 3D and turns user input into moves on any game or analysis board. It covers piece sets and themes, drag-and-drop and click-click input with hints and premoves, annotation overlays, keyboard and screen-reader accessibility, and sound feedback. Full 3D polish such as advanced materials, lighting, and piece animation depth lands in milestone M2. Delivered in milestone M1 (change `add-core-platform-mvp`).

## Requirements

### Requirement: 2D board rendering

The UI SHALL render the 2D board with at least 4 piece sets and at least 8 color themes, optional board coordinates, and crisp output at any board size and display DPI.

#### Scenario: Piece sets and themes

- **WHEN** the piece set and color theme pickers are opened
- **THEN** at least 4 piece sets and at least 8 color themes are offered and each choice applies to the visible board immediately

#### Scenario: Resolution independence

- **WHEN** the board is resized from the smallest window size to full screen on standard-DPI and high-DPI displays
- **THEN** pieces, squares, and coordinates stay sharp with no blurring or pixelation at every tested size

### Requirement: Move input and highlights

The UI SHALL accept moves by drag-and-drop and by click-click, show legal-move hints while a move is being entered, highlight the last move and any check, and buffer a premove that is applied when the turn changes to the player.

#### Scenario: Hints and highlights

- **WHEN** a piece is selected for a click-click move or grabbed for a drag
- **THEN** its legal destination squares are shown as hints and, after the move, the from and to squares and any check are highlighted

#### Scenario: Premove buffer

- **WHEN** the player issues a premove while it is not their turn
- **THEN** the premove is stored and applied when the turn changes if it is still legal
- **AND** a premove that has become illegal is discarded when the turn changes

### Requirement: Annotation overlay

The UI SHALL draw user-placed arrows and square highlights over the board and store the move glyphs !, ?, !!, ??, !?, and ?! with the game's moves.

#### Scenario: Arrows and highlights

- **WHEN** the user draws an arrow or a square highlight during a game or analysis
- **THEN** the annotation renders on the board and stays attached to the position or move it was drawn for

#### Scenario: Move glyphs

- **WHEN** the user applies any of the glyphs !, ?, !!, ??, !?, or ?! to a move
- **THEN** the glyph renders next to the move in the move list and is stored with the game across save and reload

### Requirement: 3D board

The UI SHALL render a WebGL 3D board with at least 3 piece sets and an orbit camera, sustain at least 30 fps on an integrated GPU and 60 fps on a discrete GPU, and fall back automatically to the 2D board when no usable GPU is present.

#### Scenario: Frame rate targets

- **WHEN** a game is played on the 3D board with piece movement and camera orbit on the reference integrated-GPU and discrete-GPU machines
- **THEN** the average frame rate is at least 30 fps and 60 fps respectively

#### Scenario: No-GPU fallback

- **WHEN** WebGL is unavailable or 3D initialization fails
- **THEN** the UI switches to the 2D board automatically and tells the user why

### Requirement: Accessibility

The UI SHALL allow a complete game to be played with the keyboard alone, announce moves to screen readers through an ARIA live region, and provide colorblind-safe palettes.

#### Scenario: Keyboard-only play

- **WHEN** a user plays a complete game using only keyboard navigation and entry
- **THEN** every legal move can be selected, entered, and confirmed without a pointing device

#### Scenario: Screen reader announcements

- **WHEN** a move is played, check occurs, or the game ends
- **THEN** an ARIA live region announces the move in SAN and the resulting game state for screen readers

#### Scenario: Colorblind palettes

- **WHEN** a colorblind-safe palette is selected
- **THEN** last-move, check, hint, and annotation colors stay distinguishable under that palette

### Requirement: Audio feedback

The UI SHALL play distinct sounds for moves, captures, and checks, provide a master mute, and support per-event volume adjustment.

#### Scenario: Sound control

- **WHEN** a move, a capture, or a check happens with sound enabled
- **THEN** the distinct sound for that event plays at its configured per-event volume
- **AND** with the master mute enabled no sound plays for any event
