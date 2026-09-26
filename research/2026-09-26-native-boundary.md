# Snapshot, worker and native boundary — 2026-09-26

A Swift 6 process now consumes owned frames from the Rust engine through a compiled private C ABI while a worker owns the live PTY and child. This completes the lifetime experiment preceding a native window. [ADR 0008](../docs/adr/0008-snapshot-worker-and-private-ffi.md) records the design and bounds; the [bridge README](../experiments/native-bridge/README.md) explains caller obligations.

## Reproduction and observations

Run `scripts/native-bridge --output <new-directory>` on macOS. The [retained run](../benchmarks/results/p0-09-native-boundary/summary.json) includes raw command outputs, source hashes, native library/C/Swift executable hashes, toolchain metadata and 70 Rust tests across the three relevant packages. Existing retained artifacts and fixture expectations are unchanged. Local toolchains remain Rust 1.98.1, Xcode 27.0/Swift 6.4 and Python 3.14.7 on macOS 27 arm64.

- Snapshot tests preserve combining/emoji text, wide-cell ownership and wrap padding after mutation, reflow, screen switches, engine destruction and thread transfer. Row versions survive skipped frames and cursor-only updates. An engine-valid dense cluster workload exceeds the snapshot budget and fails explicitly without mutating the engine/previous frame. Generation exhaustion also fails explicitly.
- Worker tests retain frames after worker destruction, observe failed spawn, resize a live child, request cancellation, and complete the existing 300-record title/bell/query peer with 600 denied native effects and every status reply delivered.
- Registry tests fill four session slots/eight frame leases, retain leases after releasing all originating sessions, reject excess acquisitions and stale/double releases, and demonstrate recovery of capacity when leases are released. Closing workers cannot free a slot before completion. An unwind test verifies status conversion at the C boundary.
- The actual C caller checks struct sizes/offsets, invalid arguments and unchanged error outputs, frame capacity and Unicode text after releasing its session. The Swift caller launches the deterministic PTY peer, requests resize, verifies resulting geometry/output, retains an older frame through mutation, reads both frames after session release, and exercises asynchronous close of a live child.

Full local `scripts/verify` passes 85 Rust tests, Unicode/data checks, formatting/Clippy, the unchanged reference gaps, all 19 owned fixtures / 244 replays, and the C/Swift programs. Mac-only tests are skipped by configuration on other hosts; Ubuntu's configured Rust subset is 72 tests. Hosted CI results have not been checked.

## Limits

This evidence establishes tested ownership/order behavior, not race-freedom under invalid callers, complete FFI fuzz coverage or measured performance. Valid C pointer ownership remains the caller's responsibility. Native callers must release frames only after readers finish. Allocation failure/abort is not recoverable. Direct Rust users can retain their own snapshots; the enforced session/frame counts belong to the C bridge.

Snapshots currently copy the full visible grid and compare rows; they do not implement final row-sharing storage or edit-time dirty tracking. Worker shutdown may wait for the direct child, but that wait runs on the worker. The two-millisecond polling cadence has not been benchmarked. No window, GPU renderer, keyboard encoder, shell job supervision, IME or accessibility implementation is claimed. Next connect the tested bridge to a minimal AppKit view and bounded basic input.
