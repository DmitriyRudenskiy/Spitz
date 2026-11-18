# game-commentary Delta

## Purpose

The commentary capability makes a game document rich enough to teach from and to publish: graphic
annotation elements drawn on the board, a standard annotation symbol set, spoken commentary recorded
locally and replayed in step with the notation, and export formats that carry all of it out of the
app. Everything is stored in or next to the game file and needs no network.

## ADDED Requirements

### Requirement: Graphic comment elements

The app SHALL let the user attach graphic elements to a position or to a move — arrows with a
colour and a head style, labelled squares, closed groups of squares, and short text labels — SHALL
render them on the board of the matching view, and SHALL store them with the game so they survive
save, reload, and PGN round trip through the documented sidecar extension.

#### Scenario: Elements survive a round trip

- **WHEN** a game carrying two arrows, one labelled square, one zone, and one text label is saved,
  closed, reopened, exported to PGN, and that PGN is re-imported
- **THEN** every element is present at the same move or position with its colour, label text, and
  square set unchanged

#### Scenario: Elements follow their anchor

- **WHEN** the user steps through a game that has annotation elements attached to different moves
- **THEN** only the elements anchored to the displayed position or move are drawn
- **AND** editing the move order never leaves an element drawn on an unrelated move

#### Scenario: Element classes toggle independently

- **WHEN** the user turns off arrows while labels and zones stay on, in a view or for an export
- **THEN** arrows disappear from that view or export and the other classes keep rendering

### Requirement: Standard annotation symbols

The notation editor SHALL offer the published annotation symbol set (the book-standard commentary
symbols for good, interesting, inaccuracy, mistake, blunder, brilliant, dubious, equality, unclear,
advantage for either side) as a picker, SHALL map each symbol to its PGN NAG code on export, SHALL
render the symbol in the move list with its meaning as a tooltip, and SHALL accept custom
user-defined symbols stored as text.

#### Scenario: Symbol maps to NAG

- **WHEN** the user applies the mistake symbol to a move and the game is exported to PGN
- **THEN** the move carries the NAG code assigned to that symbol and no other move is changed
- **AND** re-importing that PGN shows the same symbol on the same move

#### Scenario: Custom symbol

- **WHEN** the user defines a custom symbol with the text "my habitual weakness"
- **THEN** it is selectable for any move, renders in the move list, and exports as a comment rather
  than being silently dropped

### Requirement: Spoken commentary

The app SHALL record spoken commentary from the selected input device against a move, store it as a
local audio file referenced by the game document together with a content hash, play it back with
play, pause, seek, and stop controls, and never upload or transmit the recording.

#### Scenario: Record and replay

- **WHEN** the user records audio on a move, stops, and presses play
- **THEN** playback starts within 250 ms and the board shows the move the recording belongs to
- **AND** re-recording replaces the previous take after confirmation and reports the file it wrote

#### Scenario: Missing media is reported locally

- **WHEN** a game referencing an audio file is opened and that file is absent or its hash does not
  match
- **THEN** the app names the expected path, shows the commentary marker as unavailable, and keeps
  the rest of the game readable
- **AND** it does not attempt to fetch the file from anywhere

#### Scenario: Recording device failure

- **WHEN** no capture device is available or the device is taken by another application
- **THEN** recording is disabled with the reason stated and the game remains editable

### Requirement: Synchronized commentary playback

The app SHALL support a commentary timeline in which the board advances automatically as playback
reaches the time offset of each anchored move, SHALL let the user edit those offsets by dragging
markers or stepping during playback, and SHALL keep the offset table with the game.

#### Scenario: Board follows the narration

- **WHEN** a commentary with anchored moves is played from the start
- **THEN** the board moves to each anchored move within 250 ms of its offset and a user pause stops
  both audio and board advance

#### Scenario: Offset edit is reversible

- **WHEN** the user drags an anchor marker to a new time and then undoes the edit
- **THEN** the offset returns to its previous value and playback stays in sync after either action

### Requirement: Publication exports

The app SHALL export a game as an animated image and as a static diagram sheet, both rendered from
the same board view as the app and both including the visible annotation elements and symbols, and
SHALL export the game with spoken commentary as a media package described by a documented manifest.

#### Scenario: Animated game export

- **WHEN** the user exports a 60-move game as an animated image at 400 px with 8 frames per second
  and looping enabled
- **THEN** the file animates the moves in order with a readable clock/notation caption, plays at
  least 8 frames per second, and stays under 8 MB on the reference machine

#### Scenario: Diagram sheet

- **WHEN** the user exports the key positions of a game as a diagram sheet
- **THEN** the sheet places the chosen diagrams with their move numbers and ECO label at the
  requested page density and prints at the requested resolution (default 500 dpi) without clipped
  coordinates

#### Scenario: Media package manifest

- **WHEN** a game with spoken commentary and graphics is exported as a media package
- **THEN** the package contains the game file, the referenced media files, and a manifest listing
  paths, hashes, durations, and anchor offsets, and a second installation of the app can open that
  package and play the commentary
