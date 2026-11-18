# PGN token coverage

`chess-core/src/pgn/lexer.rs` is the tokenizer; `chess-core/src/pgn/stream.rs` splits a stream
into games at the termination markers. This note records the two things the format has to state
but the code cannot: what a round trip guarantees, and the nesting limit.

## Tokens

`[Name "value"]`, `{brace comment}`, `; line comment`, `%escape comment` (column one only),
`$<n>` annotation glyph, the six suffix annotations `! ? !! ?? !? ?!`, move numbers, moves and
other symbols, `(`, `)`, and the four termination markers. Termination markers also accept the
`1:0`, `0:1` and `1/2:1/2` spellings seen in exported files.

Tag values are captured raw, with their backslash escapes intact, and run to the closing quote —
so `\"` and a `]` inside a value are both fine. `unescape_tag_value` decodes `\"` and `\\`.
`clock_centi_seconds` reads the `%t<centiseconds>` clock escape out of a comment.

## Round trip

`write_token` emits canonical form: one space between tokens, `N. ` for a move number, a newline
terminating a line or escape comment. Whitespace, `...` ellipses, and the punctuation of move
numbers are therefore normalised. **A round trip preserves the token sequence, not the bytes.**
Comment and tag-value text is the only thing preserved verbatim, because that text is the user's.

## Nesting

Variations nest to **32** levels; deeper input is a `VariationTooDeep` fault naming the line.
The PGN minimum a reader must handle is 16, and the reference suite has no limit at all — 32 is
the point past every real file checked while staying small enough that a malformed file cannot
make a caller recurse or allocate without bound. Raise the constant if a real library needs it.

## Not covered

EPD/FEPD lines (a first token holding seven `/`), and Latin-1 to UTF-8 normalisation. Neither is
a token: the first is a document form the importer can reject, the second is an encoding concern
of whatever reads the file. Add them when a file that needs one shows up.

## Faults

Every fault carries the line it was found on: a tag pair in the movetext section, an unterminated
tag pair or brace comment, `$` with no usable number, a suffix run longer than two characters, a
`%` escape outside column one, an unmatched `)`, nesting past the limit, and a character that
cannot start a token. `split_games` returns the games that parsed before the fault along with it.
