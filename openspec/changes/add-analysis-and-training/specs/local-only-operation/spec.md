# local-only-operation Delta

## ADDED Requirements

### Requirement: No network access

The app SHALL provide analysis, play-versus-computer, books, tablebase lookup, and training entirely from local files, and SHALL NOT open network connections or require an account, a license check, or a vendor service.

#### Scenario: Network-disabled machine

- **WHEN** a full session runs on a machine with no network route to any host — a full-game report, a training session, a book lookup, and a tablebase probe
- **THEN** every feature completes from local files with no error prompts, no retry states, and no disabled-feature banners

#### Scenario: No listeners and no outbound calls

- **WHEN** a scripted session exercises play, analysis, training, and database search while the process is monitored
- **THEN** the app and its child processes have opened no listening socket and made no outbound connection
