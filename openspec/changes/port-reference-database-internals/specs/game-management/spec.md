## ADDED Requirements

### Requirement: Sorted order cache

The app SHALL keep a cached sort order per library, active filter, and sort criteria so that paging,
re-sorting, and changing the sort direction of a 100,000-row result set complete within 100 ms without
re-sorting the whole set, and SHALL invalidate only the affected rows when a game shown in the result
set is edited. The cache SHALL be built in the background, SHALL be interruptible, and SHALL use at
most 8 bytes per row.

#### Scenario: Re-sort after the first sort

- **WHEN** the user sorts a 100,000-row result set by date and then re-sorts the same set by rating
  and back
- **THEN** each subsequent sort completes within 100 ms and no full re-sort of the set is performed

#### Scenario: Editing one game does not rebuild the order

- **WHEN** the user edits the date of one game in a date-sorted 100,000-row result set
- **THEN** the row moves to its new position within 100 ms and the remaining rows keep their positions

### Requirement: Library sessions

The app SHALL keep at least 8 game libraries open at once, SHALL detect a read-only library from its
file permissions and mount and open it without writing, SHALL hold a cooperative lock on each writable
library, and SHALL refuse a second writable session on the same library with a message naming the
locked path and offering a read-only opening. When the open-library limit is reached the app SHALL
close the least recently used library, flushing pending writes first.

#### Scenario: Second writer is refused

- **WHEN** a library already open for writing is opened again for writing
- **THEN** the second opening is refused with the locked path named and a read-only opening offered,
  and no bytes of the library change

#### Scenario: Read-only library stays intact

- **WHEN** a library on a read-only mount is opened and browsed and searched and the app exits
- **THEN** every file of the library keeps its previous size and modification time and no lock file is
  left behind

### Requirement: ECO classification

The app SHALL assign an ECO code to a game by matching its move sequence against a user-supplied ECO
code file, taking the longest matching sequence when several entries match, and SHALL report the count
of games left unclassified. The app SHALL reclassify an existing library in a single pass over
1,000,000 games in at most 10 minutes and SHALL produce the same codes when the pass is repeated with
the same code file.

#### Scenario: Longest match wins

- **WHEN** the code file contains an entry for a move sequence of 4 half-moves and another for 10
  half-moves that both match a game
- **THEN** the game is classified with the code of the 10 half-move entry

#### Scenario: Unclassified games are counted

- **WHEN** a library is classified and 3 percent of its games match no entry of the code file
- **THEN** the pass reports the unclassified count and those games keep an empty code rather than a
  guessed one
