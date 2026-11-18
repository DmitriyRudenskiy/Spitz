# voice-interface Specification

## Purpose

The voice-interface capability lets users enter moves by voice and receive spoken feedback in English, German, and Russian. It covers the chess-specific vocabulary, resolves ambiguous recognitions through confidence thresholds and confirmation flows, and offers push-to-talk and continuous listening modes with audible cues. Spoken commentary for moves and evaluations is configurable, local speech models keep every voice function working offline, and recognition completes within 1.5 seconds round trip on reference hardware. Delivered in milestone M3 (change `add-online-cloud-voice`).

## Requirements

### Requirement: Multilingual chess move vocabulary

The app SHALL recognize spoken move commands built from the chess vocabulary of piece names, squares, files, ranks, castling, and capture phrasing in English, German, and Russian, and SHALL apply a recognized command as a move on the active board.

#### Scenario: Recognized move in each supported language

- **WHEN** the user speaks a valid move expression, such as a piece with destination square, a castling phrase, or a capture phrase, in English, German, or Russian
- **THEN** the app applies the corresponding legal move on the active board

#### Scenario: Utterance with no legal move

- **WHEN** the recognized utterance does not map to a legal move in the current position
- **THEN** the app reports that the move was not understood or not legal and leaves the position unchanged

### Requirement: Confidence threshold and ambiguity confirmation

The app SHALL apply a configurable confidence threshold to recognition results, SHALL require confirmation for ambiguous or below-threshold parses before applying a move, and SHALL offer a confirm mode that displays the recognized move for approval before it is applied.

#### Scenario: Ambiguous parse requires confirmation

- **WHEN** the recognizer returns a below-threshold result or more than one plausible move parse
- **THEN** the app asks the user to confirm or repeat and applies no move until one parse is confirmed

#### Scenario: Confirm mode shows the move first

- **WHEN** confirm mode is enabled and the user speaks a recognized move
- **THEN** the app displays the recognized move and applies it only after the user approves it

#### Scenario: Direct application of confident results

- **WHEN** confirm mode is off and the result exceeds the confidence threshold with a single parse
- **THEN** the app applies the move immediately

### Requirement: Push-to-talk and continuous listening

The app SHALL support push-to-talk capture activated by a user action and a continuous listening mode, and SHALL play an audible cue when continuous listening starts and stops.

#### Scenario: Push-to-talk window

- **WHEN** the user presses and releases the push-to-talk control
- **THEN** only the speech captured between press and release is processed as a move command

#### Scenario: Continuous listening cues

- **WHEN** the user enables or stops continuous listening
- **THEN** the app plays an audible cue at the start and end of capture so the user knows when speech is being captured

### Requirement: Spoken commentary with configurable voice

The app SHALL announce moves and available evaluations through speech synthesis and SHALL let the user configure the speech rate, pitch, and voice used for announcements.

#### Scenario: Move and evaluation announcements

- **WHEN** commentary is enabled and a move is played in the followed game
- **THEN** the app speaks the move and, when an evaluation is available, announces the current evaluation

#### Scenario: Voice settings take effect

- **WHEN** the user changes the speech rate, pitch, or voice for commentary
- **THEN** subsequent announcements use the new settings without requiring a restart

### Requirement: Downloadable offline speech models

The app SHALL offer local speech recognition and speech synthesis models as downloads and SHALL provide full voice function, including move entry and commentary, with no network connection once the models are installed.

#### Scenario: Full function offline

- **WHEN** the local recognition and synthesis models are installed and the machine has no network connectivity
- **THEN** move entry, confirmation flows, and spoken commentary all continue to work offline

#### Scenario: Model download in settings

- **WHEN** the user downloads a local speech model in settings
- **THEN** the app lists the model as available and uses it for offline voice function

### Requirement: Recognition latency budget

The app SHALL produce a recognition result for a spoken move command within 1.5 seconds round trip from the end of speech on reference hardware.

#### Scenario: Latency on reference hardware

- **WHEN** the user speaks a move command on reference hardware with local models installed
- **THEN** the recognized result is available within 1.5 seconds after the user stops speaking
