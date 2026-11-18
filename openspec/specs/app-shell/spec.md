# app-shell Specification

## Purpose

The app-shell capability provides the Spitz workspace: dockable panels, navigation between boards and views, theming, settings, multi-board tabs, internationalization, and keyboard shortcuts. It frames every other capability and keeps workspace state stable across sessions. Delivered in milestone M1 (change `add-core-platform-mvp`).

## Requirements

### Requirement: Dockable panels

The app SHALL support dockable panels that can be dragged, resized, pinned, and floated across at least 5 named layout slots, and SHALL restore the active layout after a restart.

#### Scenario: Layout persistence

- **WHEN** the user arranges panels into a custom mix of docked, pinned, and floating panels and restarts the app
- **THEN** the same arrangement, slot assignment, sizes, and floating windows are restored

#### Scenario: Panel operations

- **WHEN** a panel is dragged to another slot, resized by its edges, pinned, or floated
- **THEN** each operation applies immediately and the panel keeps its content state while being moved

### Requirement: Themes and accent color

The app SHALL provide light and dark themes plus a user-defined accent color applied consistently across all panels.

#### Scenario: Theme consistency

- **WHEN** the light theme, the dark theme, or a custom accent color is selected
- **THEN** every open panel, dialog, and control reflects the change with no element left in the previous style

### Requirement: Settings management

The app SHALL persist settings across restarts, reset any individual section to its defaults on request, and import and export the full settings set to a file.

#### Scenario: Persistence and reset

- **WHEN** settings are changed, the app restarts, and then one section is reset to defaults
- **THEN** all other changed settings survive the restart and only the reset section reverts to defaults

#### Scenario: Settings file round trip

- **WHEN** the full settings set is exported to a file and later imported on a fresh profile
- **THEN** the imported settings match the exported ones exactly

### Requirement: Multi-board tabs

The app SHALL keep at least 8 concurrent game tabs open with fully independent board, engine, and clock state and no cross-talk between tabs.

#### Scenario: Independent tab state

- **WHEN** 8 game tabs are open while moves are played, engine analyses run, and clocks tick in different tabs
- **THEN** each tab retains its own position, engine output, and clock, and switching tabs never changes another tab's state

### Requirement: Internationalization

The app SHALL externalize all UI strings and ship English, German, and Russian locales, with locale switching that takes effect without a restart.

#### Scenario: Locale switching

- **WHEN** the locale is switched between English, German, and Russian with games and panels open
- **THEN** all visible UI strings change to the selected language immediately without a restart

#### Scenario: Fallback for missing strings

- **WHEN** a UI string has no translation in the active locale
- **THEN** the English text is displayed as a fallback and the missing key is written to the log

### Requirement: Keyboard shortcuts

The app SHALL make every command shortcut remappable and SHALL provide a cheat-sheet overlay listing all current shortcuts.

#### Scenario: Remapping and cheat sheet

- **WHEN** a command is rebound to a new key combination and the cheat-sheet overlay is opened
- **THEN** the overlay shows the new binding and the new combination triggers the command
- **AND** the previously bound combination no longer triggers that command unless still assigned to another command
