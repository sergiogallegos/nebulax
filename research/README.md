# Research evidence

- [First terminal replay](2026-09-26-terminal-replay.md): method, observed gaps and raw evidence.
- [Grapheme maintenance assessment](2026-09-26-grapheme-engine-assessment.md): expanded corpus and pinned-source comparison of alternatives.
- [Engine gap matrix](ENGINE_GAPS.md): tested slice and remaining public-v1 obligations.
- [Original architecture review](../review/README.md): historical proposal, sources, original specification and experiment methodology. Its original paths are retained to preserve links.

Current decisions live in [ADRs](../docs/adr/0001-approved-direction.md); research outcomes do not become accepted architecture automatically. Every quantitative claim needs reproducible raw samples and environment metadata. Correctness replay counts are not performance measurements.

- [Executed Ghostty reference comparison](2026-09-26-ghostty-reference-comparison.md): 19/19 matches, mode-off control, build provenance and owned Rust design implications.

- [First owned Rust engine slice](2026-09-26-owned-engine-slice.md): dependency-free core, Unicode 18, 14 matching fixtures / 166 replays and five pending cases.

- [Owned history, alternate screen and resize](2026-09-26-history-screen-reflow.md): all 19 matches / 244 replays, exact history budgets, independent screen state, reflow tests and explicit primary resize limitation.

- [Successful resize policy and evidence](../docs/adr/0004-cursor-anchored-resize.md): replaces the previous rejection limit with explicit cropping; 44 Rust tests and unchanged replay expectations.

- [Storage and snapshot experiment](2026-09-26-storage-snapshot-spike.md): 40/8/16-byte layouts, 450 measured samples, row reuse, independent snapshot lifetimes and side-storage counterexamples.
