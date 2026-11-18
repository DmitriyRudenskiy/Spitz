# Local-only: the program opens no sockets

Spitz runs entirely on local files — no accounts, no cloud analysis, no server-delivered books or
endgame lookups, no telemetry. Everything a networked chess suite offers must be reachable through
user-supplied local data (tablebases, books, game files) or it is out of scope, recorded in
`openspec/backlog/descoped.md`. This was chosen so the single performance/privacy requirement can
be absolute ("no sockets") and testable, instead of a graded policy about what may phone home; the
rejected alternative (optional cloud features behind opt-in) reintroduces accounts, availability,
and a privacy surface a hobby project must then defend forever.
