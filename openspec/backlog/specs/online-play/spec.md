# online-play Specification

## Purpose

The online-play capability provides rated and casual chess against other players through the Spitz service, with lobbies per time control, Glicko-2 rated pools, rating-window matchmaking, custom challenges, and live spectating with kibitzing. It carries chat with per-account rate limits and moderator tools, and routes engine-suspicion heuristics to human review instead of automated sanctions. Sessions survive brief disconnects, and an in-progress game is restored with full state within 60 seconds of a reconnect. Delivered in milestone M3 (change `add-online-cloud-voice`).

## Requirements

### Requirement: Lobby joining and pairing

The service SHALL let a signed-in account join a lobby for each supported time control and SHALL pair lobby members into games with colors assigned.

#### Scenario: Join a lobby and get paired

- **WHEN** a signed-in user joins the lobby for a time control with a compatible seeker present
- **THEN** the service pairs them into a game and both clients receive the game start with colors assigned

### Requirement: Rated pools with Glicko-2

The service SHALL maintain a separate rated pool per time control using Glicko-2, SHALL update both players' rating and rating deviation after every rated game, and SHALL mark an account provisional while its rating deviation exceeds 350.

#### Scenario: Rating update after a rated game

- **WHEN** a rated game finishes with a decisive or drawn result
- **THEN** the service updates both players' Glicko-2 rating and rating deviation for that pool and the new values appear on both profiles

#### Scenario: Provisional handling

- **WHEN** a player's rating deviation in a pool exceeds 350
- **THEN** the service marks that player's rating as provisional in pairings and profile displays

### Requirement: Rating-window matchmaking

The service SHALL pair seekers whose ratings are within plus or minus 200 points initially, SHALL expand the window progressively with wait time, and SHALL form at least 95 percent of pairs within 10 seconds while 10,000 seekers are queued concurrently.

#### Scenario: Window expands with waiting

- **WHEN** a seeker is not paired within the first matchmaking interval
- **THEN** the service widens the acceptable rating window beyond 200 points in scheduled steps until a compatible opponent is found

#### Scenario: Pairing throughput under load

- **WHEN** 10,000 seekers are queued concurrently across the time-control pools
- **THEN** at least 95 percent of pairs form within 10 seconds

### Requirement: Casual games and shareable challenges

The app SHALL let users create casual games and challenges with custom time control, color assignment, and rating-range restriction, and SHALL produce a shareable challenge link that an eligible recipient can accept.

#### Scenario: Custom challenge accepted via link

- **WHEN** a user shares a challenge link carrying custom time control and color settings and an eligible recipient opens it
- **THEN** the recipient can accept the challenge and the game starts with the requested settings

#### Scenario: Rating range blocks ineligible acceptor

- **WHEN** a recipient outside the challenge's rating range opens the challenge link
- **THEN** the app shows that the challenge cannot be accepted and no game is created

### Requirement: Spectating, kibitzing, and spectator chat

The app SHALL let users follow live games and broadcast events as spectators, SHALL provide kibitzing that displays evaluation from a spectator engine, and SHALL provide spectator chat for followed events.

#### Scenario: Follow a live game or broadcast

- **WHEN** a user selects an in-progress game or broadcast event to follow
- **THEN** the app streams the moves as they are played and shows the current position and move list
- **AND** the user can read and post in the event's spectator chat

#### Scenario: Spectator kibitz

- **WHEN** a spectator enables kibitzing for a followed game with a spectator engine configured
- **THEN** the app displays the engine's evaluation and best line to spectators without transmitting that analysis to the players

### Requirement: Chat and moderation controls

The service SHALL provide game and lobby chat with enforced per-account rate limits, SHALL let any user mute, report, and block another account, and SHALL give moderators tools to mute or silence accounts from a review queue.

#### Scenario: Per-account rate limit enforced

- **WHEN** an account sends chat messages faster than the enforced per-account limit
- **THEN** the service throttles the account and informs the sender that further messages are delayed or rejected

#### Scenario: Report, block, and moderator action

- **WHEN** a user reports another account's chat behavior
- **THEN** the service records the report with context in the moderator review queue and the reporting user can block the account immediately
- **AND** a moderator can apply a mute or silence from the queue, which takes effect for the affected account and is logged

### Requirement: Anti-cheat flagging for human review

The service SHALL run engine-match heuristics on rated games to identify accounts whose move choices statistically match third-party engine output, SHALL place flagged accounts in a human review queue with supporting evidence, and SHALL never apply bans, rating resets, or other sanctions from heuristics alone.

#### Scenario: Flagged account reaches human review only

- **WHEN** the anti-cheat heuristics flag an account during or after rated play
- **THEN** the account is placed in the human review queue with supporting evidence and no sanction is applied automatically
- **AND** any sanction requires a recorded human decision that the account holder can appeal

### Requirement: Reconnect to an in-progress game

The service SHALL keep a player's session alive across brief disconnects and SHALL restore an in-progress game with its full state within 60 seconds of a reconnect.

#### Scenario: Resume with full state

- **WHEN** a player disconnects from an ongoing game and reconnects within 60 seconds
- **THEN** the client restores the exact position, both clocks, the full move list, and any pending offers, and play continues without loss of state
- **AND** no forfeit or rating penalty is recorded for the interruption alone
