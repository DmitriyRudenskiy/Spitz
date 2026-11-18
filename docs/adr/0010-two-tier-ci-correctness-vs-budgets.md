# Two-tier CI: correctness gates PRs, budgets run on the reference machine

Pull requests gate correctness - build, unit and integration tests, provenance - on a standard
Linux runner; performance-budget tests are marked `#[ignore]` and run only in a separate
manual/nightly workflow on the reference hardware (ADR-0008), where failure files a report, not a
red X on someone's feature. Shared CI runners vary more than most of our budgets, so PR-gating
absolute numbers would train everyone to click "re-run" instead of reading failures; correctness
is where shared runners are honest. Rejected: budget tests in PR CI with medians and tolerance
(the noise floor swallows real regressions below ~30%), and keeping budgets out of CI entirely
(they rot into folklore - now they run on a schedule, and the artifact is the benchmark report).
