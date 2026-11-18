# Native header record format

One fixed-size record per game, holding everything a library view, a filter pass, or a
cross-table needs without touching the game text. The field set is the proven one from the
reference suite's index entry, re-expressed as bytes here (`NOTICE.md`); nothing about this
layout is copied from its source.

**48 bytes per game**, 384 bits, packed LSB-first into six little-endian 64-bit words. A record
is a value, not a struct laid out in memory: `HeaderRecord::encode` writes the bits, `decode`
reads them back, so the on-disk form does not depend on a compiler's bitfield rules and no code
has to transmute.

| Field | Bits | Notes |
|---|---|---|
| `offset` | 48 | byte offset of the game record in the data file |
| `length` | 18 | bytes of the game record; 0 means the record is not stored yet |
| `white_id` | 28 | player dictionary id |
| `black_id` | 28 | player dictionary id |
| `event_id` | 28 | event dictionary id |
| `round_id` | 28 | round dictionary id |
| `site_id` | 32 | site dictionary id |
| `white_rating` | 12 | 0 = unknown; 4095 is the ceiling |
| `black_rating` | 12 | |
| `white_rating_kind` | 3 | see below |
| `black_rating_kind` | 3 | |
| `date` | 20 | year:11 month:4 day:5, the `chess_core::date::Date` sort key |
| `event_date` | 20 | |
| `result` | 2 | 0 unfinished, 1 white wins, 2 draw, 3 black wins |
| `flags` | 22 | one bit per flag character |
| `eco` | 16 | code table index, 0 = unclassified |
| `final_material` | 24 | the material signature of the last position |
| `comment_count` | 4 | 15 means "15 or more" |
| `variation_count` | 4 | 15 means "15 or more" |
| `nag_count` | 4 | 15 means "15 or more" |
| `half_moves` | 10 | 1023 means "1023 or more" |
| *spare* | 18 | always zero |

366 of the 384 bits are assigned. The spare bits stay zero so that two records holding the same
game hold the same bytes, which is what lets a rebuild be verified by comparison.

## Rating kind

`0` unknown, `1` Elo, `2` Fide or UsCF, `3` Brcp, `4` UsCF, `5` Issr, `6` program name. A rating
of zero always means unknown regardless of kind, and the panels render it as such rather than as
a playing strength.

## Flag bits

Bit position is the index of the flag character in this table: `c` promoted-to-pawn capture,
`O` olympic game, `N` ncm, `B` bcdb, `D` dibd, `P` dp, `U` up, `A` ap, `M` ip, `I` si, `S` to,
`F` ff, `W` w, `b` b, `X` x, `Y` y, `Z` z, then the six user-defined flags `1` to `6`. Twenty-two
bits, twenty-two characters, no gaps.

## Version byte and the rebuild path

The version is **one byte at the head of the index file**, not one per record: a byte per game
would be 1 MB per million games for information that is identical in every record. Version 1 is
this layout.

A version bump is a rebuild, not a migration in place. The data file holds the game text, which
is the actual record; the index is derived. So a reader that meets a version it does not know
reports the version it found and the range it understands, and the app rebuilds the index from
the games into a new file, verifies the record count, and swaps. Nothing tries to translate
field-by-field, which is how these formats usually lose data.

## Size assertion

`header_record::assert_layout` (and a test) pins `HEADER_RECORD_SIZE` to 48 and the encoded
length of a record to the same. A field added without dropping 18 bits elsewhere fails the test
rather than silently growing the file.
