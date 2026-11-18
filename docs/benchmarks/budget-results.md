# Budget results

Per [ADR-0010](../adr/0010-two-tier-ci-correctness-vs-budgets.md) the budgets are
enforced by the tests themselves and run on the reference machine, where the whole library is
expected to fit in RAM. The workflow's artifact is the tests' own output, which each budget test
prints as one line naming its limit:

```
cargo test --release --workspace --locked -- --ignored --nocapture
```

Two more budgets — incremental signatures, and re-placing one row after an edit — are asserted
only in release builds and so run inside the ordinary suite; they print the same way.

## Measured

| Budget | Limit | This machine | Reference machine |
|---|---|---|---|
| 1M incremental signatures | 100 ms | 16 ms | |
| Completion over 1M names | 50 ms | 34 µs | |
| Re-placing one row of 100k | 100 ms | 142 µs | |
| Eight warm sorts over 100k rows | 400 ms | 30 µs (cold 13 ms) | |
| A filtered million games | 250 ms | 773 µs | |
| 1M games classified | 10 min | 418 ms | |
| A 100k-record book imported | 2 s | 33 ms (gm2600) / 104 ms (Elo2400) | |

**This machine is not the reference machine.** The column is an Apple-silicon development
laptop, release build, 2026-11-19. It is recorded so that a slow result on the reference machine
can be told apart from a budget that was never reachable, and so a regression here is visible
too. The reference machine's own numbers go in the empty column once its
[specification](reference-hardware.md) is filled in and the `reference` runner exists —
see [ADR-0008](../adr/0008-reference-hardware-one-named-machine.md).

The book row is the ignored golden test over a real book, run with
`SPITZ_BOOK=/path/to/book.bin`. It is the only one of these that needs a file the repository
does not carry.
