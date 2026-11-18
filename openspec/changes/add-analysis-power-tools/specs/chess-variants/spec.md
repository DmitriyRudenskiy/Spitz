# chess-variants Delta

## ADDED Requirements

### Requirement: Chess960 rules support

The rules core SHALL support Chess960 (Fischer random) positions, including position-dependent castling rights and castling legality, with FEN round-trips using the extended castling encoding.

#### Scenario: Castling from non-standard squares

- **WHEN** a Chess960 position with king on e1 and rooks on h1 and a1 is loaded and the king castles toward each rook where legal
- **THEN** both castling moves are generated correctly for their actual files and the resulting positions round-trip through FEN without loss

#### Scenario: Chess960 perft fixture

- **WHEN** perft is run on the fixed Chess960 test positions
- **THEN** node counts match the recorded fixtures exactly

### Requirement: Random Chess960 start positions

The system SHALL generate valid Chess960 start positions uniformly, with exactly 960 distinct positions, deterministically seedable.

#### Scenario: Enumeration completeness

- **WHEN** the generator enumerates all start positions without a seed
- **THEN** exactly 960 distinct valid positions are produced, each with correct castling rights

### Requirement: Variant play

Play against the computer SHALL support starting games from a random or chosen Chess960 position, and the opponent's opening-knowledge setting SHALL degrade gracefully when no book data exists for the position.

#### Scenario: Game from random start

- **WHEN** the user starts a new game with the Chess960 random option
- **THEN** the game begins from a generated Chess960 position and castling works per the variant rules

### Requirement: Variant engine matches and analysis

Engine matches, tournaments, and the analysis tools (full report, histogram, shootout, batch analysis) SHALL accept Chess960 positions and games.

#### Scenario: Match from variant position

- **WHEN** an engine match is configured with a Chess960 start position
- **THEN** all games of the match start from that position and the results are stored with correct variant metadata

### Requirement: Variant-capable third-party engines

Engine orchestration SHALL enable the UCI variant option for Chess960 when the selected engine declares support, and SHALL reject incompatible engines for variant games with a clear error before the game starts.

#### Scenario: Incompatible engine rejected

- **WHEN** a match is started with an engine that does not declare Chess960 support
- **THEN** the runner shows an explanatory error naming the engine and offers a substitute instead of failing mid-match
