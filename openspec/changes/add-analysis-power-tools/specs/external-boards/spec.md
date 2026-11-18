# external-boards Delta

## ADDED Requirements

### Requirement: Electronic board integration

The app SHALL integrate electronic chess boards over a transport abstraction (serial, USB, HID) with a setup dialog including automatic port detection, supporting moves entered on the physical board and positions displayed back to the board.

#### Scenario: Physical move appears in app

- **WHEN** the user plays a legal move on a connected electronic board
- **THEN** the move appears in the app's game within 200 milliseconds

#### Scenario: App move shown on board

- **WHEN** the computer opponent plays a move in the app
- **THEN** the move is displayed on the physical board's LEDs

### Requirement: Board position synchronization

The board integration SHALL push the current position to the board when a game is loaded, after a handicap or variant setup, and after a connection is re-established.

#### Scenario: Reconnect resync

- **WHEN** the board connection drops and is re-established mid-game
- **THEN** the app re-syncs the board to the current position before accepting further input

### Requirement: Legacy board support

The app SHALL support legacy electronic board protocols on a best-effort basis, with documented limitations per supported model.

#### Scenario: Legacy board configured

- **WHEN** the user selects a supported legacy board model in the setup dialog
- **THEN** basic move input works with the documented limitations shown in the dialog
