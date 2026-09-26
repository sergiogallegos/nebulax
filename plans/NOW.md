# Current work

Architecture and Phase D/0 are approved. We are in Phase 0A engine viability. The owned Rust engine has zero dependencies; all 19 existing fixtures pass through 244 replays. `scripts/verify` passes 85 Rust tests plus Unicode and reference-adapter checks. See [latest policy and evidence](../docs/adr/0004-cursor-anchored-resize.md), [architecture status](../docs/architecture/STATUS.md) and [handoff](../HANDOFF.md) for completed work and owner decisions.

After reviewing the owner-provided Opus feedback, the next task changes from broader SGR coverage to **engine foundations before integration**. See the [assessment and sequence](../docs/architecture/FOUNDATION_REVIEW.md).

Resize rejection is resolved: every supported geometry succeeds with cursor-anchored retention and explicit crop/eviction counters, including inactive primary state. See [ADR 0004](../docs/adr/0004-cursor-anchored-resize.md).

The [storage/snapshot experiment](../research/2026-09-26-storage-snapshot-spike.md) is complete: 450 measured samples plus 45 warmups. [ADR 0005](../docs/adr/0005-storage-and-snapshot-direction.md) favors reusable row chunks, owned snapshots and a 16-byte integration candidate, while retaining 8-byte cells as an alternative. Global arena retention and snapshot text over-allocation need correction before production migration; the core representation remains unchanged.

The generic bounded parser and separate semantics are implemented, with ordered typed replies/effects and exact input resumption under output pressure. Twelve new tests cover syntax, cancellation, malformed input, queue limits and delivery order. [ADR 0006](../docs/adr/0006-bounded-parser-and-output.md) defines the contract and intentionally small supported protocol set.

The [single-PTY experiment](../research/2026-09-26-pty-session.md) is implemented: real child replies, controlling terminal, SIGWINCH/geometry, EOF/exit and cleanup, plus deterministic short-write/pressure tests. [ADR 0007](../docs/adr/0007-pty-lifecycle-experiment.md) defines ownership and the narrow unsafe OS-binding exception. It is not an interactive shell/runtime; direct-child reaping can block and job-tree supervision remains open.

The [snapshot/worker/private C boundary](../research/2026-09-26-native-boundary.md) is implemented and tested from C and Swift 6. Snapshots own bounded text, the mailbox retains only its latest frame, checked handles cap in-flight frames, and the worker owns child reaping. [ADR 0008](../docs/adr/0008-snapshot-worker-and-private-ffi.md) records limits: full visible copies/row comparison, a prototype idle cadence and no native window yet.

Immediate task: connect a minimal AppKit window to the owned frames and implement bounded basic input. Display the PTY-driven grid, exercise window resize and close, and keep text shaping/presentation outside worker locks. Queue native user input with replies without interleaving any partially written event; keep input-mode/cursor/region state in the engine. Begin with deterministic child/input scenarios, then evaluate the protocol subset needed for a basic shell. Do not advertise broad VT, IME/accessibility or renderer performance support from this initial window.

The next integration gate is one PTY, the engine, a bounded snapshot, a minimal native window and basic input. Add only the protocol subset needed to exercise that path; broader coverage follows. Xcode 27.0/Swift 6.4 are verified, and one headless PTY experiment is verified; a private C/Swift lifetime experiment is also verified. Production lifecycle, native rendering, IME and accessibility remain open.

The resize/storage checkpoint is committed and pushed as `c08ecd9`, following `db2acff`. The owner authorized committing and pushing the verified parser/output, PTY and snapshot/worker/FFI checkpoint; use Git for current commit and synchronization state. No GitHub implementation issue has been created; this file is the temporary current-task source. No parallel-agent work is active. Existing Phase 0 and owned-engine approvals remain valid.
