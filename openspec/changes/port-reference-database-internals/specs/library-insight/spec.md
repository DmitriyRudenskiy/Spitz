## Purpose

`library-insight` turns a game library into answers: who played whom and with what result, which
header name the user meant when they typed a fragment, and what the record says about a player. It
works only over local libraries and user-supplied data files.

## ADDED Requirements

### Requirement: Cross-table report

The app SHALL build a cross-table from any filtered game set of up to 500 players, showing for each
player the games played, wins, draws, losses, score, average opponent rating, and title and
federation when known, and SHALL expose the individual games between any two players of the table on
request. The app SHALL sort the table by name, rating, score, or federation, and SHALL report the
number of players and games excluded when the set exceeds 500 players or 60 rounds.

#### Scenario: Round-robin cross-table

- **WHEN** the user opens a cross-table over the games of a single event containing 12 players who
  each played each other once
- **THEN** the table shows 12 rows and 11 opponent columns, each player's score equals their wins plus
  half their draws, and every cell on the diagonal is marked as not applicable

#### Scenario: Oversized set is truncated with a report

- **WHEN** a cross-table is requested over a filtered set containing 900 distinct players
- **THEN** the report is built for at most 500 players, the excluded player and game counts are shown
  above the table, and the exclusion rule used is stated in the report header

### Requirement: Tournament mode and tie-breaks

The app SHALL support the cross-table modes all-play-all, Swiss, knockout, and automatic, and in
automatic mode SHALL select Swiss when the set has more than 30 players, or fewer than 5 games per
player, or fewer than half the games of a complete round robin, and all-play-all otherwise. The app
SHALL compute tie-breaks as: sum of opponent scores weighted by the game result for all-play-all,
unweighted sum of opponent scores for Swiss, and no tie-break for knockout; and SHALL break a
remaining tie between players of equal score by their head-to-head result, reporting which rule was
applied.

#### Scenario: Automatic mode selection

- **WHEN** automatic mode is used on a set of 8 players with 28 games and on a set of 60 players with
  220 games
- **THEN** the first is reported as all-play-all and the second as Swiss, and the rule that decided
  each is shown

#### Scenario: Head-to-head breaks an equal score

- **WHEN** two players finish a round-robin cross-table with equal score and they met each other once
- **THEN** the higher-ranked of the two is the winner of that meeting and the report names
  head-to-head as the deciding rule

### Requirement: Cross-table output formats

The app SHALL render a cross-table as plain text, as HTML, and as typeset document source, with
identical numbers and ordering in all three, and SHALL print the plain-text form to a single page for
up to 20 players.

#### Scenario: Formats agree

- **WHEN** the same cross-table is exported to plain text, HTML, and typeset source
- **THEN** every player's score, game count, and tie-break value is identical across the three
  outputs and the row order is identical

### Requirement: Header name completion

The app SHALL complete player, event, site, and round names from the names present in the open
libraries, matching a case-insensitive and diacritic-insensitive key that also matches a name written
in the opposite surname-given order, and SHALL return up to 10 ranked candidates within 50 ms on a
1,000,000-game library.

#### Scenario: Surname order does not matter

- **WHEN** a library contains the player "Ivanov, Peter" and the user types "piv" in a player field
- **THEN** "Ivanov, Peter" is among the returned candidates

#### Scenario: Completion stays responsive on a large library

- **WHEN** the user types a three-character prefix in a player field of a 1,000,000-game library
- **THEN** candidates are displayed within 50 ms of the keystroke and at most 10 are shown

### Requirement: Header name correction

The app SHALL offer, for a header value that matches no name in the library, the closest existing
names ranked by edit distance with transposition counted as one operation, considering at most
distance 2 by default, and SHALL apply a user-editable rule file of prefix, infix, and suffix
spelling replacements before matching. The app SHALL show the distance and SHALL never rewrite a
header without a confirmation from the user.

#### Scenario: Transposed letters still suggest

- **WHEN** the library contains "Spassky, Boris" and the user enters "Spsasky, Boris"
- **THEN** "Spassky, Boris" is offered as a correction with its distance shown and the original text
  is kept until the user accepts

#### Scenario: Rule file fixes a systematic spelling

- **WHEN** the rule file maps one spelling of a name to another and a library is re-scanned
- **THEN** names matching the first spelling are matched under the second without changing the stored
  header text

### Requirement: Player profile attributes

The app SHALL read a user-supplied rating list giving, per player name, the current and peak rating,
title, federation, and birth date, and SHALL use it to fill a missing game rating, to show title and
federation columns, and to report a player's age at the date of a game. Missing attributes SHALL be
reported as unknown and never as zero.

#### Scenario: Missing rating filled from the list

- **WHEN** a game with no white rating is displayed and the rating list contains the white player
- **THEN** the rating list value is shown as the source-annotated rating and the stored game is
  unchanged until the user chooses to write it

#### Scenario: Age at event

- **WHEN** a cross-table is requested with the age column enabled for players present in the rating
  list
- **THEN** each such player shows their age in whole years at the game date, and players without a
  birth date show an unknown marker instead of a number
