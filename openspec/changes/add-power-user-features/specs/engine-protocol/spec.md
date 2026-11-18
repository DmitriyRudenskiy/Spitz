# engine-protocol Delta

## MODIFIED Requirements

### Requirement: UCI option handling

The protocol layer SHALL declare the engine's options and set and synchronize options of the types Spin, String, Check, Button, and Combo. It SHALL persist the values the engine accepted in the engine profile, SHALL mark an option the engine does not publish as unavailable instead of sending it, and SHALL expose the resource options the app controls outside the protocol — scheduling priority, CPU affinity, and hash clearing — through the same profile screen.

#### Scenario: Option round trip

- **WHEN** one option of each present type is set to a new valid value
- **THEN** the corresponding set command is sent in UCI form and reading the option state returns the new value
- **AND** an out-of-range Spin value is rejected locally before it reaches the engine

#### Scenario: Unsupported option

- **WHEN** an engine does not publish an option that the profile holds a value for
- **THEN** the option is greyed out with the reason, nothing is sent for it, and the engine starts normally

### Requirement: Engine manager

The protocol layer SHALL register and remove local engine executables, run health checks on them, and restart an engine automatically after a crash while restoring the last position and option values, capturing a per-session engine log. For every running engine it SHALL report the process identifier, applied scheduling priority, applied CPU affinity, memory use, uptime, nodes per second, hash usage, and tablebase hits when the engine reports them, and SHALL keep a selectable history of those figures.

#### Scenario: Registration lifecycle

- **WHEN** a local engine executable is registered and later removed
- **THEN** it appears and then disappears from the selectable engine list
- **AND** a health check against a missing or non-responsive executable reports failure without crashing the app

#### Scenario: Crash recovery

- **WHEN** an engine process crashes during a session
- **THEN** the manager restarts it automatically and resends ucinewgame, the last position, and all previously set options
- **AND** the session log captures all commands sent to and output received from the engine for diagnosis

#### Scenario: Live statistics

- **WHEN** an engine has been searching for at least one second
- **THEN** nodes per second, node count, time, hash usage, current depth, and score are displayed and refresh at least twice per second

#### Scenario: Resource report

- **WHEN** the user opens the resource report for a running engine
- **THEN** the process identifier, applied scheduling priority, applied CPU affinity, memory use, and uptime are shown, and a value the platform refused is shown as refused with the OS error text

#### Scenario: Statistics history

- **WHEN** the user opens the statistics history for a running engine
- **THEN** a graph of nodes per second over the session is shown and the log file path is displayed with its current size

## ADDED Requirements

### Requirement: Process priority and CPU affinity

The app SHALL let the user set a scheduling priority and a CPU affinity mask per engine profile,
SHALL apply both to the engine process and its children when starting it, SHALL keep the app itself
at normal priority, and SHALL state when the platform refuses a requested value.

#### Scenario: Background engine at low priority

- **WHEN** an engine profile sets a background priority and that engine starts on Linux
- **THEN** the engine process reports the requested scheduling priority and the app keeps at least one
  core free when the affinity mask excludes one core

#### Scenario: Interface stays responsive under engine load

- **WHEN** every configured engine is searching at full thread count with a priority set
- **THEN** app input latency stays at or below 50 ms at p95 on the reference hardware during a
  60-second window

#### Scenario: Refused settings are reported

- **WHEN** the platform refuses the requested priority or affinity, for example because of a
  scheduler policy or a cgroup limit
- **THEN** the profile shows the applied value and the refused value with the OS error text, and the
  engine still starts

### Requirement: Permanent analysis

The app SHALL support a permanent analysis mode in which an engine keeps searching the current
position across the user's moves, reusing its existing search state instead of restarting from
scratch, and SHALL stop or redirect that search whenever a move is requested from it.

#### Scenario: Search state is reused

- **WHEN** permanent analysis is on and the user makes a move on the board
- **THEN** the engine continues from its current search state, shows a first result for the new
  position within 500 ms on the reference hardware, and the reported nodes per second does not drop
  below 80 % of its previous value

#### Scenario: A move request is never delayed

- **WHEN** a permanent search is running and the engine is asked to play a move
- **THEN** the permanent search is stopped first and the move request is answered within the timing
  the game or task requires

#### Scenario: Memory is bounded

- **WHEN** permanent analysis runs for 30 minutes
- **THEN** the engine's memory use stays within 10 % of the configured hash plus its baseline, and
  the app offers to clear the hash between unrelated positions

### Requirement: Hash lifecycle

The app SHALL keep an engine's transposition hash between the games of a match and between positions
of one analysis session, SHALL clear it on request and on new match, and SHALL report a hash-full
signal from the engine as a suggestion to raise the configured size.

#### Scenario: Hash kept between match games

- **WHEN** a match of five games runs with hash retention on
- **THEN** no clear-hash command is sent between the games, and with retention off exactly one
  clear-hash command is sent per new game

#### Scenario: Hash full is reported

- **WHEN** an engine reports that its hash is full
- **THEN** the app shows the figure with the currently configured size and the increase it suggests,
  and does not change the engine's configuration by itself
