# play-vs-computer Delta

## ADDED Requirements

### Requirement: Named level catalog

The app SHALL offer playing levels as a catalog of named level types — fixed search depth, fixed
node count, exact time per move, average time over a game budget, opponent Elo cap, casual random
band, blitz burst, and period-based match control — each defined as a constraint plus the engine
mechanism that realises it, and each falling back with a visible explanation when the selected
engine does not declare the mechanism.

#### Scenario: Fixed depth level holds

- **WHEN** a game runs at the fixed-depth level of 8 plies against an engine with a depth option
- **THEN** every move search reports depth 8 plus or minus 1 ply and the level name shown during
  play states the depth actually requested

#### Scenario: Elo cap only when declared

- **WHEN** the user picks the Elo-cap level for an engine without a strength option
- **THEN** the app says the cap is unavailable for that engine and offers the calibrated
  node-limit or depth substitute it measured for that engine, naming the measured band

### Requirement: Adjudication and draw claims

The app SHALL support engine-threshold resign adjudication (resign when the evaluation stays beyond
a configurable centipawn or mate-distance margin, per side or both sides or off) and material
adjudication (refuse to resign or declare lost when the rules core cannot construct any legal
mate-able ending from the remaining material), SHALL verify player-initiated draw claims for
threefold repetition, the fifty-move rule, and insufficient material through the rules core before
scoring them, and SHALL record every adjudicated or claimed ending with its standard PGN termination
cause.

#### Scenario: Fortress is not resigned

- **WHEN** resign adjudication is on and the evaluation shows a 6-pawn deficit while the rules core
  reports no legal sequence to mate for the side ahead
- **THEN** no resignation is issued and the reason is shown with the game continuing

#### Scenario: Verified threefold claim

- **WHEN** a player claims a draw by threefold repetition
- **THEN** the claim is checked against the position history, accepted games end immediately as a
  draw with the repetition termination recorded, and rejected claims are refused with the
  occurrence count without ending the game

### Requirement: Strict game mode

The app SHALL provide a strict game mode that, for the whole game, disables takebacks, hints,
evaluation display, post-move automatic analysis, manual clock editing, and position editing, marks
the game rated in the user's record, records every use attempt in the game file as an assist audit
line, and downgrades the game to unrated if any disabled facility was used.

#### Scenario: Clean strict game is rated

- **WHEN** a strict game finishes with no assist, takeback, or clock edit recorded
- **THEN** the result counts in the user's rated record and the game file carries the strict marker
  and an empty assist audit

#### Scenario: One hint voids the rating

- **WHEN** a hint is requested during a strict game
- **THEN** the hint is refused or the game is marked unrated with the audit line naming the
  facility and the move number, and the choice is stated at the moment of the attempt
