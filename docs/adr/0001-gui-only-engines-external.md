# Spitz is a GUI: engines are external, never bundled

Spitz is an interface for chess engines users supply themselves (Stockfish and any other UCI or
XBoard engine); it ships no engine of its own. A built-in engine would make the project responsible
for strength claims, engine licensing, and a release pipeline it cannot sustain — and every
requirement was therefore written against declared engine options rather than engine internals.
The rejected alternative (bundling Stockfish for out-of-the-box UX) trades a one-time download step
for permanent coupling between app releases and engine releases.
