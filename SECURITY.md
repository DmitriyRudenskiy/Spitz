# SECURITY.md

Spitz is a local-only desktop program: it binds no ports, opens no outbound connections, and stores
everything in files you can inspect. That is its main security property, and it is a requirement
with a test, not a slogan.

The boundary that matters is the engine. By design, Spitz runs engine binaries you choose, as your
user, with your permissions. A malicious engine is indistinguishable from any other program you
voluntarily executed - Spitz adds resource caps and process controls for liveliness, not a sandbox,
and does not claim otherwise in v1. Only add engine binaries you trust, exactly as you would with
any executable.

PGN, EPD, book, and tablebase files are parsed defensively (no evaluation, no shell-outs, bounded
memory per parse), but they are data formats from outside the project; treat game files received
from strangers the way you would treat any untrusted input: keep them out of your library until
you know them.

Bugs worth reporting: any network connection the app opens, any file it touches outside your data
directories and the files you pointed it at, any crash from a malformed file, any engine-process
escalation claim we got wrong. Open an issue normally; use a private channel only if the report
includes unpublished exploits.
