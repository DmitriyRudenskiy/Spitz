## ADDED Requirements

### Requirement: Binary book interchange

The app SHALL import and export opening books in the interchange format of fixed 16-byte big-endian
entries sorted by position key, each entry holding an 8-byte position key, a 2-byte packed move, a
2-byte weight, and a 4-byte field the app preserves unchanged, locating a position by binary search on
the key. On import the app SHALL recompute every position key from the move sequence it replays and
SHALL report and skip entries whose key does not verify or whose move is illegal in the position.

#### Scenario: Import verifies every key

- **WHEN** a book of 100,000 entries is imported
- **THEN** import completes within 2 seconds, every accepted entry has a recomputed key equal to its
  stored key, and the accepted and rejected counts are reported

#### Scenario: Corrupt entry is skipped

- **WHEN** a book contains an entry whose stored key does not match the position reached by replaying
  its move sequence
- **THEN** that entry is skipped, the entry's byte offset is named in the report, and the remaining
  entries are imported

#### Scenario: Export round trip

- **WHEN** a book is exported to the interchange format and imported again
- **THEN** every position has the same set of moves with the same weights as before the export

### Requirement: Imported weight mapping

The app SHALL map an imported entry weight onto the native move statistics as its play count, SHALL
mark score, average opponent rating, and last-used date as unknown for imported entries, and SHALL
display them as unknown rather than as zero.

#### Scenario: Unknown statistics are not shown as zero

- **WHEN** a book imported from the interchange format is shown in the book panel
- **THEN** play counts equal the imported weights and the score and rating columns show an unknown
  marker for entries that carry no such data
