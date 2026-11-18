# endgame-lab Delta

## ADDED Requirements

### Requirement: Square sweep of one piece

The app SHALL offer a square sweep that removes one chosen piece from the current position, probes
or searches the resulting position for every legal square that piece could occupy, and writes the
per-square value onto the board: exact win/draw/loss plus distance to conversion or mate where a
configured tablebase covers the material, engine evaluation elsewhere. The sweep SHALL complete for
a six-piece tablebase-covered material in at most 10 seconds on the reference hardware and SHALL
offer the result as an EPD export with one record per evaluated square.

#### Scenario: Tablebase-backed sweep

- **WHEN** the user runs a square sweep on a white pawn of a K+Q+P vs K+P position covered by the
  configured tablebases
- **THEN** every square where the position stays legal shows the exact theoretical value and its
  distance, and squares that make the position illegal show no value
- **AND** the panel states that the values are tablebase-exact and names the material probed

#### Scenario: Engine-backed sweep when no tablebase covers the material

- **WHEN** a square sweep runs on a material the tablebases do not cover
- **THEN** each square is searched with the engine at the user's chosen per-square budget, the
  values are labelled as engine estimates with that budget shown, and estimates are never displayed
  in the same view as exact tablebase values unless the user turns the mixed view on

### Requirement: Wildcard position sweep

The app SHALL let the user define a family of positions by assigning every piece on the board a
scope of fixed square, its file, its rank, or the whole board, chosen for white to move, black to
move, or both, and enumerate every combination of the scoped pieces. The enumeration SHALL be
interruptible with partial results kept, SHALL skip illegal and symmetric-duplicate positions, and
SHALL report the number of positions examined when it finishes.

#### Scenario: File and board scopes

- **WHEN** a two-knights-versus-pawn family is defined with the pawn fixed and the three remaining
  pieces given whole-board scopes for both sides to move
- **THEN** the sweep enumerates the resulting position set, reports the examined count, and skips
  combinations where the side not to move is in check

#### Scenario: Missing tablebase material is named

- **WHEN** a wildcard sweep targets a material the configured tablebases do not contain
- **THEN** the sweep does not start and the dialog states the material, the formats and paths it
  checked, and which piece counts are covered

### Requirement: Sweep statistics

The sweep panel SHALL report over the enumerated set, split by side to move: won, drawn, and lost
counts with percentages to one decimal, the longest mate found with a clickable example position,
and the counts of mutual and one-sided zugzwang positions. Zugzwang counts SHALL be derived from
probing the same position with both sides to move and never inferred from a single probe.

#### Scenario: Longest mate with example

- **WHEN** a finished sweep found a longest mate of 73 moves
- **THEN** the statistic names both sides to move, and selecting it loads that exact position on a
  board and records it in the result set

#### Scenario: Zugzwangs from double probes

- **WHEN** a position is won for the side to move but would be drawn or stalemate with the other
  side to move
- **THEN** it is counted once in the matching zugzwang class and listing the class shows those
  positions

### Requirement: Filtered result export

The app SHALL filter a finished sweep by result, distance-to-mate band, and zugzwang class, and
export the selected subset to an EPD file whose comment field records the sweep definition (scopes,
side to move, filter) so an exported set can be reloaded as a position list by the lab, the
training centre, and batch position analysis.

#### Scenario: Mate-in-25 drill export

- **WHEN** the user filters for white-to-move mates in exactly 25 moves and exports
- **THEN** the EPD contains exactly those positions, every record carries the sweep definition in
  its comment, and loading the file into batch position analysis runs the same set

### Requirement: Compare two sweeps

The app SHALL run a second sweep definition alongside a finished one and show the two result sets
side by side: examined counts, per-class percentages with the difference, longest mates, and the
positions whose value differs between the sets, each selectable.

#### Scenario: Pawn one square further

- **WHEN** a KQPKQ family is swept with the defending king's scope set to one file and then to
  another
- **THEN** both statistics sets stay visible together with the percentage differences, and no
  re-run of the first sweep is needed to compare

### Requirement: Practice against perfect defence

The app SHALL start a practice session from any filtered result set, playing the user on the
winning or drawing side of each position against tablebase-perfect defence where the tablebases
cover every position of the set, with a per-position result recorded in a local lab history keyed
by material class, and SHALL fall back to a user-chosen engine level with the fallback stated when
the set is not fully covered.

#### Scenario: Tablebase defence session

- **WHEN** the user starts practice on 10 exported queen-versus-pawn positions that the tablebases
  cover
- **THEN** every defence move comes from the tablebases, each finished position records win, draw,
  or loss under the material class, and the session can be stopped without losing recorded results

#### Scenario: Partial coverage refuses silently perfect claims

- **WHEN** the practice set contains a position outside tablebase coverage
- **THEN** the session states that defence is engine-based for those positions, names the engine
  and level used, and records the session as engine-defended in the history
