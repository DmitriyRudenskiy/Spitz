## ADDED Requirements

### Requirement: Partial dates

The library SHALL parse, compare, and format game dates whose month or day or both are unknown,
accepting the numeric forms with zero, question-mark, or omitted components, and SHALL preserve which
components were unknown through a round trip. A comparison against a date range SHALL match when the
known components of the partial date can fall inside the range and SHALL not match otherwise.

#### Scenario: Partial date inside its year

- **WHEN** the date `1999.??.??` is tested against the range 1999-03-01 to 1999-11-30
- **THEN** it matches, and the same date does not match the range 2000-01-01 to 2000-12-31

#### Scenario: Unknown components survive a round trip

- **WHEN** the date `2011.05.??.` is parsed, formatted, and parsed again
- **THEN** the year and month are unchanged, the day is reported as unknown, and the formatted text
  marks the day as unknown rather than as a number

### Requirement: Material and home-pawn signatures

The library SHALL compute a packed 32-bit material signature holding, for each colour, the piece
counts with 4 bits for pawns and 2 bits each for knights, bishops, rooks, and queens and no bits for
kings, and a home-pawn signature of one bit per file. Flipping the colours of a position SHALL equal a
defined swap of the two halves of the signature. Both signatures SHALL be maintained incrementally on
move and undo and SHALL equal the value computed from scratch for the same position.

#### Scenario: Incremental equals from scratch

- **WHEN** every position of a 1,000,000-position stream is reached by incremental moves and each is
  also computed from scratch
- **THEN** the two signatures are equal for every position and the incremental pass completes in at
  most 100 ms

#### Scenario: Colour flip is a half swap

- **WHEN** the signature of a position with white queens and rooks and black bishops is compared with
  the signature of the same position with the colours exchanged
- **THEN** the two halves of the signature are exchanged and the per-file pawn bits are mirrored

### Requirement: PGN token coverage

The library SHALL accept, in PGN move text, brace comments spanning multiple lines, semicolon
comments to end of line, percent comments to end of line when the percent sign starts a line, the
clock escape sequence inside comments, numeric annotation glyphs in `$<n>` form, the six standard
glyph suffixes, escaped quotation marks inside tag values, and variation nesting to depth 8 or more,
and SHALL reject any other unexpected token with its line number.

#### Scenario: Tolerated forms round trip

- **WHEN** a game using every tolerated form listed above is imported and exported
- **THEN** each form is preserved and no token is reported as a fault

#### Scenario: Unexpected token is located

- **WHEN** move text contains a token outside the tolerated set on a known line
- **THEN** the import reports that line number and the offending token, and games already parsed from
  the same stream are still returned
