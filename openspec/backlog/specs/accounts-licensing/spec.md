# accounts-licensing Specification

## Purpose

The accounts-licensing capability provides account identity, entitlement, device management, and privacy controls for Spitz. A one-time purchase permanently unlocks local features, a premium plan unlocks server-side features, trials are configurable per campaign, and licensed features keep working through an offline grace period. Registration supports password and OAuth sign-in with optional multi-factor authentication, activations are limited to three concurrent devices, and privacy obligations for export, deletion, and voice audio are enforced. Delivered in milestone M3 (change `add-online-cloud-voice`).

## Requirements

### Requirement: Registration, login, and multi-factor authentication

The service SHALL support registration and login with email plus password, sign-in through at least two OAuth providers, sessions with refresh tokens, and optional time-based one-time password multi-factor enrollment and verification.

#### Scenario: Password login issues tokens

- **WHEN** a user logs in with valid email and password credentials
- **THEN** the service issues a session token and a refresh token and the client stays signed in across restarts until logout or revocation

#### Scenario: OAuth sign-in

- **WHEN** a user signs in through one of the supported OAuth providers
- **THEN** the service creates or links the account and issues the same session and refresh tokens as password login

#### Scenario: Multi-factor enforced only when enrolled

- **WHEN** a user with multi-factor enabled logs in with valid credentials
- **THEN** the service requires a valid one-time password code before issuing tokens
- **AND** users without enrollment can log in without a second factor

### Requirement: Entitlement tiers and configurable trials

The service SHALL grant permanent access to all local features on a completed one-time purchase, SHALL grant access to server-side features only while a premium plan is active, and SHALL support trial entitlements whose duration and scope are configurable per campaign.

#### Scenario: One-time purchase is permanent

- **WHEN** a user completes the one-time purchase
- **THEN** all local features remain unlocked on that account permanently without further payment

#### Scenario: Premium plan gates server features

- **WHEN** a user's premium plan lapses
- **THEN** server-side features become unavailable while previously purchased local features remain unlocked

#### Scenario: Campaign-configured trial

- **WHEN** a campaign defines a trial with a specific duration and feature scope and an account is created under it
- **THEN** the account receives exactly that trial duration and scope without a code change

### Requirement: Device activation limit and self-service deactivation

The service SHALL allow at most 3 concurrent device activations per entitlement, SHALL refuse a fourth activation while three are active, and SHALL let the user deactivate any device through self-service.

#### Scenario: Fourth activation refused

- **WHEN** a user tries to activate a fourth device while three devices hold active activations
- **THEN** the service refuses the activation and explains that a device must be deactivated first

#### Scenario: Deactivation frees a slot

- **WHEN** the user deactivates a device from account management
- **THEN** that device's activation ends and a new device can be activated in its place

### Requirement: Offline entitlement grace period

The client SHALL keep all licensed features fully working for at least 30 days after the last successful entitlement check and SHALL re-check entitlements automatically when connectivity returns.

#### Scenario: Grace period continues offline

- **WHEN** a licensed installation has had no successful entitlement check for 30 days or fewer
- **THEN** all licensed features continue to work without restrictions

#### Scenario: Automatic re-check on reconnect

- **WHEN** connectivity returns after an offline period
- **THEN** the client performs an entitlement check automatically and applies any entitlement state changes from the service

### Requirement: Data export and account deletion

The service SHALL provide self-service data export and account deletion, SHALL complete both within 30 days of the request, and SHALL honor deletion by removing or irreversibly anonymizing all personal data.

#### Scenario: Export completes within 30 days

- **WHEN** a user requests an export of personal data
- **THEN** the service delivers a complete machine-readable export within 30 days

#### Scenario: Deletion completes within 30 days

- **WHEN** a user confirms account deletion
- **THEN** within 30 days the service removes or irreversibly anonymizes the account and its personal data and the account can no longer authenticate

### Requirement: Voice audio consent

The system SHALL never store voice audio without the user's explicit consent, SHALL keep microphone processing in memory or on the local machine unless consent is granted, and SHALL delete stored audio after consent is withdrawn.

#### Scenario: No storage without consent

- **WHEN** a user who has not granted audio-storage consent uses voice features
- **THEN** captured audio is used for recognition only and no audio recording is persisted in server or client storage

#### Scenario: Consent withdrawal

- **WHEN** a user who had granted audio-storage consent withdraws it
- **THEN** the system stops storing new audio immediately and deletes previously stored audio from that user within 30 days
