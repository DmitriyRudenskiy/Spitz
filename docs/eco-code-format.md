# ECO code file format

A code file is a plain text file the user supplies, holding the opening sequences and the codes to
give them. It is not a database and not a proprietary format: one entry per line, readable, and
diffable.

```
# comment lines start with a hash, blank lines are ignored
A00 1. b3
B92 1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6
C20 e4 e5            # move numbers are optional
```

## The line

`CODE <move sequence>`

- **Code** — 2 to 6 characters, starting with a letter, the rest letters or digits. Compared and
  stored upper case, so `b92` and `B92` are one code.
- **Move sequence** — one or more moves in standard algebraic notation, in the standard starting
  position. Move numbers (`1.`, `1...`), ellipses (`...`), annotation suffixes (`!`, `?`, `!!`, `!?`,
  `?!`, `+`, `#`) and NAGs (`$6`) are ignored, so the same move written any usual way matches the
  same entry. Castling is castling whether it is written `O-O` or `0-0`.

A line with no code, no moves, or a code that is not a code is reported with its line number and
skipped; one bad line never stops the rest of the file loading.

## Matching

An entry matches a game when the entry's moves are a prefix of the game's **main line** — moves
inside variations are not part of the sequence. When several entries match, the longest wins: with
entries of 4 and 10 half-moves both matching, the 10 half-move entry's code is the game's code. A
game matching nothing keeps an empty code, and the pass reports how many games that was.

Two lines giving different codes for the same sequence are a mistake in the file: the first line
wins and the second is reported as a duplicate.

## Codes in the header record

The header record holds 16 bits per game, which is an index, not the code itself. The index is the
code's position in the file's distinct codes sorted as text, plus one; zero means unclassified. The
consequence is stated plainly: the ids are stable for a given code file, and classifying a library
with a *different* code file rewrites every code. The code file is small — a few thousand lines —
so reclassifying is cheap, and that is the repair path.
