# chess-rules-core Specification

## Purpose

The chess-rules-core capability is the authoritative, exhaustively tested chess rules library used by every other Spitz capability. It owns board state, FEN parsing, legal move generation, game state detection, the SAN and UCI notation codecs, PGN exchange, and game clocks, and exposes them through a stable interface with no rendering or engine-search dependencies. Delivered in milestone M1 (change `add-core-platform-mvp`).

## Requirements

### Requirement: FEN parsing and serialization

The library SHALL parse and serialize FEN including piece placement, side to move, castling rights, en passant target, halfmove clock, and fullmove number, and SHALL reject malformed input with a precise error.

#### Scenario: FEN round trip

- **WHEN** a valid FEN string covering all six fields is parsed and the resulting position is serialized back to FEN
- **THEN** the output is byte-identical to the input
- **AND** castling rights, en passant target, halfmove clock, and fullmove number are preserved exactly

#### Scenario: Malformed FEN rejection

- **WHEN** a FEN string with a wrong field count, illegal piece characters, invalid castling flags, a bad en passant square, or out-of-range counters is parsed
- **THEN** parsing fails with an error naming the offending field and the concrete reason

### Requirement: Legal move generation

The library SHALL generate every legal move for any position, including castling only when rights exist and the king's start, transit, and destination squares are not attacked, en passant captures including pinned-pawn legality, promotions to queen, rook, bishop, and knight, and full pin restrictions.

#### Scenario: Perft fixtures

- **WHEN** perft runs from the standard start position to depth 6 and from the Kiwipete test position to depth 5
- **THEN** the node counts are exactly 119,060,324 and 193,690,690 respectively
- **AND** both fixtures run in the automated regression suite on every change to move generation

### Requirement: Game state detection

The library SHALL detect checkmate, stalemate, the 50-move rule, threefold repetition, insufficient material, and dead positions, and SHALL expose the distinction needed to support draw claims by the players.

#### Scenario: Repetition key

- **WHEN** the same piece placement and side to move recur while castling rights or the en passant target differ
- **THEN** the positions are not counted as repetitions of each other
- **AND** a threefold recurrence with identical piece placement, side to move, castling rights, and en passant rights is detected as threefold repetition

#### Scenario: Draw claims

- **WHEN** the halfmove clock reaches 100, or the material is reduced so that no sequence of legal moves can lead to checkmate
- **THEN** the library reports the claimable 50-move draw or the dead position respectively
- **AND** automatic results such as checkmate and stalemate are reported as finished outcomes, separate from claimable draws

### Requirement: Notation codecs

The library SHALL encode and decode SAN with file, rank, or combined disambiguation (for example Nbd2 and R1e2) and with check and mate suffixes, and UCI long algebraic including the promotion suffix (for example e7e8q), with round-trip correctness for every legal move.

#### Scenario: SAN disambiguation

- **WHEN** a move is encoded in a position where two or more identical pieces could reach the target square
- **THEN** the SAN output disambiguates by file, rank, or both as minimally required and carries the plus or hash suffix when the move gives check or mate

#### Scenario: Codec round trip

- **WHEN** every legal move of the perft fixture positions is encoded to SAN and to UCI long algebraic and then decoded again
- **THEN** each decoded move equals the original move in both codecs

### Requirement: PGN import and export

The library SHALL import and export PGN with tag pairs, comments, NAGs, nested variations, and multi-game streams, and SHALL report malformed input with the line number of the fault.

#### Scenario: Multi-game stream

- **WHEN** a file containing many concatenated games is imported
- **THEN** each game is delivered as a separate game with its tags, comments, NAGs, and variations intact

#### Scenario: Malformed input diagnostics

- **WHEN** a PGN stream contains an illegal token or an unbalanced variation at a known line
- **THEN** the error message contains that line number and a description of the fault
- **AND** games parsed before the faulty point are still returned

### Requirement: Game clocks

The library SHALL manage game clocks for the time controls none, sudden death, increment, and delay, and SHALL emit a flag-fall event the instant a player's remaining time reaches zero.

#### Scenario: Increment and delay

- **WHEN** moves are completed under an increment control and under a delay control
- **THEN** the increment is credited only after a completed move, and the delay consumes stored time only when a move takes longer than the delay

#### Scenario: Flag fall event

- **WHEN** a player's remaining time reaches zero
- **THEN** a flag-fall event is emitted immediately, identifying the player whose flag fell and the game time state
