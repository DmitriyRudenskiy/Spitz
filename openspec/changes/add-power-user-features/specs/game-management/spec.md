# game-management Delta

## MODIFIED Requirements

### Requirement: Database search

The database SHALL support search by player (with substring matching), by Elo range, by date range, by result, by ECO code, by tag or by the presence of a tag pattern, by the presence of comments or of a symbol on a move, by time-control class, by the number of moves, by a material-and-turn position filter, by a position or by a manoeuvre, with results paginated at most 500 rows per page and with the estimated row count shown before the search runs. Each filter SHALL name the accelerator it uses, and a filter with no accelerator SHALL report the number of records it scanned.

#### Scenario: Search by player and Elo band

- **WHEN** a search is run for a player substring with an Elo range
- **THEN** only matching games are returned and the count is shown

#### Scenario: Manoeuvre search

- **WHEN** the sequence "Ra1-a4" is searched in the open database
- **THEN** games containing that rook manoeuvre are returned

#### Scenario: Search by tag and comment

- **WHEN** a search is run for games tagged with a chosen tag, or for games whose comments contain a phrase
- **THEN** only games carrying that tag or that phrase are returned, and a phrase search over a
  1M-game database finishes in at most 5 seconds with an accelerator present

#### Scenario: Material and turn filter

- **WHEN** a search asks for queen against rook and bishop with White to move
- **THEN** only games containing such a position are returned, and the material specification that
  produced the result is shown next to the count

#### Scenario: Unindexed filter reports its cost

- **WHEN** a filter runs that has no secondary index for it
- **THEN** the result panel states the records scanned and suggests the index that would cover the
  filter

#### Scenario: Header filter budget

- **WHEN** a combined header filter over player, ECO code, result, date range, and Elo range runs on a 1,000,000-game database
- **THEN** the matching games are returned within 5 seconds

#### Scenario: Position search budget

- **WHEN** a position search runs over the same 1,000,000-game database
- **THEN** the matching games are returned within 60 seconds


### Requirement: Print and clipboard export

The app SHALL export games and selected positions through: single-position image (PNG/JPEG/SVG at 500 dpi, square or cropped), text (PGN/FEN/EPD/UCI), notation table (CSV/HTML), and game (PGN). Image export SHALL support transparent background and a configurable pixel size, SHALL apply the visibility filters of the active view, and SHALL include the visible annotation element classes and symbols of that view. Any export of a game with clock settings SHALL be rejected with a clear error.

#### Scenario: Image export filters

- **WHEN** an image is exported with hints and last-move highlight hidden and arrows and check highlight shown
- **THEN** the PNG contains only the allowed layers

#### Scenario: 500 dpi square image

- **WHEN** the user exports the current position as PNG at 500 dpi with square cropping
- **THEN** the PNG metadata says 500 dpi and its print size is 35×40 mm, and the board coordinates are not clipped

#### Scenario: Clock rejection

- **WHEN** any export is requested for a game with clock settings
- **THEN** the export is rejected with the reason and the offending game is identified

#### Scenario: Print and PDF with diagrams

- **WHEN** a game with embedded diagrams is printed or exported to PDF
- **THEN** the output shows headers, move text, and every diagram at its move with correct layout

#### Scenario: Clipboard copies

- **WHEN** the user invokes copy PGN, copy FEN, or copy diagram image for the current game or position
- **THEN** the clipboard receives the PGN text, the FEN string, or the rendered diagram image respectively

## ADDED Requirements

### Requirement: Game tags

The app SHALL let the user attach, edit, and remove free-form tags on games, individually or in
bulk from the game list, SHALL offer tags automatically produced by local passes such as a blunder
check, SHALL enforce a configurable cap of at most 100,000 distinct tags per database, and SHALL
make tags searchable and sortable in the game list.

#### Scenario: Bulk tagging

- **WHEN** the user selects 200 games in the list and applies a tag
- **THEN** all 200 games carry the tag within 2 seconds, the tag appears in the tag filter, and the
  action can be undone as one step

#### Scenario: Tag cap is enforced

- **WHEN** the number of distinct tags in the open database reaches the configured cap
- **THEN** creating another tag is refused with the current count and the list of least-used tags to
  clean up

#### Scenario: Tag list view

- **WHEN** the user opens the tag list
- **THEN** every tag is shown with its game count and can be renamed or deleted, with the number of
  games it will be removed from stated before confirming

### Requirement: Secondary search indexes

The app SHALL maintain secondary indexes over the open database for players, events, tags, ECO
codes, comments, material signatures, and positions, SHALL report for each index its size and
freshness, and SHALL rebuild any index that is stale or whose file no longer matches its stored
checksum.

#### Scenario: Index rebuild is visible

- **WHEN** an index is missing or its checksum does not match
- **THEN** the app offers a rebuild, shows progress with the records processed, and reports the
  resulting size; rebuilding 1M games takes at most 10 minutes on the reference hardware

#### Scenario: Indexes survive an unclean shutdown

- **WHEN** the app is killed during an index write and reopened
- **THEN** the index is detected as stale through its checksum and rebuilt, and no search returns an
  error

### Requirement: Database save and restore cycle

The app SHALL support saving the open library to a chosen local folder, restoring it from such a
folder, and archiving it as a single archive file, and SHALL report every component saved or
restored with its size and checksum.

#### Scenario: Save and restore round trip

- **WHEN** the user saves a library, makes further changes, and restores the saved folder
- **THEN** the game counts, tags, repertoires, books, training progress, and settings are exactly as
  at save time, and the report lists each component with its checksum

#### Scenario: Single-file archive

- **WHEN** the user archives a library
- **THEN** one archive file is written containing every component plus a manifest naming them, and a
  restore from that archive reports any component whose checksum does not match instead of partially
  restoring

#### Scenario: Restoring over a busy library

- **WHEN** a restore is requested while analysis, a match, or an engine job is running
- **THEN** the restore is refused with the list of running jobs and offers to stop them first

### Requirement: Player rating list

The app SHALL maintain a per-database player list with the name, the number of games, and the
highest and average Elo recorded for that player, SHALL update it as games are added or removed, and
SHALL use it to complete player names and Elo fields during search and game entry.

#### Scenario: List is derived from the games

- **WHEN** 1,000 games from two players with known Elo values are imported
- **THEN** the list shows both players with their game counts, highest and average Elo, and the
  figures match what the game headers contain

#### Scenario: Stale values are corrected

- **WHEN** the user begins typing a player name that the list already knows
- **THEN** the Elo field offers the stored value and marks a manually entered value that differs from
  it by more than 250 points as needing confirmation
