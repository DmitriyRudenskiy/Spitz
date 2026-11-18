# classic-ui Delta

## ADDED Requirements

### Requirement: Tip of the Day

The app SHALL show a Tip of the Day dialog on launch with rotating tips, next/previous navigation, and a persistent disable option; tips SHALL ship as a versioned content pack.

#### Scenario: First launch tips

- **WHEN** the app starts for the first time
- **THEN** the Tip of the Day dialog shows the first tip with navigation controls and a disable checkbox that persists the choice

### Requirement: Customizable toolbar

The app SHALL let users add, remove, and reorder command buttons on the toolbar, switch between small and large icon sizes, and reset the toolbar to its default configuration.

#### Scenario: Toolbar edit

- **WHEN** the user removes a command from the toolbar and switches to large icons
- **THEN** the toolbar reflects the change immediately, and Reset restores the default configuration

### Requirement: Element visibility toggles

Every panel and toolbar SHALL be individually showable and hideable from both the main menu and a context menu, and the visibility state SHALL persist per layout.

#### Scenario: Minimal workspace

- **WHEN** the user hides all panels except the board
- **THEN** only the board remains visible and the layout is restored unchanged after an app restart

### Requirement: Context menus

The app SHALL provide context menus on the board, notation, analysis lines, and book panels, each offering the commands relevant to that panel.

#### Scenario: Analysis lines context menu

- **WHEN** the user right-clicks an analysis line
- **THEN** the context menu offers copy line, copy FEN, and send-to-engine commands for that line

### Requirement: Font scaling

The app SHALL offer a global UI font scale factor plus per-panel overrides, respect operating-system DPI scaling, and enforce minimum readable sizes for dense panels.

#### Scenario: Scaling up

- **WHEN** the user sets the global font scale to 150 percent
- **THEN** all text scales by that factor, dense panels keep readable minimum sizes, and no layout overflows its container

### Requirement: Detached board windows

The app SHALL open any game board in a separate top-level window, synchronized both ways with the workspace, supporting multiple detached windows for multi-monitor setups.

#### Scenario: Two detached boards

- **WHEN** the user detaches two game boards into separate windows on different monitors
- **THEN** moves in either view stay synchronized with the workspace and with each other

### Requirement: Built-in manual viewer

The app SHALL include a built-in manual viewer with a thematic table of contents, full-text search, and offline availability; every advanced tool SHALL have its own manual section.

#### Scenario: Finding a tool in the manual

- **WHEN** the user searches the manual for a tool added by a later change, such as selected-moves analysis
- **THEN** the viewer opens the corresponding section directly and the manual works fully offline

### Requirement: Theme preview

The board and piece theme chooser SHALL render live preview thumbnails using the actual rendering pipeline for each theme before it is applied.

#### Scenario: Preview before apply

- **WHEN** the user hovers over a board theme in the chooser
- **THEN** a thumbnail rendered with that exact theme and piece set is shown before the user applies it
