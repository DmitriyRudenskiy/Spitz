# opening-book Delta

## MODIFIED Requirements

### Requirement: Weighted Selection

The selection layer SHALL select book moves by one of three selectable modes: weighted by play
frequency, weighted by average opponent Elo, and weighted by performance (score percentage).

#### Scenario: Mode changes selection

- **WHEN** the selection layer must choose a book move in a position with several booked moves and
  the user switches between the frequency, Elo, and performance modes
- **THEN** the chosen move follows the weights of the active mode and the per-move weights are shown
  before the move is played
- **AND** repeated automated trials per mode produce move distributions consistent with that mode's
  weighting

## ADDED Requirements

### Requirement: Play-value classes and breadth modes

Every booked move SHALL carry a play-value class from `a` to `g`, editable per move and shown in
the book panel, and book selection SHALL honour a breadth mode: tournament mode plays only classes
`a`-`c` when any is available, standard mode `a`-`d`, wide mode `a`-`e`; classes `f` and `g` are
never played and are reported as excluded when they are the only entries.

#### Scenario: Tournament breadth skips risky lines

- **WHEN** a book position holds moves of classes `d` and `e` only and breadth mode is tournament
- **THEN** no book move is played, the panel states that entries exist but fall outside the
  tournament breadth, and switching to wide mode makes the `d` and `e` moves selectable for play

#### Scenario: Never-play classes

- **WHEN** every booked move of a position is class `f` or `g`
- **THEN** book selection reports the position as exhausted rather than playing one of those moves

### Requirement: Book play gate

Book selection SHALL apply a configurable gate — minimum games played, minimum games won, minimum
score percentage split by the colour playing the move, and a maximum move number beyond which no
book move is played — and the book panel SHALL mark every gate-excluded move with the specific
threshold it failed.

#### Scenario: Weak score against colour gate

- **WHEN** the gate requires at least 50 games, 10 wins, and 45 percent for Black, and a booked
  move shows 300 games, 40 wins, and 41 percent for Black
- **THEN** that move is excluded, marked as failing the Black score threshold with both numbers
  shown, and a move at 46 percent passes

#### Scenario: Last book move number

- **WHEN** the maximum book move number is set to 12 and the game reaches move 13 in a booked
  position
- **THEN** the engine leaves the book, the panel states the move-number cap as the reason, and no
  later position of the game is served from the book

### Requirement: Recent-results correction

The app SHALL maintain, per booked move, a rolling window of the user's own most recent results
with that move (default 20 games, configurable 5-100) and SHALL display and weight an adjusted
probability that demotes moves whose recent results diverge from their lifetime statistics, with
the lifetime and adjusted values shown side by side.

#### Scenario: Refuted line demoted

- **WHEN** a book move with a 60 percent lifetime score has lost 9 of the user's last 10 games with
  it
- **THEN** its adjusted probability is below its lifetime value, the panel shows both, and the move
  is still selectable with its class unchanged

### Requirement: Recency bias in selection

Book selection SHALL offer a classic-to-modern bias that shifts selection weight toward entries by
their last-used date, at three strengths (off, moderate, strong), and a strong modern bias SHALL
produce a measurably newer distribution of played entries than the off setting over a fixed
100-position replay of one library.

#### Scenario: Modern bias observable

- **WHEN** the same 100 book positions are selected 200 times with the bias off and then with the
  strong modern setting
- **THEN** the mean last-used date of the played entries is later under the modern setting and the
  panel shows the bias in force for the session

### Requirement: Book import reporting and log

Every book import run (games or another book) SHALL report counts of records read, games played
through to the configured length, positions added, moves merged into existing entries, statistics
updates, and rejected records with a reason per rejection class, and SHALL append a line to a
per-book plain-text import log naming timestamp, source path, options used, and those counts.

#### Scenario: Broken PGN import is explained

- **WHEN** a 10,000-game PGN file with 43 games containing an illegal move is imported
- **THEN** the report separates the 43 rejected games by reason, the log line contains the same
  counts, and the book's statistics reflect only the accepted games
