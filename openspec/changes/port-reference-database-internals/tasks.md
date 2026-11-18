# Tasks

## 1. Provenance and licensing gate

- [x] 1.1 Add `NOTICE.md` recording, per ported component, the reference file whose behaviour was studied and the statement that no source was copied; carry the MIT notice of the PGN tokenizer there.
- [x] 1.2 Add a CI check that no file added by this change contains a copyleft header and that every module named in `NOTICE.md` exists.

## 2. Partial dates (`chess-core`)

- [x] 2.1 Implement the packed partial-date type with year, month, day fields and unknown components, parsing the zero, question-mark, and omitted forms; cover each form in tests.
- [x] 2.2 Implement ordering and range matching where unknown components match, with the `1999.??.??` in-year and out-of-year cases from the spec as tests.
- [x] 2.3 Implement formatting that preserves which components were unknown and add the round-trip test.

## 3. Material and home-pawn signatures (`chess-core`)

- [x] 3.1 Implement the packed 32-bit material signature with the specified field widths and the colour-flip half swap; assert the layout and the flip rule in tests.
- [x] 3.2 Implement the per-file home-pawn signature and its mirrored flip test.
- [x] 3.3 Maintain both signatures incrementally on move and undo, saturating rather than wrapping over-cap counts, and add the incremental-equals-from-scratch test over a 1,000,000-position stream with the 100 ms budget.

## 4. PGN token coverage (`chess-core`)

- [x] 4.1 Extend the PGN lexer to the full token set: multi-line brace comments, semicolon comments, column-one percent comments, the clock escape, `$<n>` annotation glyphs, the six glyph suffixes, escaped quotes in tag values; keep the MIT notice reference in the module header.
- [x] 4.2 Add the tolerated-forms round-trip fixture and the unexpected-token line-number test, including the already-parsed-games-are-returned case.
- [x] 4.3 Add a nesting-depth test at depth 8 and deeper and record the accepted depth in the format note.

## 5. Header record and name dictionary (`game-db`)

- [x] 5.1 Write the native header record format note: field list, bit layout, 48-byte target, version byte, and the rebuild path for a version bump.
- [x] 5.2 Implement the packed header record with the documented field set and a size assertion, plus encode/decode property tests.
- [x] 5.3 Implement the interned name dictionary (player, event, site, round) with `u32` ids, a version byte, and a compaction pass; test id stability and reuse after compaction.
- [x] 5.4 Implement the shared collation: diacritic and case folding, leading-integer-aware round comparison, surname/given rotation key; table-test each rule.
- [x] 5.5 Build the collation prefix index over the dictionary and test prefix lookup against the surname-order completion case from the spec.

## 6. Filter and sort cache (`game-db`)

- [x] 6.1 Implement the byte-per-game filter with lazy allocation, an atomic match count, resize, and fill, with tests for the lazily-unallocated case.
- [x] 6.2 Add filter-combination ordering by expected selectivity and report rows scanned versus index used, asserted on a 1,000,000-game fixture.
- [x] 6.3 Implement the sort-rank cache: background build, interrupt, refcount, and the 8-bytes-per-row ceiling; test that an interrupted build leaves the previous order usable.
- [x] 6.4 Implement single-row re-placement on edit and the 100 ms re-sort and paging tests over a 100,000-row view.

## 7. Library sessions (`game-db`)

- [x] 7.1 Implement library opening with read-only detection from permissions and mount, and the no-write guarantee test for a read-only opening.
- [x] 7.2 Implement the cooperative lock file recording host and pid, the second-writer refusal naming the path, the read-only fallback, and stale-lock recovery after an unclean exit.
- [x] 7.3 Implement the session pool with at least 8 open libraries, least-recently-used closing with a flush, and the no-leftover-lock exit test.

## 8. ECO classification (`game-db`)

- [x] 8.1 Write the ECO code file format note and the parser for a user-supplied code file, reporting malformed entries by line.
- [x] 8.2 Build the half-move prefix trie with longest-match resolution and the two-entry spec case as a test.
- [x] 8.3 Implement the whole-library classification pass writing the code into the header record, with the 1,000,000-game 10-minute budget, the unclassified count, and the repeat-pass determinism test.

## 9. Rating list and name intelligence (`game-db`, `library-insight`)

- [ ] 9.1 Fix the rating-list file layout in a format note, choosing between the reference list layout and a documented CSV, then implement the parser for rating, title, federation, and birth date.
- [ ] 9.2 Implement rating-list lookup: fill a missing game rating with a source annotation, peak rating, and age at a date, with unknown attributes never rendered as zero.
- [ ] 9.3 Implement name completion over the collation index returning up to 10 ranked candidates, and the 50 ms budget test on a 1,000,000-game fixture.
- [ ] 9.4 Implement the spelling rule file (prefix, infix, suffix replacements) with malformed-rule reporting, and the systematic-spelling test.
- [ ] 9.5 Implement edit-distance-with-transposition correction bounded at distance 2, returning the distance, never auto-applying; test the transposed-name case.

## 10. Cross-table (`game-db`, `library-insight`)

- [ ] 10.1 Implement the cross-table builder over a filtered game set: per-player games, wins, draws, losses, score, average opponent rating, title, federation, and the per-pair clash records.
- [ ] 10.2 Implement mode detection with the documented thresholds and explicit override, and the automatic-selection test for both spec sets.
- [ ] 10.3 Implement tie-breaks for all-play-all, Swiss, and knockout, plus head-to-head resolution that reports the deciding rule.
- [ ] 10.4 Implement the 500-player and 60-round caps with excluded counts, and the truncation-report test.
- [ ] 10.5 Implement the plain-text, HTML, and typeset-source emitters and the cross-format equality test; assert single-page output for 20 players.

## 11. Binary book interchange (`game-db`, `opening-book`)

- [ ] 11.1 Add the position-key table as a versioned data file with its provenance note, and the golden-fixture test that every key of a real book file recomputes equal. Drop the interchange requirement if any key mismatches, and record the outcome here.
- [ ] 11.2 Implement the 16-byte big-endian entry reader with binary search by key, key verification by replay, illegal-move rejection, and the offset-named skip report; test the 100,000-entry 2-second budget.
- [ ] 11.3 Implement import into the native book store mapping weight to play count and leaving score, average rating, and last-used date unknown, with the unknown-not-zero panel test.
- [ ] 11.4 Implement export to the interchange format preserving the reserved field, and the export-import round-trip test.

## 12. Surfaces (`desktop`)

- [ ] 12.1 Add the cross-table panel with mode and sort controls, clash drill-down to a game, and the excluded-count header.
- [ ] 12.2 Wire name completion and correction into the search filter and game-entry header fields, including the accept-correction confirmation.
- [ ] 12.3 Add the ECO column and the classification status to the library views, and the rating-list source annotation to the rating display.

## 13. Across

- [ ] 13.1 Run `openspec validate port-reference-database-internals --strict` and fix every reported requirement and delta problem.
- [ ] 13.2 Run the full test suite including the 1,000,000-game and 100,000-row budget fixtures on the reference hardware, and record the measured numbers in the change directory.
