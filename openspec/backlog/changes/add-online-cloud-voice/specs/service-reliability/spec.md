# service-reliability Delta

## ADDED Requirements

### Requirement: Service availability

The online service SHALL achieve 99.5% monthly availability for play and sync endpoints, excluding announced maintenance windows.

#### Scenario: Monthly SLO report

- **WHEN** a month closes with availability below 99.5% not caused by an announced maintenance window
- **THEN** the SLO report records the breach and an incident review is filed

### Requirement: Matchmaking latency

The server SHALL form rated pairs within their time-control pool in at most 10 seconds at the 95th percentile under peak load of 10,000 concurrent seekers.

#### Scenario: Peak-load matchmaking test

- **WHEN** the load harness simulates 10,000 concurrent seekers
- **THEN** at least 95% of pairs form within 10 seconds and the result is recorded in the load-test report

### Requirement: Durable rated results

Rated game moves and results SHALL be durably persisted through a write-ahead log before acknowledgement is sent to clients.

#### Scenario: Server crash mid-game

- **WHEN** the server crashes after a move was acknowledged to both clients
- **THEN** the move and any game-ending result survive the restart with no loss

### Requirement: Abuse controls

The service SHALL enforce per-account rate limits on chat, seek, and analysis requests, and SHALL route anti-cheat flags to human review rather than automatic bans.

#### Scenario: Chat flood attempt

- **WHEN** an account exceeds the chat rate limit
- **THEN** further messages are throttled and the event is logged for moderators
