# Current work

Architecture and Phase D/0 are approved. We are in Phase 0A engine viability. The owned Rust engine has zero dependencies; all 19 existing fixtures pass through 244 replays. `scripts/verify` passes 50 Rust tests plus Unicode and reference-adapter checks. See [latest policy and evidence](../docs/adr/0004-cursor-anchored-resize.md), [architecture status](../docs/architecture/STATUS.md) and [handoff](../HANDOFF.md) for completed work and owner decisions.

After reviewing the owner-provided Opus feedback, the next task changes from broader SGR coverage to **engine foundations before integration**. See the [assessment and sequence](../docs/architecture/FOUNDATION_REVIEW.md).

Resize rejection is resolved: every supported geometry succeeds with cursor-anchored retention and explicit crop/eviction counters, including inactive primary state. See [ADR 0004](../docs/adr/0004-cursor-anchored-resize.md).

The [storage/snapshot experiment](../research/2026-09-26-storage-snapshot-spike.md) is complete: 450 measured samples plus 45 warmups. [ADR 0005](../docs/adr/0005-storage-and-snapshot-direction.md) favors reusable row chunks, owned snapshots and a 16-byte integration candidate, while retaining 8-byte cells as an alternative. Global arena retention and snapshot text over-allocation need correction before production migration; the core representation remains unchanged.

Immediate task: separate bounded generic VT parser events from terminal meaning, and define a bounded typed reply/effect channel with ordering and backpressure. Cover multi-parameters, colon subparameters, private prefixes/intermediates, cancellation and limit recovery with independent tests. Coordinate cursor/mode/region and snapshot ownership contracts for the early native path; avoid broad protocol expansion before that integration.

The next integration gate is one PTY, the engine, a bounded snapshot, a minimal native window and basic input. Add only the protocol subset needed to exercise that path; broader coverage follows. Xcode 27.0/Swift 6.4 are verified, but PTY, FFI, native rendering, IME and accessibility implementation remain open.

The earlier progress checkpoint is committed and pushed as `db2acff`. The owner authorized the resize/storage checkpoint on main; use Git for current commit and synchronization state. No GitHub implementation issue has been created; this file is the temporary current-task source. No parallel-agent work is active. Existing Phase 0 and owned-engine approvals remain valid.
