# chess-rules-core Delta

## MODIFIED Requirements

### Requirement: Game clocks

The library SHALL manage game clocks for the time controls none, sudden death, increment, delay,
per-move time without a game clock, moves-per-period with one to three periods where the final
period may be all remaining time, and SHALL emit a flag-fall event the instant a player's remaining
time reaches zero, SHALL record for every completed move the time spent and the remaining time, and
SHALL accept a corrected clock value for either player while the calling game mode allows clock
edits.

#### Scenario: Increment and delay

- **WHEN** moves are completed under an increment control and under a delay control
- **THEN** the increment is credited only after a completed move, and the delay consumes stored time
  only when a move takes longer than the delay

#### Scenario: Flag fall event

- **WHEN** a player's remaining time reaches zero
- **THEN** a flag-fall event is emitted immediately, identifying the player whose flag fell and the
  game time state

#### Scenario: Period control transitions

- **WHEN** a game runs under 40 moves in 90 minutes, then 40 moves in 60 minutes, then all
  remaining moves in 15 minutes with a 30-second increment from the first move
- **THEN** each period's moves consume that period's time, unspent time carries into the final
  all-remaining period exactly once, the increment is credited on every completed move, and flag
  fall in any period ends the game

#### Scenario: Per-move time records

- **WHEN** twenty moves are played under any running control
- **THEN** each completed move exposes its spent time and remaining time accurate to 10
  milliseconds, and per-move times sum to the elapsed control time within one second

#### Scenario: Manual correction is permitted or refused by mode

- **WHEN** a clock correction arrives while the game mode disallows clock edits
- **THEN** the correction is refused with a stated reason and both clocks keep running unchanged
