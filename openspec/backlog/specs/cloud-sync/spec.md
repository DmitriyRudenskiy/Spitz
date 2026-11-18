# cloud-sync Specification

## Purpose

The cloud-sync capability stores each account's games, training progress, opening repertoire, and a defined settings subset on the Spitz service and keeps every signed-in device up to date. It resolves concurrent edits with per-entity last-writer-wins and tombstones while retaining losing versions for recovery, queues mutations made offline for idempotent replay on reconnect, and enforces quota, transport, and storage security. Delivered in milestone M3 (change `add-online-cloud-voice`).

## Requirements

### Requirement: Per-account dataset synchronization

The service SHALL synchronize the account's games, training progress, repertoire, and settings subset across all signed-in devices of that account, applying each accepted change to every other online device within 5 seconds.

#### Scenario: Change propagates between devices

- **WHEN** the user saves a game on device A while device B is signed in to the same account and online
- **THEN** device B shows the new game within 5 seconds of the server accepting the write

#### Scenario: Non-synced settings stay local

- **WHEN** the user changes a setting outside the defined sync subset, such as a purely local appearance preference
- **THEN** the service does not synchronize it and the setting remains device-local

### Requirement: Conflict resolution with tombstones and version retention

The service SHALL resolve conflicting concurrent edits per entity using last-writer-wins, SHALL retain every losing version for 30 days for user-initiated recovery, and SHALL treat deletions as tombstones so a deletion is not silently undone by an older unsynchronized copy.

#### Scenario: Concurrent edits to one entity

- **WHEN** two devices modify the same entity while offline and both writes reach the server
- **THEN** the server keeps the later write as current, stores the losing version as a recoverable snapshot retained for 30 days, and marks the entity with a conflict record visible to the user

#### Scenario: Delete versus stale copy

- **WHEN** device A deletes an entity while offline and device B uploads a stale edit for that entity afterwards
- **THEN** the tombstone wins, the entity stays deleted, and the stale edit is preserved as a recoverable snapshot for 30 days

### Requirement: Offline mutation queue with idempotent replay

The client SHALL queue every mutation while offline, SHALL replay the queue automatically on reconnect, and SHALL make replay idempotent so retries and duplicate deliveries never create duplicate entities.

#### Scenario: Offline edits sync on reconnect

- **WHEN** the user creates or edits entities while offline and connectivity is restored
- **THEN** the client replays all queued mutations automatically within 30 seconds and each reaches the server exactly once despite retries

#### Scenario: Replay after a crash

- **WHEN** the client restarts between queueing a mutation and receiving its acknowledgment
- **THEN** the mutation is applied exactly once because each queued mutation carries a client-generated identifier that the server deduplicates

### Requirement: Storage quota and near-limit warnings

The service SHALL enforce a default per-account quota of 2 GB for synced data, SHALL display current usage in the client, and SHALL warn the account holder when usage crosses 80 percent of the quota and when a write is refused at the limit.

#### Scenario: Near-limit warning

- **WHEN** account usage crosses 80 percent of the 2 GB quota
- **THEN** the client shows current usage, the quota, and a near-limit warning in the sync settings

#### Scenario: Write refused at the limit

- **WHEN** an account at its quota limit attempts another upload
- **THEN** the service refuses the write with a clear quota-exceeded reason and existing data remains intact and readable

### Requirement: Transport and storage security

The service SHALL accept sync traffic only over TLS 1.3 or later and SHALL encrypt all stored sync data at rest.

#### Scenario: Cleartext connection refused

- **WHEN** a client attempts to connect to the sync endpoint without TLS 1.3 or later
- **THEN** the server refuses the connection and no sync payload is exchanged in cleartext

#### Scenario: Data encrypted at rest

- **WHEN** synchronized data is persisted on the server
- **THEN** it is stored encrypted so a reader of the raw storage backend cannot recover game or account content without the service key management

### Requirement: Per-device selective sync toggles

The client SHALL let the user enable or disable each synchronized dataset independently per device, and SHALL apply the toggles immediately to both uploads and downloads on that device.

#### Scenario: Repertoire sync disabled on one device

- **WHEN** the user turns off repertoire sync on one device
- **THEN** that device stops uploading repertoire changes and stops applying incoming repertoire updates while other datasets and other devices continue unchanged
