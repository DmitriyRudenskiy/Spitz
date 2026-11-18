# Test data is generated or handwritten, never third-party corpora

Fixtures are small, purpose-built game/comment/date cases committed as text; anything large
(million-game imports, search budgets, sweep throughput) is produced by a committed seeded
generator - pseudo-master games with scripted results, plausible headers, and a known defect mix -
not by vendoring someone else's database dump. A CC0 dump still adds a license line, an upstream
drift story, and fixtures nobody can extend deliberately; the generator is code, so it documents
the formats under test and its seed pins reproducibility. Rejected: committing a public dataset
(one more provenance ledger for test data nobody reads) and CI-time downloads (the product opens
no sockets, and a CI that needs a mirror is a CI that fails on unrelated Tuesdays).
