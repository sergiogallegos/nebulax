# Current work

Architecture and Phase D/0 are approved. We are in Phase 0A engine viability. The owned Rust engine has zero dependencies; all 19 existing fixtures pass through 244 replays. `scripts/verify` passes 91 Rust tests plus Unicode and reference-adapter checks. See [latest policy and evidence](../docs/adr/0004-cursor-anchored-resize.md), [architecture status](../docs/architecture/STATUS.md) and [handoff](../HANDOFF.md) for completed work and owner decisions.

After reviewing the owner-provided Opus feedback, the next task changes from broader SGR coverage to **engine foundations before integration**. See the [assessment and sequence](../docs/architecture/FOUNDATION_REVIEW.md).

Resize rejection is resolved: every supported geometry succeeds with cursor-anchored retention and explicit crop/eviction counters, including inactive primary state. See [ADR 0004](../docs/adr/0004-cursor-anchored-resize.md).

The [storage/snapshot experiment](../research/2026-09-26-storage-snapshot-spike.md) is complete: 450 measured samples plus 45 warmups. [ADR 0005](../docs/adr/0005-storage-and-snapshot-direction.md) favors reusable row chunks, owned snapshots and a 16-byte integration candidate, while retaining 8-byte cells as an alternative. Global arena retention and snapshot text over-allocation need correction before production migration; the core representation remains unchanged.

The generic bounded parser and separate semantics are implemented, with ordered typed replies/effects and exact input resumption under output pressure. Twelve new tests cover syntax, cancellation, malformed input, queue limits and delivery order. [ADR 0006](../docs/adr/0006-bounded-parser-and-output.md) defines the contract and intentionally small supported protocol set.

The [single-PTY experiment](../research/2026-09-26-pty-session.md) is implemented: real child replies, controlling terminal, SIGWINCH/geometry, EOF/exit and cleanup, plus deterministic short-write/pressure tests. [ADR 0007](../docs/adr/0007-pty-lifecycle-experiment.md) defines ownership and the narrow unsafe OS-binding exception. It is not an interactive shell/runtime; direct-child reaping can block and job-tree supervision remains open.

The [snapshot/worker/private C boundary](../research/2026-09-26-native-boundary.md) is implemented and tested from C and Swift 6. Snapshots own bounded text, the mailbox retains only its latest frame, checked handles cap in-flight frames, and the worker owns child reaping. [ADR 0008](../docs/adr/0008-snapshot-worker-and-private-ffi.md) records limits: full visible copies/row comparison, a prototype idle cadence and no native window yet.

The [AppKit window/basic-input preview](../research/2026-09-26-native-window.md) now completes the first integrated path. Native-event typing, resizing, Core Text drawing and close after worker cleanup pass. The input mailbox is bounded, partial writes cannot interleave, and duplex echo is drained. One clean Bash prompt/command/exit passes with `TERM=dumb`; this is not broad shell/TUI compatibility. [ADR 0009](../docs/adr/0009-native-window-and-basic-input.md) records scope.

Immediate task: define coherent cursor/mode/scroll-region state and implement the smallest protocol/input slice needed beyond the clean dumb-shell case, with independent fixtures and the native regression preserved. Coordinate saved cursor, origin/scroll regions and application cursor-key encoding before widening VT features one sequence at a time. Styling/storage migration follows the recorded storage findings. Native marked-text/IME and accessibility remain separate acceptance gates; the current basic-key window does not satisfy them.

The first integration gate—one PTY, engine, bounded snapshot, minimal window and basic input—is implemented as a research preview. Broader protocol/native acceptance remains open. Xcode 27.0/Swift 6.4 are verified, and one headless PTY experiment is verified; a private C/Swift lifetime experiment is also verified. Production lifecycle, native rendering, IME and accessibility remain open.

The resize/storage checkpoint is committed and pushed as `c08ecd9`, following `db2acff`. The parser/output, PTY and snapshot/worker/FFI checkpoint is committed and pushed as `bf93a6b`. Native-window/input changes are verified locally and uncommitted; use Git for current commit and synchronization state. No GitHub implementation issue has been created; this file is the temporary current-task source. No parallel-agent work is active. Existing Phase 0 and owned-engine approvals remain valid.
