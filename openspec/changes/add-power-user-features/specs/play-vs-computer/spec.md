# play-vs-computer Delta

## MODIFIED Requirements

### Requirement: Assists

The app SHALL provide, during play against a computer opponent, toggleable training aids: an evaluation bar and numeric score next to the board, a threat view showing the opponent's best reply and the squares it targets, dynamic move hints for the square under the cursor, a display of the move the app expected before the user's move was committed, and the existing calculation view and move counter. Each aid is off by default in rated-style play, every aid use is recorded, and no aid reveals the opponent's planned move in advance.

#### Scenario: Threat view

- **WHEN** the threat view is enabled and the opponent has a winning threat in the current position
- **THEN** the threatened squares are highlighted and the threat is described in text within 300 ms of the position change

#### Scenario: Hints without revealing the plan

- **WHEN** any assist is enabled and the opponent is thinking
- **THEN** the opponent's chosen move is not revealed before it is played, and the assist only reflects the position as it stands

#### Scenario: Expected move shown after the fact

- **WHEN** the user plays a move that differs from the move the app expected and the expected-move assist is on
- **THEN** the expected move is shown on the board and in the move list after the user's move is committed, and the pair is recorded for the summary

#### Scenario: Aids are recorded

- **WHEN** a user plays a game with any assist switched on
- **THEN** the game records which aids were used and how often, the aid use is shown in the post-game summary, and the recording names each aid

#### Scenario: Takebacks and blindfold

- **WHEN** the user configures a takeback count and enables blindfold mode with a chosen hiding level before a game
- **THEN** exactly the configured number of takebacks is available and each use consumes one
- **AND** blindfold mode hides pieces progressively by level, with hidden pieces remaining selectable through their squares.

#### Scenario: Evaluation bar and hints

- **WHEN** the evaluation bar is enabled and the user requests a hint during the game
- **THEN** the bar updates with the current evaluation and the hint reveals a suggested move without playing it
- **AND** assist usage during the game is recorded for the post-game summary.


### Requirement: Post-Game Summary

The app SHALL show a post-game summary after mate, resignation, or time out, showing result, accuracy per side, largest mistake, tactical stats (matrix and sparkline), opening name and moves played, and assist usage, and SHALL additionally offer a move-by-move commentary listing every move of the game with its classification, the better alternative when one existed, and the reason the alternative is better stated from the recorded analysis data.

#### Scenario: Summary content

- **WHEN** a game ends
- **THEN** the summary shows the result, accuracy per side, the decisive moment, the error matrix and sparkline, the opening with the number of book moves reached, and assist usage

#### Scenario: Move-by-move commentary

- **WHEN** the user opens the move-by-move commentary of a finished game
- **THEN** every move is listed with its classification, its evaluation before and after, the best
  alternative when the played move was not it, and a one-line reason naming the structural or
  tactical fact behind the judgement

#### Scenario: Commentary is exportable

- **WHEN** the user exports the move-by-move commentary
- **THEN** the exported game carries the same statements as comments and symbols on the moves they
  describe, and a plain PGN export without them is still possible

#### Scenario: Opening detection

- **WHEN** the game followed a known ECO line for 9 moves
- **THEN** the summary names the opening and reports "9 book moves"

#### Scenario: Summary with links

- **WHEN** a game against the computer ends
- **THEN** the summary shows the game result, both players' accuracy percentages, and the blunder count
- **AND** it offers links that open the full-game report and the training exercises suggested from the game's mistakes.

## ADDED Requirements

### Requirement: Position brief

On request, without waiting for an engine move, the app SHALL present a static description of the
current position covering its phase, the material balance, the pawn structure and its weaknesses,
which pieces of each side are active or passive, king safety, and whose advantages they are, with
each statement derived from the recorded position statistics.

#### Scenario: Brief on demand

- **WHEN** the user asks for a position brief during a game or in analysis
- **THEN** the brief appears within 250 ms of the request, without a new engine search, and states
  only facts present in the position statistics for that position

#### Scenario: Brief in trainer mode

- **WHEN** the trainer is enabled and the user plays a move the trainer expected
- **THEN** the brief for the resulting position is shown together with the move confirmation, and the
  next expected move is stated after it

#### Scenario: No statistics available

- **WHEN** the position statistics cannot be computed for the current position
- **THEN** the brief states which part is unavailable instead of presenting an unsupported judgement

### Requirement: Position temperature

The app SHALL compute a position temperature describing how forced the play is — from the density of
checks, captures and threats in the position and the volatility of the evaluation around it — SHALL
show it as a value with a quiet-to-sharp label and a sparkline across the game, and SHALL calibrate
it so a fixed quiet corpus scores at most half the value of a fixed sharp corpus.

#### Scenario: Quiet and sharp positions are told apart

- **WHEN** the temperature is computed for the calibration corpus of 100 quiet and 100 sharp
  positions
- **THEN** the median quiet score is at most half the median sharp score and the labels place at
  least 80 % of each corpus in the expected band

#### Scenario: Temperature shown in the coach panel

- **WHEN** a game with recorded evaluations is reviewed with the coach panel open
- **THEN** the temperature curve is drawn over the moves, points the pointer at are readable, and
  clicking a point jumps to that move

### Requirement: Sparring from a position

The app SHALL start a training session from a chosen position — the current position, a position
from the notation, or a position reached by playing a line from the start — playing a configurable
number of games against a configured engine level, and after each game SHALL offer the same position
again, a related position, or the next position of the current list.

#### Scenario: Repeat the same position

- **WHEN** the user starts sparring from a position with 5 games at one level
- **THEN** 5 games are played from that position with the colours alternating, each game is saved,
  and the session summary shows the result of each game and the moves where the user deviated from
  the line being trained

#### Scenario: Reach the position by playing it

- **WHEN** the user chooses to reach the training position by playing the line from the start
- **THEN** the app plays the opponent's side of the line until the position is reached, and a
  deviation by the user is answered with the line's intended move shown before play continues

#### Scenario: Position lists drive sessions

- **WHEN** a position list of 10 positions loaded from EPD, PGN, or a saved library selection is
  used as the session source
- **THEN** the session walks the list, records progress per position, and resumes at the first
  unfinished position when the app is reopened

### Requirement: Training from a losing position

The app SHALL offer, after a lost game, to start a training session from a position at the moment the
game was still drawable, configured with the number of attempts and the difficulty, and SHALL report
per attempt whether the user held the position against the engine.

#### Scenario: Retrain a lost game

- **WHEN** the user accepts the offer after a loss and asks for 3 attempts
- **THEN** 3 games start from the chosen position with the user defending the losing side, and each
  attempt is recorded with its result and the move where it failed

#### Scenario: Position choice is shown

- **WHEN** the app proposes the position to train
- **THEN** it shows the position, the move number, and why that point was chosen as the last
  defensible moment, and the user can pick another move before the session starts
