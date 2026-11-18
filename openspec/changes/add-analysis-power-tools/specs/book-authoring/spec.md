# book-authoring Delta

## ADDED Requirements

### Requirement: Create books from scratch

The app SHALL let users create a new opening book from scratch and add positions and moves manually from any board or game, with no size limit — books of at least 1,000,000 positions SHALL be supported.

#### Scenario: Author a small book

- **WHEN** the user creates a new book and imports the moves of ten games plus manual additions
- **THEN** the book opens with all imported lines and manual moves present and correct

#### Scenario: Million-position book

- **WHEN** a book containing 1,000,000 positions is opened and queried
- **THEN** lookups return correct move statistics within interactive latency

### Requirement: Edit and extend any book

The book editor SHALL allow editing any book: add, rename, penalize, and delete moves; adjust weights and comments; import games or the current game into the book; export the book; and run utilities to count book moves and weed rarely played lines.

#### Scenario: Weed a book

- **WHEN** the user runs the weed utility with a threshold for minimum play count
- **THEN** all lines below the threshold are removed, a summary reports the removals, and the result is recoverable by undo

### Requirement: Per-side books in engine runners

Engine match and tournament runners SHALL support assigning a different opening book to each side with per-side use toggles.

#### Scenario: Different books per side

- **WHEN** a match is configured with book A for White and book B for Black
- **THEN** each engine plays only from its assigned book during the opening phase

### Requirement: Book parameters shape opening play

Book selection parameters — frequency, Elo, and performance weighting — SHALL be configurable per book and SHALL visibly change which moves the engine chooses in book positions.

#### Scenario: Weighting change observable

- **WHEN** the same start position is played repeatedly with frequency weighting and then with performance weighting
- **THEN** the distribution of chosen book moves differs measurably between the two configurations

### Requirement: Book learning in matches

The runners SHALL optionally update opening-book weights from game results during a match or tournament (book learning), per side, writing to the local book file the side is using.

#### Scenario: Book weight updated after loss

- **WHEN** a side loses a match game out of a book line with book learning enabled
- **THEN** that line's weight is reduced for subsequent games of the same match and the change is visible in the book editor
