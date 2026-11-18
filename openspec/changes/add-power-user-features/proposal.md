# Change: add-power-user-features

## Why

The current twelve capabilities cover the loop (play, analyse, train, store), but a complete desktop
chess program is judged on the surface around that loop: finding anything in a big library, annotating a
game richly enough to teach from it, seeing at a glance where a game turned, and knowing whether the
engine and hardware you picked are actually working. A feature inventory taken from the public user
manual of the analysis-focused reference product (316 pages, harvested into reading notes kept
outside this repository) was diffed against these specs and produced a concrete list of gaps that
are all local, GUI-side work.

## What Changes

- **Two new capabilities**
  - `game-commentary` — the annotation and teaching layer of a game document: graphic comment
    elements persisted per move (arrows, labelled squares, zones), a standard annotation symbol set,
    microphone-recorded spoken commentary attached to a move and replayed in step with the notation,
    and animated/printed publication formats (animated image, diagram sheet).
  - `engine-benchmarking` — measuring machines and engines instead of trusting marketing numbers:
    a one-run hardware report (rules throughput, per-engine nodes/second, tablebase probe latency),
    user-supplied test position sets in EPD where a position counts as solved/not solved, and a
    position-set match in which each side plays every position once as White and once as Black.
- **Game library depth** (`game-management`): free-form game tags with bulk editing, cap, and
  search; material-and-turn fragment search; checksummed secondary indexes for players, events,
  tags, ECO codes, comment text, material signatures, and positions; an
  auto-save/backup/archive cycle; a player rating list to fill missing PGN Elo; and exports that
  carry the visible annotation elements.
- **Analysis depth** (`analysis-workbench`): an evaluation profile graph over the whole game, a
  deep positional pass that ends in a written positional summary, a blunder-check pass that writes
  symbols, tags, and the engine's name into the game, and notation log columns (evaluation/depth,
  move time, expected move) that are recorded into the document.
- **Coaching during play** (`play-vs-computer`): opponent-threat overlay, hover dynamic hints,
  a static position brief with threats and structural features, a position-temperature measure,
  sparring sessions that start from a chosen position and call the trainer on deviation, and a
  per-move explanation in the post-game summary.
- **Engine workspace** (`engine-protocol`): per-engine process priority and resource class,
  hash reuse versus clearing between games, permanent background analysis with a user cap, and a
  live engine statistics view (nodes, hash fill, tb hits, nps graph).
- **Books and repertoires** (`opening-book`, `opening-trainer`): ECO-coded book display with
  win/draw/loss percentages and Elo-band filtering, a strongest-move marker per band, book
  maintenance operations (import games, import analysis, remove one source, Elo cutoff, per-band
  compilation), multiple book sources with a configured order, repertoire status colouring,
  repertoire built from the user's own games, and a free-play drill mode.
- **Training material** (`training-center`): importing curated lesson and exercise collections from
  local files, and resetting one discipline's history without touching the others.
- **Input and rendering** (`board-ui`, `chess-rules-core`): one-click move entry, typed notation
  with lenient parsing and disambiguation candidates, mouse-wheel navigation, right-click board
  menu, a clock column showing time spent per move, piece-letter and figurine labels on squares,
  the persisted graphic annotation elements, and asset rules for original piece sets and sounds.
- **Correctness fix carried over from the GUI-only decision**: three requirements still said
  "the engine SHALL" about GUI-owned behaviour (analysis panels, archetypes). They are rewritten to
  name the app and the selection layer, because we ship no engine.
- **Excluded on purpose** — present in the manual, out of this project: the cloud analysis service
  and its crowd corpus, the community move of the opening book, hosted engine marketplaces and
  auction systems, account activation and licensing, bundled commercial engines, and any bundled
  commercial book, lesson, or test-position content. Requirements are written so the user supplies
  those files. Full list with reasons in `design.md`.

## Capabilities

### New Capabilities

- `game-commentary`: annotation elements beyond text (graphics, symbols, spoken audio) and the
  publication formats that carry them out of the app.
- `engine-benchmarking`: hardware and engine measurement reports, test position sets, and matches
  driven by a position list.

### Modified Capabilities

- `board-ui`: move-input modes and expected-move preview, clock column, square labels, persisted
  graphic annotations, piece-set and theme asset rules.
- `chess-rules-core`: lenient SAN decoding with disambiguation candidates; figurine and letter
  encodings.
- `game-management`: tag search, fragment search, secondary indexes, save/backup
  cycle, rating list, annotation-aware exports.
- `analysis-workbench`: evaluation profile, deep positional pass, blunder check with document side
  effects, notation log columns, and the two misattributed engine requirements.
- `play-vs-computer`: threat and hint overlays, position brief, temperature, sparring from a
  position, per-move explanation.
- `engine-protocol`: priority and resource class, hash lifecycle, permanent analysis, engine
  statistics view.
- `opening-book`: ECO/statistics display, Elo bands, maintenance operations, multiple sources.
- `opening-trainer`: repertoire status colouring, repertoire from own games, free-play drill mode.
- `training-center`: lesson collection import, per-discipline reset.
- `style-engine`: archetype behaviour moved from the engine to the selection layer.

## Impact

- **Crates**: `chess-core` (lenient SAN, figurine encoding, position statistics for the brief and
  temperature), `game-db` (tags, indexes, backup cycle, rating list, export formats), `uci-host`
  (priority, hash lifecycle, permanent analysis, statistics stream), `move-select` (archetypes,
  sparring gate), `desktop` (all panels and overlays), new `assets/` tree for original piece sets,
  icons, and sounds.
- **File formats**: tags and annotation elements must survive PGN round trip (NAG/glyph mapping for
  symbols, documented sidecar for audio and graphics); secondary index files are a documented
  cache that can be deleted; audio files stay outside the game file and are referenced by path with
  a hash check.
- **Dependencies to decide in design**: animated-image encoder, audio capture and playback stack,
  and the audio container that ships by default.
- **No network code** is added. Every feature above reads local files and local processes.
- **Sequencing**: independent of M1-M3 internals but builds on `engine-protocol`, `game-management`
  and `board-ui`; the coaching overlay work reuses board-ui layers added in M1.
