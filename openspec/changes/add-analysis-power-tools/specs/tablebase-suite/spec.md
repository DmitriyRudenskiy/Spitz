# tablebase-suite Delta

## ADDED Requirements

### Requirement: Multi-format tablebase hosting

The app SHALL manage endgame tablebases in multiple formats — the fast native format plus Nalimov and Syzygy — by locating installed files, validating them, handing the path to engines that declare tablebase probing, and probing them itself for the analysis and training features that need per-move results.

#### Scenario: Backend selection

- **WHEN** the user configures the Nalimov backend while Syzygy files are also present
- **THEN** app-side probing uses the selected backend, reports correct win/draw/loss values for covered positions, and passes that path to the registered engine so its own probing uses the same files

#### Scenario: Missing files are named

- **WHEN** the configured tablebase path contains no usable files for a 5-piece position
- **THEN** the app states which format and path it checked and which piece counts are covered, and continues without probing

### Requirement: Endgame position explorer

The endgame tooling SHALL show for any covered position: the exact theoretical result, the result after every legal move, winning and drawing continuations, and distance to conversion or mate.

#### Scenario: Per-move results

- **WHEN** the user opens the endgame explorer on a covered position
- **THEN** every legal move is annotated with its theoretical result and the distance to the next conversion or mate

### Requirement: Evaluation-changing error flags

The analysis and endgame tooling SHALL immediately flag a played move whose theoretical value differs from the previous position's value (win to draw or loss, draw to loss, and the reverse).

#### Scenario: Draw-to-loss blunder flagged

- **WHEN** a played move turns a theoretically drawn endgame into a theoretical loss
- **THEN** the move is flagged with the value change and the maintaining move is indicated

### Requirement: Endgame position explorer search

The endgame tooling SHALL offer a search dialog for finding drill positions: side to move, result filters including white wins, black wins, draw, and mates in N for either side, with an option to ignore symmetric positions.

#### Scenario: Mate-in-three drill set

- **WHEN** the user searches with the filter "white mates in 3" and ignore-symmetric enabled
- **THEN** a drill set of distinct positions with that exact property is produced for training
