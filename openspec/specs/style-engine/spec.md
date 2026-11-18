# style-engine Specification

## Purpose

The style engine shapes the computer opponent's play and commentary so that it feels human. Because the engine is a third-party UCI process, style is realized as selection policy in the move-selection layer — how a move is chosen from the lines the engine reports, and which openings the book is allowed to play — not as engine evaluation weights. On that basis it exposes continuous style axes with measurable effects on move preference, ships a set of named style presets, derives historical-style personas from public-domain game corpora under neutral configurable names, and implements behavior archetypes with distinct, specifiable tendencies. Delivered in milestone M2 (change `add-analysis-and-training`).

## Requirements

### Requirement: Style Axes

The app SHALL expose four continuous style axes (aggression, risk tolerance, exchange orientation, and positional-versus-tactical bias), each in the range 0 to 100, applied by the move-selection layer to the lines reported by the engine, and SHALL document the measured effect of each axis together with the size of that effect.

#### Scenario: Axis effect is observable

- **WHEN** the same suite of calibration positions is played twice with identical settings except one axis moved from 50 to 90
- **THEN** the move and line preferences shift in the direction documented for that axis
- **AND** the axis values in force, the engine id, and the selection seed are stored with the game so any style can be reproduced exactly

#### Scenario: Weak axis is not shipped

- **WHEN** an axis changes the chosen move on fewer than 5% of the calibration positions between the 20 and 80 settings
- **THEN** that axis is removed from the user-facing controls rather than shipped with an unmeasurable effect

### Requirement: Style Presets

The app SHALL ship at least 12 named style presets, each selectable per game and per analysis session.

#### Scenario: Preset selection

- **WHEN** the user opens the preset picker before a game or an analysis session
- **THEN** at least 12 named presets are listed and each shows its style vector
- **AND** selecting a preset applies its axis values to the session immediately.

### Requirement: Personas

The app SHALL ship at least 5 historical-style personas derived from public-domain game corpora, each with a style vector and a phrase bank, all shipped under neutral configurable names.

#### Scenario: Persona configuration and naming

- **WHEN** the user lists the personas in the new-game dialog
- **THEN** at least 5 personas are offered, each showing its style vector and phrase bank
- **AND** every persona ships with a neutral default name that the user can rename, and no name, description, or phrase references a specific player.

### Requirement: Live Reactions

The system SHALL emit persona comments on evaluation swings of at least 1.5 pawns and on tactical shots, drawing on at least 50 phrases per persona and rate-limiting comments to prevent spam.

#### Scenario: Swing comment with rate limit

- **WHEN** during a game with an active persona the evaluation swings by at least 1.5 pawns or a tactical shot appears on the board
- **THEN** the persona emits a comment drawn from its phrase bank
- **AND** no further comment is emitted within the configured rate-limit interval even if further qualifying events occur.

#### Scenario: Phrase bank coverage

- **WHEN** a persona's phrase bank is inspected during a test run
- **THEN** it contains at least 50 distinct phrases
- **AND** the phrases cover both evaluation-swing situations and tactical-shot situations.

### Requirement: Archetype Opponents

The engine SHALL implement four behavior archetypes with specifiable behaviors: an attacker that prefers forcing lines, a coward that avoids complications, a swindler that keeps setting traps in lost positions, and an endgame wizard that steers toward favorable endgames.

#### Scenario: Attacker and coward behavior

- **WHEN** the attacker and the coward each play a fixed forcing-positions test suite at equal search settings
- **THEN** the attacker selects a forcing line in the majority of positions where a forcing line exists
- **AND** the coward selects a quiet or simplifying alternative in the majority of those same positions.

#### Scenario: Swindler and endgame wizard behavior

- **WHEN** the swindler plays from evaluation-lost positions and the endgame wizard plays from positions with a favorable endgame conversion available
- **THEN** the swindler chooses trap-setting moves in the majority of the lost test positions
- **AND** the endgame wizard chooses the line that steers into the favorable endgame in the majority of the conversion test positions.
