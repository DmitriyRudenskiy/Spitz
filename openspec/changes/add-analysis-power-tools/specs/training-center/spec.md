# training-center Delta

## MODIFIED Requirements

### Requirement: Hint Ladder

The trainer SHALL offer three graduated hint levels for every exercise (theme, then candidate moves, then a full explanation) and SHALL never auto-play the solution unless the user explicitly requests it. For every puzzle exercise the trainer SHALL additionally offer an attacked-pieces view listing attacked and hanging pieces for both sides.

#### Scenario: Graduated hints

- **WHEN** the user requests hints on an exercise successively
- **THEN** level 1 names the theme, level 2 lists candidate moves, and level 3 gives the full explanation
- **AND** the solution move is shown or played only after an explicit user request, never automatically

#### Scenario: Attacked pieces view

- **WHEN** the user activates the attacked-pieces command on a puzzle
- **THEN** the panel lists all attacked pieces of both sides with hanging pieces highlighted
- **AND** the list updates immediately when the user plays a candidate move on the board

## ADDED Requirements

### Requirement: Daily puzzle

The trainer SHALL offer a daily puzzle in three difficulty levels (easy, medium, hard) selected from local exercise stores by a date-derived seed, so the puzzle changes once per calendar day without any network access, and SHALL record completions in progress.

#### Scenario: Deterministic for the day

- **WHEN** the daily puzzle is opened three times on the same date at the same difficulty
- **THEN** the same puzzle is offered every time, and the next date yields a different puzzle
- **AND** two installations whose exercise stores share the same version offer the same puzzle for the same date

#### Scenario: Thin store at a difficulty

- **WHEN** the store holds fewer than 20 puzzles at the selected difficulty
- **THEN** the trainer says so and offers the nearest difficulty instead of silently repeating puzzles it has already shown that day

#### Scenario: Completion recorded

- **WHEN** the user solves or skips today's puzzle
- **THEN** the result appears in the progress statistics for the tactics discipline and the puzzle is not offered again that day
