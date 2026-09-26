# PTY and engine integration — 2026-09-26

The owned engine now exchanges bytes with a real macOS PTY child. This validates the parser/output contract across an OS boundary without requiring renderer, GUI or interactive-shell compatibility. Implementation/policies are in [ADR 0007](../docs/adr/0007-pty-lifecycle-experiment.md) and the [experiment README](../experiments/pty-session/README.md).

## Method and observations

Run `scripts/pty-session` on macOS. It builds and runs the eleven experiment tests serially, records source and test-executable hashes, raw test output, compiler/OS/SDK versions and the dependency trees. The [retained run](../benchmarks/results/p0-08-pty-session/summary.json) passed all eleven tests with no ignored cases and unchanged sources during recording. Local host: macOS 27.0 arm64, Xcode 27.0, pinned Rust 1.98.1 and Python 3.14.7. No global installation or toolchain change was needed.

- A raw synthetic Python peer verifies that stdin/stdout/stderr are terminals, its session/process group owns the controlling terminal, and descriptors 3–255 were not inherited. It checks initial geometry, receives exact status/cursor replies, observes SIGWINCH and new geometry, then emits stdout/stderr and an incomplete UTF-8 prefix. The engine drains final output and inserts exactly one replacement scalar at EOF.
- Three hundred alternating title/bell/query records survive withheld effect acceptance. The child receives every status reply in order. Native actions are never executed; the host explicitly acknowledges inert events.
- Separate cases distinguish EOF before child exit, nonzero exit with final output, rejected geometry, missing executable, repeated sessions and idempotent explicit cancellation. A unit test drops a live session and verifies the direct child is already reaped (`waitpid` reports ECHILD).
- Portable scripted transports force one-byte writes, WouldBlock after each reply byte, interrupted reads/writes, incomplete UTF-8/CSI input, zero writes and broken pipes. A 1,000-record flood checks effect/reply order and retained-input/event bounds while blocked. These tests isolate transport edge cases without depending on a particular kernel queue capacity.

Full `scripts/verify` also passes: 73 Rust tests, official Unicode checks, two Python reference-harness tests (one optional live reference test skipped), exact known Alacritty gaps, and 19 strict owned fixtures / 244 replays. Original fixture expectations and earlier evidence are unchanged.

## Limits and implications

This is correctness evidence, not a throughput, latency, RSS or scheduler benchmark. The OS adapter is macOS-only; portable tests alone do not establish another platform's PTY behavior. CI is configured for macOS and Ubuntu, but hosted results were not checked. `libc` is newly direct for this adapter while retaining the already-locked package; the engine still has no dependencies or unsafe code.

No shell job tree, interactive input encoder, mode registry, GUI, FFI, renderer or native effect policy is implemented. Reaping may block; the future native path needs worker-owned teardown. Resize temporarily clones bounded engine state; this validates ordering/failure behavior without solving final storage costs. Geometry validation and disconnected-session failures are tested; kernel ioctl failures are propagated by inspection rather than a fault-injected syscall test. The experiment leaves caller scheduling and effect policy explicit.

Next test bounded snapshots and session-worker/private C lifetimes, then display the PTY-driven grid in a minimal native window with basic input. Keep one authoritative engine and preserve the tested parser/output ordering.
