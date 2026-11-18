# Work lands through PRs onto a green main; no commit stacks on red

Multiple agents and humans commit concurrently to this repository — which is exactly why main must
be a promise, not a aspiration: every unit of work lands through a pull request whose CI (correctness
tier, ADR-0010) is green at merge time, and a red main is the current top priority for whoever broke
it, not background scenery for the next feature. Direct pushes to main remain possible for spec-only
docs changes where a workflow run adds nothing (typos, ADR text), but anything touching `chess-core`,
`game-db`, or later crates goes through a branch and PR. Rejected: trunk-style direct pushes with
"fix forward" (with concurrent writers, fix-forward becomes fix-sideways — two sessions each assuming
the other will repair the shared tree, which is precisely how main arrived red today), and long-lived
feature branches (they breed merge surprises in a codebase whose formats are still moving).
