# engine-protocol Delta

## ADDED Requirements

### Requirement: Protocol transcript

The protocol layer SHALL offer an on-demand raw transcript of one engine session — every line sent
and received in order with a timestamp — shown in a panel or appended to a plain-text file the user
picks, off by default, with the transcript tap costing at most 1 percent of the session's measured
nodes per second against the same session with the tap off.

#### Scenario: Transcript for a diagnosis

- **WHEN** the user enables the transcript, plays two moves, and disables it
- **THEN** the transcript contains the handshake, both `position` and `go` commands, and the engine
  replies in order with timestamps, and a session that never enabled it wrote no transcript file

#### Scenario: Tap does not throttle the engine

- **WHEN** the same search runs with the transcript off and then on
- **THEN** the measured nodes per second differ by at most 1 percent on the reference hardware

### Requirement: Engine-owned opening book deferral

When a registered engine declares that it serves its own opening book, the app SHALL offer a
per-side setting that leaves opening play to the engine, disabling the app book for that side while
it is on, and SHALL state in the book panel which side of the board is currently being served by
which book source; engines without the declaration SHALL never show the setting.

#### Scenario: Engine book takes over one side

- **WHEN** the setting is enabled for White on an engine that declares an own book
- **THEN** White's opening moves come from the engine without app-book queries, Black continues to
  use the configured app book, and the panel names both sources per side

#### Scenario: Setting absent without the declaration

- **WHEN** the selected engine declares no own-book capability
- **THEN** the setting is not offered and the app-book gate remains the only book source
