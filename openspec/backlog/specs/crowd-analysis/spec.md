# crowd-analysis Specification

## Purpose

The crowd-analysis capability connects every Spitz installation to a shared, community-built store of engine analyses for chess positions. Opted-in users contribute local analysis results to the store; submissions are deduplicated and pass a minimum-depth quality gate so the shared store stays compact and trustworthy. The client consults cached analyses offline and never blocks the interface on the network, and every contribution is pseudonymous so account identity is never exposed publicly. Delivered in milestone M3 (change `add-online-cloud-voice`).

## Requirements

### Requirement: Position query with full metadata

The system SHALL answer a query for any stored position with all matching analyses and their evaluation, search depth, engine class, contribution count, and last-update timestamp within 500 milliseconds at the 95th percentile under nominal load.

#### Scenario: Query returns stored analyses

- **WHEN** the user requests crowd analysis for a position that exists in the shared store
- **THEN** the client displays each stored analysis with its evaluation, depth, engine class, contribution count, and last-update timestamp
- **AND** the response completes within 500 milliseconds at the 95th percentile

#### Scenario: Query for an unanalyzed position

- **WHEN** the user requests crowd analysis for a position with no stored analyses
- **THEN** the client shows an explicit no-stored-analysis state instead of an error or stale rows

### Requirement: Opt-in contribution with minimum-depth quality gate

The system SHALL upload a user's local analysis only after that user has explicitly enabled contribution, SHALL reject any submission whose search depth is below 18, and SHALL report the rejection reason to the contributor.

#### Scenario: Contribution disabled by default

- **WHEN** a fresh installation completes its first local engine analysis
- **THEN** nothing is uploaded to the shared store until the user enables contribution in settings

#### Scenario: Shallow analysis rejected

- **WHEN** a user with contribution enabled submits an analysis produced at depth 12
- **THEN** the system rejects the submission, keeps it out of the shared store, and reports that the minimum depth of 18 was not reached

#### Scenario: Qualified analysis accepted

- **WHEN** a user with contribution enabled submits an analysis produced at depth 24
- **THEN** the system accepts it and it becomes visible to later queries for that position

### Requirement: Duplicate submission deduplication

The system SHALL deduplicate contributions by the combination of position and depth bucket so repeated submissions of equivalent analyses increase the contribution count without creating duplicate records.

#### Scenario: Repeated submission of an equivalent analysis

- **WHEN** a submission arrives that falls in the same position and depth bucket as an existing record
- **THEN** the system merges it into the existing record and increments the record's contribution count instead of adding a second record

### Requirement: Blunder verification against crowd consensus

The system SHALL compare the user's played move against crowd consensus for the position and SHALL flag the move when the evaluation disagreement exceeds a configurable threshold.

#### Scenario: Disagreement beyond the threshold

- **WHEN** a user plays a move whose loss against the crowd-consensus evaluation exceeds the configured blunder threshold
- **THEN** the client flags the move as a candidate blunder and shows the consensus evaluation for the position

#### Scenario: Threshold is configurable

- **WHEN** the user changes the blunder threshold in settings
- **THEN** subsequent move verification uses the new threshold without requiring a restart

### Requirement: Local cache with non-blocking offline fallback

The client SHALL cache query results locally, SHALL serve cached results read-only when the network is unavailable, and SHALL never block the user interface on a network round trip.

#### Scenario: Offline query hits the cache

- **WHEN** the user queries a position while offline and the position is present in the local cache
- **THEN** the client displays the cached analyses in read-only mode with a visible offline indicator

#### Scenario: Slow network does not freeze the interface

- **WHEN** a crowd-analysis query is in flight over a slow connection
- **THEN** the interface stays responsive and the result replaces the pending indicator asynchronously when it arrives

### Requirement: Pseudonymous contributions

The system SHALL attach only a pseudonymous identifier to every contribution and SHALL never expose account identity, email, or profile name in any query result or public record.

#### Scenario: Public record contains no identity data

- **WHEN** any user inspects a stored analysis record
- **THEN** the record exposes only the pseudonymous identifier and analysis metadata, with no field that resolves to an account identity
