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

- [PTY lifecycle experiment](2026-09-26-pty-session.md): one macOS child/engine round trip, eleven transport/lifecycle tests, bounded buffering and cleanup limits.

- [Snapshot, worker and private C/Swift boundary](2026-09-26-native-boundary.md): owned frames, bounded handles, worker cleanup and native lifetime evidence.

- [Native window and basic input](2026-09-26-native-window.md): AppKit/Core Text drawing, bounded full-duplex input, native-event GUI tests and the clean dumb-shell baseline.

- [Cursor/region/application-key integration](2026-09-26-cursor-regions.md): screen state ownership, thirteen core tests, live PTY negotiation and native protocol regression.

- [Integrated compact storage](2026-09-26-compact-storage.md): real 16-byte cells, direct tail reclamation, row reuse, 100-sample before/after measurements and native regression.

- [Bounded styles and native rendition](2026-09-27-bounded-styles.md): reclaimable style IDs, SGR subset, copied snapshot palettes, ABI v2 and styled AppKit evidence.

- [Tabs and line/display erasure](2026-09-27-tabs-and-erasure.md): bounded shared stops, wide-aware ranges, history policy and clean-shell/native evidence.

- [Character and line insertion/deletion](2026-09-27-insertion-deletion.md): in-place ownership, wide-cell cuts, region/history isolation and shell/native evidence.

- [Autowrap and cursor visibility](2026-09-27-autowrap-visibility.md): saved wrap modes, whole wide-owner edge policy, metadata-only snapshots and native pixel evidence.
