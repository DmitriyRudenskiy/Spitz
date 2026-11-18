# humanized-opponent Delta

## ADDED Requirements

### Requirement: Strength slider with opening knowledge

The app SHALL offer a continuous strength slider from beginner to grandmaster plus a separate opening-knowledge setting (limited to full opening-book use) that is independent of search strength, and a full-force option limited only by hardware.

#### Scenario: Weak search, full book

- **WHEN** the user sets a mid-range strength with full opening knowledge
- **THEN** the opponent plays established book moves in the opening but shows clearly weaker play in the middlegame

#### Scenario: Full force

- **WHEN** the strength slider is set to maximum with the full-force option
- **THEN** the opponent plays at the engine's unrestricted strength with complete opening knowledge

### Requirement: Rated games with measured rating

The app SHALL support rated and casual games; rated games update a measured user rating with a visible history, and casual games SHALL NOT change it.

#### Scenario: Rating update after rated win

- **WHEN** the user wins a rated game against an opponent set above their current rating
- **THEN** the measured rating increases and the change appears in the rating history chart

#### Scenario: Casual game isolation

- **WHEN** the user finishes a casual game
- **THEN** the measured rating is unchanged

### Requirement: Opponent selection by strength

The app SHALL allow choosing a fixed opponent level or an "opponent with my strength" option that uses the measured user rating to pick the level.

#### Scenario: Opponent with my strength

- **WHEN** the user selects "opponent with my strength" after rated games exist
- **THEN** the chosen opponent level corresponds to the measured rating within one level step

### Requirement: Auto-adjust strength

The app SHALL automatically adjust opponent strength after each rated game based on the result — up after user wins, down after user losses — bounded per adjustment, and the behavior SHALL be disableable.

#### Scenario: Loss lowers strength

- **WHEN** the user loses a rated game with auto-adjust enabled
- **THEN** the next game's opponent strength is lowered by the bounded step and the change is announced

### Requirement: Handicap games

The app SHALL offer handicap games with material odds (piece removal), time odds, and combined handicap options, applied before the first move.

#### Scenario: Knight odds game

- **WHEN** the user starts a game with knight odds against the computer
- **THEN** the knight is removed from the computer's side, the clock and material display reflect the handicap, and the game result is recorded with the handicap tag

### Requirement: Human-like error model

At reduced strength the opponent SHALL produce a human-shaped error distribution — occasional weaker and stronger moves with within-game variance — rather than alternating perfect moves with random blunders; a fixed calibration suite SHALL verify the distribution.

#### Scenario: Calibration suite

- **WHEN** the fixed calibration matches run for a mid-range level
- **THEN** move-quality variance and error streaks fall within the recorded reference bounds for that level

### Requirement: Natural game behavior

In training games the opponent SHALL offer a draw when materially or positionally worse or when the position is equal, SHALL resign clearly lost positions through an accept/decline dialog, and SHALL offer to continue after losing to a user without mate.

#### Scenario: Draw offer when worse

- **WHEN** the opponent's evaluation drops beyond the offer threshold with material deficit in a training game
- **THEN** the opponent offers a draw via dialog at most once per game phase, and refusing continues the game normally

#### Scenario: Resignation

- **WHEN** the opponent's evaluation passes the resignation threshold for a sustained interval
- **THEN** the opponent resigns with a dialog the user can dismiss

### Requirement: In-game coach aids

With coach mode enabled the app SHALL warn when the user's move loses significant evaluation, suggest a better move with an optional takeback, show requested hints as an on-board arrow, and — when a piece is clicked — mark safe destination squares and discouraged squares with distinct colors.

#### Scenario: Blunder warning with takeback

- **WHEN** the user plays a move that loses more than the warning threshold with coach mode enabled
- **THEN** the coach message names the mistake, marks the better move, and offers a takeback that restores the position when accepted

#### Scenario: Safe-square dots

- **WHEN** the user clicks a piece during a training game with coach mode enabled
- **THEN** legal destinations show green dots for safe squares and red dots for discouraged squares, based on a short engine check
