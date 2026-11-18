# Provenance and licensing notice

Scope: the OpenSpec change `port-reference-database-internals`.

The behaviour of the components listed below was studied by reading the Scid chess database suite
(commit `ad78554b4678c62652a9f3632d3dc762ae167965`), which is distributed under the GNU General
Public License (see its `COPYING`). **No source line from that suite is copied into this
repository**, with the single exception marked MIT in the second section. Algorithms, on-disk byte
layouts, and edge-case rules are re-expressed as original Rust, per decision D1 of the change's
`design.md`.

Naming note: boundary 3 of `openspec/config.yaml` keeps third-party product names out of specs, code,
UI strings and product docs. This file is a legal provenance record, not product documentation, so it
names the upstream project. Every other document refers to it as "the reference suite".

`scripts/check-provenance.sh` enforces this file in CI: an `implemented` row must name a module that
exists, and no source file of this project may carry a copyleft header.

## Studied components

| Module | Status | Behaviour studied in | Copied source |
|---|---|---|---|
| `chess-core/src/date.rs` | implemented | `src/date.h` | none |
| `chess-core/src/matsig.rs` | implemented | `src/matsig.h`, `src/matsig.cpp` | none |
| `chess-core/src/pgn/lexer.rs` | implemented | `src/pgn_lexer.h` | MIT-licensed file, notice below |
| `chess-core/src/position_key.rs` | planned | `src/polyglot/hash.cpp` | none; the key table is interchange-format data carried as a data file with its own note |
| `game-db/src/header_record.rs` | implemented | `src/indexentry.h` | none |
| `game-db/src/name_dict.rs` | implemented | `src/namebase.h` | none |
| `game-db/src/collation.rs` | implemented | `src/misc.h` (`strCaseCompare`, `strCompareRound`) | none |
| `game-db/src/fold_data.rs` | implemented | none; generated from Unicode NFD data, not from the reference suite |
| `game-db/src/filter.rs` | implemented | `src/hfilter.h` (`Filter`) | none |
| `game-db/src/sort_cache.rs` | implemented | `src/sortcache.h`, `src/sortcache.cpp` | none |
| `game-db/src/session.rs` | implemented | `src/dbasepool.cpp`, `src/filemap.h` | none |
| `game-db/src/eco.rs` | planned | `src/scidbase.cpp` (ECO read and classification), the `.eco` code-file layout | none |
| `game-db/src/crosstable.rs` | planned | `src/crosstab.h`, `src/crosstab.cpp` | none |
| `game-db/src/rating_list.rs` | planned | `src/spellchk.cpp` (`PlayerElo`, `PlayerInfo`) | none |
| `game-db/src/name_correct.rs` | planned | `src/spellchk.h` (`NameNormalizer`), `src/spellchk.cpp` | none |
| `game-db/src/book_binary.rs` | planned | `src/polyglot/book.cpp`, `src/pbook.cpp` | none |

A row moves from `planned` to `implemented` in the same commit that adds the module.

## MIT-licensed exception

`chess-core/src/pgn/lexer.rs` is the one module whose structure follows an upstream file closely. That
file, `src/pgn_lexer.h` of the reference suite, carries this notice, reproduced as required:

> Copyright (C) 2018 Fulvio Benini.
>
> Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
> associated documentation files (the "Software"), to deal in the Software without restriction,
> including without limitation the rights to use, copy, modify, merge, publish, distribute,
> sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
> furnished to do so, subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all copies or
> substantial portions of the Software.
>
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
> NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
> NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
> DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
> OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

## Not ported

The reference suite's own on-disk database container (its index, name base, and game codecs) is not
read or written here, per decision D2. Its bundled engines and its Tcl front end are not ported at
all.
