# Launch checklist

Things that are cheap before the first public release and expensive after. None of it is an
OpenSpec requirement; this file is the project's memory, and `openspec validate` ignores it.

- [ ] Name check. "Spitz" is a working title that has never been searched for collisions
      (games called Spitz exist; chess tools may too). Verify name and trademark posture before
      the first tagged release or announcement, while renaming is still `git mv` and a sed.
- [ ] `CONTRIBUTING.md` with the DCO sign-off rule (ADR-0007) and a CI check for
      `Signed-off-by:` on PRs.
- [ ] License hygiene sweep: every source file carries no third-party copyleft header
      (`scripts/check-provenance.sh` holds the line), LICENSE is GPL-3.0-or-later (done),
      `cargo deny` or equivalent for crate licenses that are incompatible with GPL distribution.
- [ ] Release artifacts for Linux first-class platforms per ADR-0005 (packaging format is an
      open decision — see the change that adds CI release).
- [ ] Fill in `docs/benchmarks/reference-hardware.md` (ADR-0008) — the seventeen budget clauses
      are unverifiable until one machine is named.
- [ ] Connect the self-hosted `reference` runner and give `budgets.yml` its nightly schedule
      (ADR-0010).
