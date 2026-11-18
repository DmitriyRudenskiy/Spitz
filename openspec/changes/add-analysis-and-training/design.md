# Design: Analysis and Training

## Context

M1 delivered the rules core, UCI hosting, the move-selection layer, the 2D board, the app shell, and the game database. This milestone adds the analytical and training surface.

## Key Decisions

1. **Report pipeline as background jobs**: the full-game report runs tiered MultiPV sweeps (fast tactical pass at depth 12; full report at depth 18+) on worker threads and streams classifications into the UI and then into game annotations. Thresholds (0.5 / 1.0 / 2.0 pawns) are user-configurable.
2. **Monte Carlo analysis**: fixed node-budget random playouts per candidate move, stored per position with a recency decay so re-runs merge instead of duplicating.
3. **Opening book**: own documented open format (position key → move stats). Import compiles PGN databases; books are files the user owns and shares out of band, and a book set is version-stamped so a ladder calibration can be replayed against the exact book that produced it.
4. **Style engine as selection policy, not engine internals**: an engine gives us ranked lines with scores; a style vector changes how a line is chosen from them (and which openings the book offers), because we cannot change a third-party engine's evaluation weights. Aggression and risk tolerance bias sampling toward sharp lines measured from the reported variation and score volatility, exchange orientation prefers trades among near-equal lines, positional bias prefers quiet improving moves. Personas are a style vector, a repertoire filter, and a phrase bank, with style profiles derived from public-domain game corpora and neutral configurable names. Every axis effect must remain measurable as a shift in move preference on a fixed calibration suite; an axis whose effect cannot be measured is dropped rather than shipped.
5. **Training center**: exercise store in local SQLite; generators produce exercises from the user's games and themed sets; spaced repetition (an SM-2 variant) drives tactics and repertoire scheduling; worksheets (Blitz & Train) render to PDF via the existing print pipeline.
6. **3D board via WebGL** (three.js) with automatic 2D fallback when no GPU is available; completes the board-ui requirement deferred from M1.

## Risks / Trade-offs

- Style axes are weaker through selection bias than through evaluation weights, and the effect ceiling is set by how much the reported lines differ. Mitigation: measure each axis on the calibration suite, publish the measured effect size with the axis, and drop axes that stay under the measurable threshold.
- Monte Carlo cost: playout budget is user-capped and results are cached aggressively.
- Persona phrase banks: content work is scoped (≥50 phrases per persona) and rate-limited to avoid spam; generated text is static, not a live LLM dependency.

## Migration / Rollout

No schema migration beyond the new local exercise store.
