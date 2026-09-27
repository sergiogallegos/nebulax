# Private snapshot/worker bridge

This macOS research crate exports [a private C header](include/nebulax.h) and a `cdylib` consumed by real C and Swift tests. It adds no third-party dependency. Unsafe code is denied except in `ffi.rs`; the terminal core still forbids unsafe and owns all terminal state.

```sh
scripts/native-bridge
scripts/native-bridge --output target/native-bridge/evidence
scripts/verify
```

The first command builds and executes C layout/argument/lifetime checks and a Swift 6 PTY/resize/teardown check. `--output` additionally records 145 Rust tests across the core, PTY and bridge packages, exact sources, raw command results, native binary hashes and toolchains. Full workspace verification currently runs 160 Rust tests on macOS, plus these C/Swift checks. Non-macOS verification runs the portable Rust subset and explicitly skips the native smoke checks.

## Native ownership contract

Start with trusted executable/argument bytes. `nb_session_start` returns a checked integer handle while the worker performs PTY launch. Poll status to distinguish starting, running, exited, stopped and failed. `nb_session_resize` queues/coalesces geometry; success is not an OS acknowledgement. The worker owns the PTY, engine, reply writes and child reaping. Bell/title effects are counted and denied; they never execute native actions.

Use `nb_frame_acquire` with your latest generation, then `nb_frame_view`. The complete frame owns cell metadata, UTF-8 text and style values; it contains no engine borrows, cluster-arena IDs or Arc pointers. Read it without holding engine/registry locks. Row versions are compared with the last displayed frame, so a consumer may skip intermediate frames. Geometry/screen switches invalidate every row; cursor and wrap state are explicit.

`nb_session_close` requests cancellation; it does not join or wait. Keep the session handle until `status.finished` is true, then release it. Early release returns `NB_BUSY` and keeps the slot occupied. Retained frames remain readable after session release. Release each frame separately, synchronizing so its views are not being read concurrently. Never dereference its pointers after release. Duplicate/stale handles return `NB_INVALID`; handle numbers are never reused, even across object kinds. Rust panics that unwind are translated to status; invalid foreign pointers, allocation failure and abort are not recoverable guarantees.

All input pointer lengths, validity and output ownership obligations are in the header. Strings are copied and bounded before launch. The interface is process-local trusted native API, not an agent/control channel. No path from PTY bytes can start a session or call the ABI.

## Explicit bounds and current costs

- Four live session slots, including closing workers. Failure to release handles consumes bounded capacity rather than allowing unlimited workers.
- Two acquired frames per session and eight total, including frames whose originating session has been released.
- One latest-frame mailbox per worker. New frames replace stale mailbox frames; consumers do not queue a history of frames.
- Two MiB maximum cell/text/row/style payload per snapshot, checked before constructing payload arrays. Overflow stops the worker with a visible failure and retains the last valid frame; no truncated text is published.

The worker may hold its previous frame while building a candidate. Across four sessions and eight acquired handles, this bounds snapshot payload to at most 32 MiB; engine state, resize clones, worker stacks, metadata and allocator overhead are separate. These are logical payload bounds, not RSS measurements. Direct Rust users of `Snapshot`/`Worker` must bound their own retained values; the handle caps are enforced by this C bridge.

Snapshots are copied from the authoritative visible grid with exact text sizing. Row versions are derived by content comparison, not engine edit-time dirty tracking. Private ABI v3 uses 12-byte snapshot cells, 12-byte style values and 112-byte frame metadata. Core cells remain 16 bytes. Callers check version 3 before using the interface. Full-frame copy and a 2 ms idle worker cadence are prototype choices, not performance conclusions. History export remains open; copied styles and color-sensitive damage are implemented. A basic key encoder and a separate AppKit preview now consume this bridge; IME/accessibility remain unimplemented.

See [ADR 0008](../../docs/adr/0008-snapshot-worker-and-private-ffi.md), [findings](../../research/2026-09-26-native-boundary.md) and [retained evidence](../../benchmarks/results/p0-09-native-boundary/summary.json). The [AppKit/basic-input preview](../native-window/README.md) now connects this frame lifecycle to a view. `nb_session_text/key` accept bounded native input; see [ADR 0009](../../docs/adr/0009-native-window-and-basic-input.md) for byte/event limits and partial-write ordering.

[ADR 0012](../../docs/adr/0012-bounded-styles-and-native-rendition.md) defines style encoding, ownership and the v2 layout. [Style evidence](../../benchmarks/results/p0-13-style-boundary/summary.json) extends the historical v1 run with styles and retained palettes.

[Tab/erase boundary evidence](../../benchmarks/results/p0-14-tabs-boundary/summary.json) records 123 relevant Rust tests and the unchanged ABI v2 C/Swift checks after tab/erase integration.

[Editing boundary evidence](../../benchmarks/results/p0-15-edit-boundary/summary.json) records 134 relevant Rust tests and the unchanged ABI v2 checks after insertion/deletion.

[Current boundary evidence](../../benchmarks/results/p0-16-modes-boundary/summary.json) records 145 relevant Rust tests plus C/Swift ABI v3 checks. `cursor_visible` is frame-owned; `reserved` is zero. Cursor metadata can change without row damage, so renderers must handle the overlay separately. [ADR 0015](../../docs/adr/0015-autowrap-and-cursor-visibility.md) records the layout change.

The [owned PTY binding migration](../../docs/adr/0020-owned-macos-pty-bindings.md) removes the bridge's last transitive third-party Cargo dependency. Its closure now consists only of this crate, the owned PTY crate and the terminal core. The private C ABI remains version 3; system libSystem/AppKit/Core Text are separate native dependencies.

The additive private-v3 `nb_session_paste` entry point copies bounded explicit native paste input. It has the same checked-session/atomic admission behavior as text/key input; the core owns validation, CR line-ending normalization and optional framing at dispatch. C tests check null/oversized/invalid-UTF-8 input; Swift drives a real PTY through enabled, soft-reset and hard-reset paste exchanges and checks rejected payloads. No struct layout changed and no public SDK compatibility promise is implied. See [ADR 0021](../../docs/adr/0021-bounded-bracketed-paste.md) for the 4,096-byte payload and queue policy.
