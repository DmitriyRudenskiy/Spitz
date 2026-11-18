# Reference hardware is one named machine, written down

Performance budgets across the specs say "on the reference hardware" seventeen times while no
document named any machine; a budget without a machine is hearsay. Reference hardware is now the
single machine recorded in `docs/benchmarks/reference-hardware.md` - exact CPU, cores, RAM, disk,
kernel - and only that machine verifies SHALL-numbers, with a ±20% tolerance band between runs to
absorb thermal noise. Rejected: a min/recommended hardware matrix (three machines, three truths,
every failure arguable) and relative-only benchmarking against the previous release (silent slow
regressions that never trip an absolute budget look like success). When a budget stops holding on
the reference machine, the spec changes with a measured number, per rule 5 of the project context;
when the maintainer's hardware actually changes, this file and the budget report change together.
