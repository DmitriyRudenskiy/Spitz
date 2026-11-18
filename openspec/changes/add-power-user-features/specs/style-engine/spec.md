# style-engine Delta

## MODIFIED Requirements

### Requirement: Archetype Opponents

The selection layer SHALL implement at least 6 named archetypes, each defined by declared role features measured from the engine's reported lines — forcing-ness from the checks and captures in the line, volatility from the spread between the engine's alternatives, exchange appetite from line length and resulting material, endgame preference from remaining material, and the width of the candidate set it is willing to accept — and each archetype SHALL be calibrated against a fixed test suite so that the share of archetype-consistent moves it selects is at least 70 % on a recorded 200-position suite.

#### Scenario: An aggressive archetype plays forcing lines

- **WHEN** the aggressive archetype plays a set of 20 standard opening positions with a fixed engine, seed, and node cap
- **THEN** at least 70 % of its moves are forcing moves — checks, captures, or direct threats — when a forcing move of comparable quality exists in the recorded comparison

#### Scenario: A positional archetype prefers quiet maneuvers

- **WHEN** the positional archetype plays the same 20 positions with the same engine, seed, and node cap
- **THEN** at least 70 % of its moves avoid unnecessary exchanges and preserve the pawn structure when a quiet alternative within 0.3 pawns of the engine's best move exists

#### Scenario: Named archetypes are selectable

- **WHEN** the user opens the opponent style picker
- **THEN** at least 6 named archetypes are listed with a one-line description of what each prioritizes, and each choice changes the observed move selection

#### Scenario: Style choice is auditable

- **WHEN** a game finished against an archetype is exported to PGN
- **THEN** the header records the archetype name and its feature values so the same conditions can be reproduced

#### Scenario: Attacker and coward behavior

- **WHEN** the attacker and the coward each play a fixed forcing-positions test suite at equal search settings
- **THEN** the attacker selects a forcing line in the majority of positions where a forcing line exists
- **AND** the coward selects a quiet or simplifying alternative in the majority of those same positions.

#### Scenario: Swindler and endgame wizard behavior

- **WHEN** the swindler plays from evaluation-lost positions and the endgame wizard plays from positions with a favorable endgame conversion available
- **THEN** the swindler chooses trap-setting moves in the majority of the lost test positions
- **AND** the endgame wizard chooses the line that steers into the favorable endgame in the majority of the conversion test positions.

## ADDED Requirements

### Requirement: Editable and importable style profiles

The app SHALL let the user copy an archetype, edit its role feature values and its acceptance
thresholds, name the result, use it as an opponent, and export or import such profiles as local
files.

#### Scenario: Custom profile plays differently

- **WHEN** the user raises an archetype's exchange-appetite value and plays the fixed 20-position
  suite again with the same engine and seed
- **THEN** the measured exchange rate moves in the direction of the edit and the report names the
  profile version used

#### Scenario: Profile exchange

- **WHEN** a profile file is exported and then imported on a second installation
- **THEN** the imported profile reports identical feature values and selects the same moves for the
  suite at the same engine, node cap, and seed
