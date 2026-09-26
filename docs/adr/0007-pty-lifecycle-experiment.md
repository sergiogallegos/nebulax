# 0007 — Single-owner PTY lifecycle experiment

Status: implemented and locally verified on macOS 27 arm64, 2026-09-26. An integration experiment under the approved architecture; not a final runtime, supported OS floor or public ABI.

## Decision

Connect one PTY and direct child to one authoritative `Terminal`. Keep transport ownership in a single session owner and separate portable safe buffering (`pump.rs`) from macOS calls (`os.rs`). Use nonblocking master I/O, a fixed 4 KiB input buffer, one retained output event, and at most 64 operations per turn. Replies retain their write offset and complete before later output events; effect acceptance can independently apply backpressure. No terminal lock or callback crosses into the engine.

The existing engine reply/effect bounds stay unchanged. Stop reading the PTY when its output cannot be delivered; never hide congestion with an unbounded staging queue. Read EOF, child exit, transport error, explicit cancellation and fully drained completion are distinct conditions. EOF finishes the decoder only after retained input/output. Cancellation closes the master, kills/reaps the direct child and explicitly abandons pending data. Drop provides a best-effort cleanup fallback. Reaping currently waits synchronously and belongs off the main thread before native integration. Descendant/job-group supervision and graceful escalation remain unimplemented.

Resize first prepares a bounded clone of terminal state and checks the PTY's 16-bit dimensions, then applies `TIOCSWINSZ`, then publishes the prepared state. No engine input is processed between the ioctl and publication. Failed validation/ioctl does not change engine state. This costs a temporary duplicate engine allocation; future storage work must reduce that cost without reintroducing fallible state changes after OS resize. There is no total ordering guarantee between independently emitted child bytes and resize. Already generated replies retain query-time values.

The private OS module owns every descriptor through `File`. Master/slave opens use Rust's close-on-exec behavior; only the master is nonblocking. A fixed 128-byte `TIOCPTYGNAME` buffer avoids shared static `ptsname` storage. The child hook establishes a session/controlling terminal and resets signal mask/common dispositions, using only OS calls and errno-only errors after fork. Parent slave descriptors are dropped after spawning. The synthetic child verifies terminal/session/foreground identity and absence of inherited descriptors 3–255.

## Dependency and unsafe exception

Make `libc = 0.2.189` a direct, macOS-only dependency of the experiment. Its [published release history](https://docs.rs/crate/libc/0.2.189) was checked on 2026-09-26: it is the newest listed stable release; 1.0.0-alpha.4 is a prerelease. The existing lockfile already contains this exact package/checksum, and it has no normal transitive dependency under these features. It supplies maintained platform ABI definitions rather than terminal behavior. Rust/Python pins are unchanged; Python is only the deterministic test peer.

The workspace and terminal-core `unsafe_code = "forbid"` remain intact. This experiment explicitly opts into package-level `deny`, then allows unsafe only in private `os.rs`, with undocumented unsafe blocks denied by Clippy. Pointer validity, descriptor ownership and post-fork restrictions are documented at each call. The [Rust CommandExt contract](https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html#tymethod.pre_exec), [Apple PTY access documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/posix_openpt.3.html) and local Xcode 27 SDK `sys/ttycom.h`/libc declarations informed this boundary. This is a reviewed implementation exception, not permission for unsafe code throughout the core.

## Evidence and follow-up

The [research report](../../research/2026-09-26-pty-session.md) and [raw run](../../benchmarks/results/p0-08-pty-session/summary.json) record eleven tests: six macOS lifecycle/cleanup tests and five portable transport tests. Full local verification passes 73 Rust tests and all 19 owned fixtures / 244 replays. CI now includes macOS alongside Ubuntu; hosted results remain unverified.

Next establish a bounded owned visible snapshot and session-worker/private C lifetime contract, then connect a minimal AppKit window and basic input. Snapshot text must survive mutation/teardown, queued frames must be bounded, and child cleanup must not block the UI. Cursor, scroll-region and input-mode state remain engine-owned; add only what that path needs. A complete optimized storage framework or broad SGR coverage is not a prerequisite to the integration test.
