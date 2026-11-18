# training-center Delta

## MODIFIED Requirements

### Requirement: Progress Tracking

The app SHALL track per-discipline ratings, history charts, and streaks in a local on-device store that the user can export and import as a documented file. The user SHALL be able to reset a single discipline's rating, history, and streaks without touching any other discipline, and SHALL be able to see the tracked rating of any discipline next to the results of a training session in the same view.

#### Scenario: Local progress records

- **WHEN** the user completes sessions in several disciplines
- **THEN** each discipline's rating, history chart, and streak update from locally stored records
- **AND** exporting and re-importing the progress file on a fresh install restores the ratings, history, and streaks exactly

#### Scenario: Reset one discipline

- **WHEN** the user resets the rating and history of a single discipline and confirms the dialog
- **THEN** that discipline returns to its starting rating with empty history and streak, every other discipline is unchanged, and the dialog offered a progress export before it was confirmed

#### Scenario: Reset by mistake

- **WHEN** the user asks to undo a reset in the same session before any further training
- **THEN** the reset is reverted from the exported snapshot the dialog took, and the restored figures equal the pre-reset values

#### Scenario: Rating shown with results

- **WHEN** a training session ends
- **THEN** the result view shows the discipline's rating before and after the session and the number of solved and unsolved items it is based on

## ADDED Requirements

### Requirement: Exercise collections

The app SHALL import exercise collections from local files — PGN with position and comment fields, EPD with discipline and level markers in the comment field, and the app's documented collection format — SHALL store the imported exercises per discipline and level without duplicating a position already present, and SHALL make them available to the trainers and filters exactly like generated exercises.

#### Scenario: Import and use a collection

- **WHEN** the user imports a collection of 5,000 exercises tagged by discipline and level
- **THEN** the import reports the accepted, duplicated, and rejected counts with the reason for each rejection, and a trainer filtered to that discipline, level, and collection serves only those exercises

#### Scenario: Collection exercises behave like generated ones

- **WHEN** an imported exercise is solved with hints, timed, or solved wrongly
- **THEN** the hint ladder, timing, rating, and history rules of its discipline apply unchanged, and the exercise records the collection it came from

#### Scenario: Missing media is reported, not fetched

- **WHEN** an imported collection refers to audio or image files that are not present locally
- **THEN** the exercises still run with the media item marked unavailable and its expected path shown, and the app performs no download

#### Scenario: Large import stays bounded

- **WHEN** a collection of 50,000 positions is imported
- **THEN** the import finishes within 60 seconds on the reference hardware, keeps the interface responsive at or below 50 ms p95 input latency, and can be cancelled with the partial import rolled back
