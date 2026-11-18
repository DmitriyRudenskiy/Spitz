# opening-book Specification

## Purpose

The opening book capability stores and serves human opening practice with statistics for both play and analysis. It defines a documented native book format, compiles books from large PGN databases, selects engine book moves with configurable weighting, and supports transposition-aware editing. Books are local files that the user imports, exports, and shares by hand; nothing in this capability talks to a network. Delivered in milestone M2 (change `add-analysis-and-training`).

## Requirements

### Requirement: Native Book Format

The system SHALL store books in a documented native format as a position-to-moves mapping in which every move entry carries play count, score percentage, average opponent Elo, and last-used date.

#### Scenario: Statistics round trip

- **WHEN** a book with known play counts, score percentages, average opponent Elo values, and last-used dates is saved and reloaded
- **THEN** all four statistics are identical for every move entry after the round trip
- **AND** the shipped format documentation describes the position-to-moves mapping and every stored field.

### Requirement: Book Panel

The app SHALL display a book panel with the stored statistics for the current position during play and during analysis.

#### Scenario: Statistics at the board

- **WHEN** the current position exists in the loaded book
- **THEN** the panel lists the booked moves with play count, score percentage, average opponent Elo, and last-used date
- **AND** when the position is not in the book the panel shows an explicit no-data state instead of an empty list.

### Requirement: Book Compilation

The app SHALL compile a book from a PGN database of at least 1,000,000 positions, merging duplicate occurrences of the same position into a single entry with aggregated statistics.

#### Scenario: Large import with duplicate merging

- **WHEN** the user compiles a book from a PGN database containing at least 1,000,000 positions with repeated games and transpositions
- **THEN** compilation completes and produces one entry per unique position with aggregated move statistics
- **AND** each duplicate occurrence increments the play counts instead of creating a duplicate move entry.

### Requirement: Weighted Selection

The engine SHALL select book moves by one of three selectable modes: weighted by play frequency, weighted by average opponent Elo, and weighted by performance (score percentage).

#### Scenario: Mode changes selection

- **WHEN** the engine must choose a book move in a position with several booked moves and the user switches between the frequency, Elo, and performance modes
- **THEN** the chosen move follows the weights of the active mode and the per-move weights are shown before the move is played
- **AND** repeated automated trials per mode produce move distributions consistent with that mode's weighting.

### Requirement: Book Editing

The book editor SHALL support adding, renaming, and penalizing moves and attaching comments to moves, with every edit applied transposition-aware to all occurrences of the same position.

#### Scenario: Transposition-aware edits

- **WHEN** the user penalizes a move or adds a comment in one position that also occurs in the book under a different move order
- **THEN** the penalty and the comment appear on every transposed occurrence of that position and move
- **AND** added and renamed moves remain consistent across all transpositions of the affected position.
