# game-management Specification

## Purpose

The game-management capability stores and organizes chess games in Spitz: the game document model, streaming PGN exchange, a documented native database format, header and position search, a virtualized game list, duplicate detection, and print and export paths. It scales from single game files to million-game reference databases. Delivered in milestone M1 (change `add-core-platform-mvp`).

## Requirements

### Requirement: Game document model

The app SHALL represent a game as a document holding moves, nested variations, comments, NAGs, glyphs, and embedded diagrams, with unlimited undo and redo of document edits.

#### Scenario: Editing with unlimited undo

- **WHEN** the user adds nested variations, comments, NAGs, glyphs, and an embedded diagram and then performs many undo and redo steps in one session
- **THEN** every step restores the exact prior document state with no limit on the number of steps

#### Scenario: Embedded diagram survives reload

- **WHEN** a diagram is embedded at a move and the game is saved and reopened
- **THEN** the diagram renders again at the same move with the same position

### Requirement: PGN streaming exchange

The app SHALL import and export PGN as a stream, including multi-game files, such that variations and comments survive a round trip.

#### Scenario: Streaming large files

- **WHEN** a multi-game PGN file larger than available memory is opened
- **THEN** games become browsable progressively while the rest of the file continues importing

#### Scenario: Round trip fidelity

- **WHEN** a game with nested variations, comments, and NAGs is exported to PGN and re-imported
- **THEN** the resulting move tree, comments, and NAGs are identical to the original

### Requirement: Native database format

The app SHALL provide a documented native open database format that holds at least 1,000,000 games per database and builds the header index in at most 10 minutes on the reference hardware.

#### Scenario: Million-game database

- **WHEN** a database containing 1,000,000 games is created on the reference hardware
- **THEN** the header index builds in at most 10 minutes
- **AND** the format documentation describes the on-disk layout so third parties can read and write the format

#### Scenario: Incremental growth

- **WHEN** games are appended to a database across several sessions and the database is reopened
- **THEN** every game is preserved and the header index remains valid without a full rebuild

### Requirement: Database search

The app SHALL search a 1,000,000-game database by header filters (player, ECO code, result, date range, Elo range, material) within 5 seconds and by board position within 60 seconds.

#### Scenario: Header filter budget

- **WHEN** a combined header filter over player, ECO code, result, date range, and Elo range runs on a 1,000,000-game database
- **THEN** the matching games are returned within 5 seconds

#### Scenario: Position search budget

- **WHEN** a position search runs over the same 1,000,000-game database
- **THEN** the matching games are returned within 60 seconds

### Requirement: Virtualized game list

The app SHALL present search results in a virtualized list with sorting and filtering whose interactions complete in under 100 milliseconds with 100,000 rows displayed.

#### Scenario: List responsiveness

- **WHEN** a result set of 100,000 rows is displayed and the user sorts, applies a filter, and scrolls
- **THEN** each interaction completes in under 100 milliseconds
- **AND** only the visible rows are rendered

### Requirement: Duplicate detection

The app SHALL detect duplicate games using header data plus a moves hash and SHALL show merge guidance for each duplicate group.

#### Scenario: Duplicate group guidance

- **WHEN** a database holds games with identical move trees and matching headers
- **THEN** they are grouped as duplicates with guidance on which header fields differ and how to merge them

#### Scenario: Move differences prevent false positives

- **WHEN** two games share header data but differ in their move trees
- **THEN** the moves hash distinguishes them and they are not merged as duplicates

### Requirement: Print and clipboard export

The app SHALL print games and export them to PDF with embedded diagrams, and SHALL copy PGN, FEN, and a diagram image to the system clipboard on request.

#### Scenario: Print and PDF with diagrams

- **WHEN** a game with embedded diagrams is printed or exported to PDF
- **THEN** the output shows headers, move text, and every diagram at its move with correct layout

#### Scenario: Clipboard copies

- **WHEN** the user invokes copy PGN, copy FEN, or copy diagram image for the current game or position
- **THEN** the clipboard receives the PGN text, the FEN string, or the rendered diagram image respectively
