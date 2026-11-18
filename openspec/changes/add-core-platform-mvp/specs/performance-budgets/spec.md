# performance-budgets Delta

## ADDED Requirements

### Requirement: Rules-core throughput budget

The rules library SHALL reach at least 30,000,000 perft nodes per second per core on the reference hardware.

#### Scenario: Perft benchmark

- **WHEN** perft depth 6 from the start position runs on the reference hardware
- **THEN** throughput is at least 30,000,000 nodes per second per core and node counts match the fixtures exactly

### Requirement: Cold start budget

The packaged desktop app SHALL show a board ready to accept moves within 3 seconds of a cold launch on the reference hardware, with no engine registered.

#### Scenario: Cold start measurement

- **WHEN** the packaged app is launched cold on the reference machine
- **THEN** the first board accepts a move within 3 seconds of process start

### Requirement: Memory budget

The app SHALL cap default analysis memory at 2 GB for the whole process including engine child processes, and the installer SHALL not exceed 600 MB.

#### Scenario: Long analysis session

- **WHEN** infinite analysis runs for one hour at default settings against one registered engine
- **THEN** total memory of the app and its engine children stays below 2 GB throughout the session
