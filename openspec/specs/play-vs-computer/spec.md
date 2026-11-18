# play-vs-computer Specification

## Purpose

This capability delivers the complete experience of playing against engines and personas inside the app. A single dialog configures opponent, strength level, time control, color, and handicap; adaptive strength keeps the opponent near the user's real level; standard time controls end games on flag fall; assists support learning; and a post-game summary connects the finished game to the full report and to training. Delivered in milestone M2 (change `add-analysis-and-training`).

## Requirements

### Requirement: New-Game Dialog

The app SHALL provide a single new-game dialog in which opponent (engine or personality), strength level, time control, color, and handicap are configured together before the game starts.

#### Scenario: One-dialog setup

- **WHEN** the user sets opponent, level, time control, color, and handicap in the dialog and confirms
- **THEN** a game starts with exactly the configured opponent, level, time control, color assignment, and handicap
- **AND** a combination the selected opponent cannot fulfill is reported in the dialog and cannot start the game silently.

### Requirement: Adaptive Strength

The app SHALL track the user's results and auto-adjust the opponent level within plus or minus 150 Elo bands around the estimated user strength, with total level drift over any 20-game window limited to at most one level step.

#### Scenario: Bounded adjustment

- **WHEN** the user accumulates wins or losses across consecutive games
- **THEN** the app updates the strength estimate and moves the engine level only in steps inside the plus or minus 150 Elo band around that estimate
- **AND** across any 20 consecutive games the cumulative level drift never exceeds one level step.

### Requirement: Time Controls

The app SHALL offer time-control presets from 1+0 through 90+30 plus a custom option, and the rules core SHALL end the game on flag fall.

#### Scenario: Presets and custom control

- **WHEN** the user opens the time-control options in the new-game dialog
- **THEN** presets from 1+0 to 90+30 are selectable and a custom option accepts an arbitrary base time and increment
- **AND** the selected control is shown on the game clock for both players.

#### Scenario: Flag fall ends the game

- **WHEN** a player's remaining time reaches zero
- **THEN** the game ends immediately with the flag-fall result defined by the rules core
- **AND** the clock display marks the flagged side and the game record stores the termination cause.

### Requirement: Assists

The app SHALL provide assists consisting of an evaluation bar, hints, a configurable takeback count, and a blindfold mode that hides pieces progressively.

#### Scenario: Takebacks and blindfold

- **WHEN** the user configures a takeback count and enables blindfold mode with a chosen hiding level before a game
- **THEN** exactly the configured number of takebacks is available and each use consumes one
- **AND** blindfold mode hides pieces progressively by level, with hidden pieces remaining selectable through their squares.

#### Scenario: Evaluation bar and hints

- **WHEN** the evaluation bar is enabled and the user requests a hint during the game
- **THEN** the bar updates with the current evaluation and the hint reveals a suggested move without playing it
- **AND** assist usage during the game is recorded for the post-game summary.

### Requirement: Post-Game Summary

The app SHALL show a post-game summary containing the result, per-player accuracy, and the blunder count, with links into the full report and into suggested training exercises.

#### Scenario: Summary with links

- **WHEN** a game against the computer ends
- **THEN** the summary shows the game result, both players' accuracy percentages, and the blunder count
- **AND** it offers links that open the full-game report and the training exercises suggested from the game's mistakes.
