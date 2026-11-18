# chess-rules-core Delta

## MODIFIED Requirements

### Requirement: Notation codecs

The library SHALL encode and decode SAN with file, rank, or combined disambiguation (for example Nbd2 and R1e2) and with check and mate suffixes, and UCI long algebraic including the promotion suffix (for example e7e8q), with round-trip correctness for every legal move. It SHALL additionally decode lenient notation entered by a human — missing or redundant disambiguation, lower-case input, omitted capture marks, `=` or omitted promotion marks, `0-0` and `O-O` castling spellings, and trailing check or mate marks — returning the set of legal moves a string matches so callers can ask the user to disambiguate, and it SHALL encode figurine and piece-letter display forms for the notation panel.

#### Scenario: SAN disambiguation

- **WHEN** a move is encoded in a position where two or more identical pieces could reach the target square
- **THEN** the SAN output disambiguates by file, rank, or both as minimally required and carries the plus or hash suffix when the move gives check or mate

#### Scenario: Codec round trip

- **WHEN** every legal move of the perft fixture positions is encoded to SAN and to UCI long algebraic and then decoded again
- **THEN** each decoded move equals the original move in both codecs

#### Scenario: Lenient input resolves to one move

- **WHEN** the strings "kxd5", "Kxd5", "Nf3", "nf3" and "e2e4" are decoded in positions where exactly one legal move fits each
- **THEN** each decodes to that single legal move without the user being asked

#### Scenario: Ambiguous or illegal input is never guessed

- **WHEN** a decoded string matches two or more legal moves, or none
- **THEN** the library returns the matching candidates or an empty result with the position in the input string where it failed, and plays nothing

#### Scenario: Display encodings

- **WHEN** a move is encoded in figurine form and in piece-letter form for a given locale
- **THEN** both encodings render the piece, disambiguation, and check marks for that locale, and an unsupported glyph set falls back to piece letters rather than to a missing-glyph box

## ADDED Requirements

### Requirement: Position statistics for coaching

The library SHALL compute deterministic structural statistics for a position: material balance per
piece type, pawn structure features (isolated, doubled, passed, backward pawns and pawn chains),
piece mobility and attack counts, king exposure counters, capture and check density in the position
and its recent history, evaluation volatility over a caller-supplied window, and a phase
classification.

#### Scenario: Statistics are deterministic

- **WHEN** the same position and the same history are measured twice, including after a save and
  reload of the game
- **THEN** every statistic has the same value, and the same position with the same history
  recomputed on another machine yields the same values

#### Scenario: Quiet and sharp positions separate

- **WHEN** the density and volatility statistics are computed over a fixed corpus of 100 quiet and
  100 sharp test positions
- **THEN** the median sharp-position score is at least twice the median quiet-position score

#### Scenario: Cost is bounded

- **WHEN** statistics are computed for a position during a live game
- **THEN** the p95 cost is at or below 2 ms on the reference hardware and the computation runs off
  the UI thread
