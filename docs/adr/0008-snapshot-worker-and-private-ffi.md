# 0008 — Owned snapshots, worker teardown and private C lifetimes

Status: implemented Phase 0 native-boundary experiment on 2026-09-26; private ABI v1, not a stable external interface or completed native app.

## Snapshot contract

Capture a complete owned visible frame with fixed cell metadata, one exactly sized UTF-8 text allocation, row wrap flags, cursor/screen state and per-row versions. Check a two-MiB cell/text/row payload limit before allocation; never publish partial/truncated text. The frame has no mutable state authority and no engine borrow. It remains valid after engine mutation, reflow, alternate switching and destruction.

Compare rows by cell role, width, text and wrap state against the previous published frame. Unchanged rows retain their versions; changed rows use the new monotonically increasing frame generation. Consumers compare these values against their last displayed frame, not generation minus one, because publication coalesces. Geometry/active-screen changes invalidate every row. Generations and handles fail explicitly on exhaustion rather than wrapping. Damage currently requires a full visible-row comparison; it is not edit-time grid damage tracking.

The eight-byte snapshot cell is only a transfer layout. It does not migrate the core's storage or replace ADR 0005's measured 16-byte candidate. Contiguous owned text addresses the prototype's per-cluster/snapshot over-allocation concern without adopting its global arena. Production storage reclamation and unchanged-row sharing remain future optimizations.

## Runtime contract

A single worker owns each PTY, engine, resize, output transport and child cleanup. UI-facing clients read an owned latest-frame mailbox and submit one coalescing resize request or cancellation. No engine lock survives frame access. Bell/title effects are explicitly denied and counted; PTY bytes never become control requests or native actions.

Close requests cancellation without waiting for child exit. The worker drops/reaps its session before publishing final status. Native session release returns busy until the worker body completes; closing handles continue to occupy capacity. A completed snapshot can outlive both the worker and native session handle. The underlying direct-child wait can still block the worker; it cannot block the UI through this API. Shell job trees, graceful escalation and production event readiness remain open. The initial idle cadence is two milliseconds with a condition variable for close/resize wakeups; this is not a scheduling/performance decision.

## Private ABI and bounds

Use a maintained C header with fixed-width fields, lengths and opaque integer handles. Export no Rust references, Vec/Arc layout or cluster slots. Four session slots and eight frame slots are fixed; each session can lease two frames. Frame slots remain charged after their originating session is released. Handles are monotonically assigned across both kinds, never reused. Stale/double release and capacity violations return status. The worker retains only the latest frame plus a previous frame while a candidate is built: at most 32 MiB aggregate snapshot payload across bridge caps, excluding engine/metadata/allocator storage.

Only the bridge's private `ffi.rs` permits unsafe pointer handling and symbol exports. The core still forbids unsafe, and the OS exception remains separate. Caller pointers must remain valid for their declared lengths; frame release cannot race use of its immutable views. Exported operations catch unwinding panics and return status, but do not promise recovery from invalid pointers, abort or OOM. Sources: the [Rust FFI safety/ABI guidance](https://doc.rust-lang.org/nomicon/ffi.html), [catch_unwind limitations](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html) and [JoinHandle completion query](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html#method.is_finished). No new third-party package/version or global toolchain change is introduced.

## Evidence and next gate

Five new core tests cover lifetime/Unicode/roles, skipped generations, resize/screen invalidation, byte overflow and generation exhaustion. Four worker tests cover finished frames, resize/cancel, failed spawn and denied effects with successful replies. Three bridge tests cover bounded slots/leases, stale handles, release after cleanup and unwind containment. Real C and Swift 6 callers check layout, invalid arguments, UTF-8 views, PTY replies, SIGWINCH/resize and reading frames after session teardown. The [research report](../../research/2026-09-26-native-boundary.md) links raw evidence. Full local verification passes 85 Rust tests and all 19 owned fixtures / 244 replays, plus C/Swift checks; hosted CI remains unverified.

Next connect a minimal AppKit window to these frames and add a bounded basic-input path. Keep shaping/rendering outside worker locks and merge input with replies without interleaving a partially written event. Native rendering, IME/accessibility, comprehensive terminal modes and production performance remain unproven.
