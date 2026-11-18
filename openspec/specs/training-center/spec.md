# training-center Specification

## Purpose

The training center turns the user's own games and themed material into structured practice across several disciplines. It provides a graduated hint ladder, generated tactics exercises, calculation training with per-mode ratings, an endgame course, timed See&Click bursts, printable Blitz & Train worksheets, a blindfold trainer, and local progress tracking with ratings, charts, and streaks, stored in a documented local format. Delivered in milestone M2 (change `add-analysis-and-training`).

## Requirements

### Requirement: Hint Ladder

The trainer SHALL offer three graduated hint levels for every exercise (theme, then candidate moves, then a full explanation) and SHALL never auto-play the solution unless the user explicitly requests it.

#### Scenario: Graduated hints

- **WHEN** the user requests hints on an exercise successively
- **THEN** level 1 names the theme, level 2 lists candidate moves, and level 3 gives the full explanation
- **AND** the solution move is shown or played only after an explicit user request, never automatically.

#### Scenario: Explicit solution reveal

- **WHEN** the user asks to reveal the solution after working through the hints
- **THEN** the app shows the full solution and marks the exercise as revealed rather than solved
- **AND** the reveal is reflected in the exercise's progress state.

### Requirement: Tactics Generator

The trainer SHALL generate tactics exercises from the user's own games and from themed sets covering attack, defense, and mating nets, producing at least 100 exercises per theme from a 1000-game database.

#### Scenario: Themed generation volume

- **WHEN** the user generates tactics exercises for each theme from a 1000-game database
- **THEN** the attack, defense, and mating-net themes each yield at least 100 exercises
- **AND** exercises taken from the user's own games store a reference to the source game and move number.

### Requirement: Calculation Training

The trainer SHALL provide calculation training in assisted, evaluated, and tactical modes and SHALL maintain an independent rating between 800 and 2800 for each mode, updated after every session.

#### Scenario: Per-mode rating update

- **WHEN** the user completes a session in any calculation mode
- **THEN** that mode's rating is updated immediately and remains within the 800 to 2800 range
- **AND** the three modes keep fully independent ratings.

### Requirement: Endgame Course

The trainer SHALL provide an endgame course of at least 27 modules covering typical endgame types, each with a drill loop and per-module pass criteria.

#### Scenario: Drill loop and pass criteria

- **WHEN** the user opens any course module and starts its drill loop
- **THEN** the module presents repeated drill positions of its endgame type with immediate feedback
- **AND** the module is marked passed only when its defined pass criteria are met and the criteria are displayed to the user.

### Requirement: See and Click

The trainer SHALL provide See&Click exercises as 60-second timed bursts and SHALL record a score history per session.

#### Scenario: Timed burst scoring

- **WHEN** the user starts a See&Click burst
- **THEN** the exercise runs for exactly 60 seconds, counts correct and incorrect selections, and displays the final score
- **AND** the score is appended to the recorded per-session history.

### Requirement: Blitz and Train

The trainer SHALL generate a printable PDF worksheet from a selected blitz game in which every position where the player erred becomes an exercise, with a separate solutions page.

#### Scenario: Worksheet generation

- **WHEN** the user selects a blitz game and generates a Blitz & Train worksheet
- **THEN** the PDF contains one exercise for every position in which the player erred plus a separate solutions page
- **AND** the worksheet is print-ready with a fixed page layout and no interactive elements.

### Requirement: Blindfold Trainer

The trainer SHALL provide a blindfold trainer with progressive piece-hiding levels and per-level performance tracking.

#### Scenario: Progressive hiding with tracking

- **WHEN** the user completes a training set at a blindfold level
- **THEN** pieces are hidden according to that level's rules and performance is recorded for the level
- **AND** each higher level hides at least one more piece class than the level below it.

### Requirement: Progress Tracking

The app SHALL track per-discipline ratings, history charts, and streaks in a local on-device store that the user can export and import as a documented file.

#### Scenario: Local progress records

- **WHEN** the user completes sessions in several disciplines
- **THEN** each discipline's rating, history chart, and streak update from locally stored records
- **AND** exporting and re-importing the progress file on a fresh install restores the ratings, history, and streaks exactly
