# Current work

Architecture and Phase D/0 are approved. We are in Phase 0A engine viability. The owned Rust engine has zero dependencies; all 19 existing fixtures pass through 244 replays. `scripts/verify` passes 36 Rust tests plus Unicode and reference-adapter checks. See [latest evidence](../research/2026-09-26-history-screen-reflow.md), [architecture status](../docs/architecture/STATUS.md) and [handoff](../HANDOFF.md) for completed work and owner decisions.

After reviewing the owner-provided Opus feedback, the next task changes from broader SGR coverage to **engine foundations before integration**. See the [assessment and sequence](../docs/architecture/FOUNDATION_REVIEW.md).

Immediate task: replace ordinary primary-resize rejection with an explicit bounded preservation/crop policy, including inactive primary state. Then measure cell/side storage and snapshot costs before introducing styles; separate bounded parser syntax from terminal operations and define typed replies/effects, cursor/mode/region ownership and snapshot/damage contracts.

The next integration gate is one PTY, the engine, a bounded snapshot, a minimal native window and basic input. Add only the protocol subset needed to exercise that path; broader coverage follows. Xcode 27.0/Swift 6.4 are verified, but PTY, FFI, native rendering, IME and accessibility implementation remain open.

The owner authorized committing and pushing all progress to `main`. Use Git for commit and synchronization state. No GitHub implementation issue has been created; this file is the temporary current-task source. No parallel-agent work is active. Existing Phase 0 and owned-engine approvals remain valid.
