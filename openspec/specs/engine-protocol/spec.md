# engine-protocol Specification

## Purpose

The engine-protocol capability hosts any UCI-compatible chess engine behind one manager — Spitz ships no engine of its own, so this layer is how the product gets every ounce of playing strength. It owns the handshake and option exchange, engine discovery, registration and health, crash recovery and logging, and concurrent engine sessions, so play, analysis, and training can drive any registered engine interchangeably. Delivered in milestone M1 (change `add-core-platform-mvp`).

## Requirements

### Requirement: UCI handshake cycle

The protocol layer SHALL complete the full UCI handshake cycle of uci and uciok, isready and readyok, ucinewgame, position, go, info, and bestmove, and SHALL pass the project UCI compliance test suite with every registered engine.

#### Scenario: Compliance suite

- **WHEN** a registered engine is driven through the project UCI compliance test suite
- **THEN** every handshake step completes in the required order and the suite passes

#### Scenario: Engine identification

- **WHEN** the handshake completes with an engine that sends id name and id author commands
- **THEN** the engine name and author are stored and shown with the session

#### Scenario: Ready synchronization

- **WHEN** isready is sent while the engine is still initializing after ucinewgame
- **THEN** readyok is returned only after initialization completes, so callers synchronize deterministically

### Requirement: UCI option handling

The protocol layer SHALL declare the engine's options and set and synchronize options of the types Spin, String, Check, Button, and Combo.

#### Scenario: Option round trip

- **WHEN** one option of each present type is set to a new valid value
- **THEN** the corresponding set command is sent in UCI form and reading the option state returns the new value
- **AND** an out-of-range Spin value is rejected locally before it reaches the engine

### Requirement: Engine manager

The protocol layer SHALL register and remove local engine executables, run health checks on them, and restart an engine automatically after a crash while restoring the last position and option values, capturing a per-session engine log.

#### Scenario: Registration lifecycle

- **WHEN** a local engine executable is registered and later removed
- **THEN** it appears and then disappears from the selectable engine list
- **AND** a health check against a missing or non-responsive executable reports failure without crashing the app

#### Scenario: Crash recovery

- **WHEN** an engine process crashes during a session
- **THEN** the manager restarts it automatically and resends ucinewgame, the last position, and all previously set options
- **AND** the session log captures all commands sent to and output received from the engine for diagnosis

### Requirement: Concurrent sessions

The protocol layer SHALL run at least 4 simultaneous engine instances with independent cancellation, applying each stop request to its own instance within 200 milliseconds.

#### Scenario: Independent cancellation

- **WHEN** 4 engine instances search concurrently and receive stop requests at different times
- **THEN** each instance delivers its own bestmove within 200 milliseconds of its own stop request
- **AND** the other instances continue their searches unaffected

### Requirement: Engine discovery and declared capability

The app SHALL detect engine executables on `PATH` and in the standard install directories of the host platform, offer them for registration, and record the option set each engine declares so that features depending on an absent option are hidden rather than silently ignored.

#### Scenario: Autodetection at first run

- **WHEN** the app starts for the first time with a supported engine installed but never registered
- **THEN** the engine is listed as a candidate with its name and version, and registering it with one action makes it usable for play and analysis
- **AND** an executable that fails the handshake is reported as unusable with the captured log, and no feature that needs it is offered

#### Scenario: Missing option is not offered

- **WHEN** the selected engine declares no `UCI_Elo` option and the user opens the strength settings
- **THEN** the Elo-limiting control is absent, the depth/node based controls are present, and the UI states which mechanism is in use for this engine

### Requirement: Tolerant stream handling

The protocol layer SHALL tolerate unknown, malformed, or out-of-order engine output without stalling or crashing the session.

#### Scenario: Malformed engine output

- **WHEN** an engine emits an unrecognized command or a truncated info line during a search
- **THEN** the line is written to the session log and the search continues to its normal end
