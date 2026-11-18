# Design docs: ADRs for cross-change decisions, design.md per change

Decisions that outlive a single OpenSpec change — scope boundaries, naming policy, project-wide
document conventions like this one — are ADRs here; decisions about one change's behaviour stay in
that change's `design.md`, where its readers already are. Two parallel histories of the same
decisions were the alternative, and they drift: an ADR states a boundary once and points at the
change that implements it, so `design.md` files can be archived with their change without losing
the durable "why".
