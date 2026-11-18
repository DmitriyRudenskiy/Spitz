# engine-learning Delta

## ADDED Requirements

### Requirement: Persistent engine learning

The engine SHALL learn persistently from previously played positions — recording local evaluation adjustments for positions where its play was outplayed — and SHALL apply the learned adjustments in later games when similar positions occur.

#### Scenario: Lost line avoided

- **WHEN** the engine loses repeatedly out of a specific opening line in earlier sessions
- **THEN** in a later session from the same line the engine's chosen continuation and evaluation reflect the recorded learning

### Requirement: Learning control and visibility

Engine learning SHALL be toggleable per engine, the learning data SHALL be viewable (position count and storage size), and resettable to empty.

#### Scenario: Reset learning

- **WHEN** the user resets an engine's learning data
- **THEN** the store is emptied, the view reports zero positions, and subsequent games start without learned adjustments

### Requirement: Book learning in matches

Engine match and tournament runners SHALL optionally update opening-book weights from game results during the match or tournament (book learning), per side.

#### Scenario: Book weight updated after loss

- **WHEN** a side loses a match game out of a book line with book learning enabled
- **THEN** that line's weight is reduced for subsequent games of the same match
