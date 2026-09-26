# 0009 — Minimal native window and bounded input

Status: implemented local macOS walking path on 2026-09-26. This is a research preview, not the completed native MVP, final renderer or shell compatibility profile.

## Decision

Connect the private bridge to one AppKit window. Shape owned lead-cell clusters with Core Text, place/clip them according to the authoritative grid, and draw a cursor from snapshot state. Acquire the next frame before releasing the old one; render only while its lease is live. Shaping never holds worker/engine locks. Full redraw and a 60 Hz latest-frame polling timer establish an initial functional path; Metal/wgpu comparison, caching, font quality and frame scheduling still need evidence.

Accept committed non-control UTF-8 and a normal-mode basic-key subset. Keep key encoding in the Rust terminal crate, invoked on its session owner; native code only maps native keys to semantic key IDs. Application-cursor and extended-keyboard modes remain unsupported until the engine owns those modes. Do not interpret PTY text as input/control requests.

Bound the shared input mailbox to 64 events / 16 KiB and each text event to 4 KiB. Acceptance is atomic; full queues return a visible failure rather than dropping bytes or creating an unbounded native staging queue. The pump can additionally hold one staged event and one current write, each at most 4 KiB. Reuse its single writer/offset for replies and native input. Complete an in-flight write before selecting another; already generated replies precede staged input, while events not yet parsed have no global time ordering against user input.

Continue bounded reads/feeding when a partial write is blocked, preserving that writer and offset. This prevents echo-induced duplex deadlock. If generated output reaches the engine's bounded queue, its consumed count stops further input feeding; the existing pressure contract remains intact. Cancellation/exit/error can abandon queued input, so acceptance does not promise eventual delivery.

Resize remains worker-owned/coalescing. Window close and application quit request cancellation, continue servicing AppKit, and release/close after worker completion. No UI-thread join or child wait is added. The optional shell preview uses a clean environment, `TERM=dumb`, no startup files and `/dev/null` history. It is an opt-in basic-shell scenario; the default is a synthetic interactive peer that never executes entered commands.

## Evidence and next gate

Full local verification passes 91 Rust tests, 19 owned fixtures / 244 replays, C/Swift checks and compilation of the AppKit preview. Six new Rust tests cover key encoding, atomic queue bounds, input/reply byte ordering, duplex echo progress, a real PTY input peer and a controlled Bash prompt/command/exit. The separate GUI run passes native-event typing/backspace/arrow input, a 90 × 25 resize, actual Core Text drawing and asynchronous window close after worker cleanup. [Findings](../../research/2026-09-26-native-window.md) and [raw GUI evidence](../../benchmarks/results/p0-10-native-window/window.json) record the method and limits. Existing corpus expectations and earlier artifacts are unchanged; no dependency/toolchain change occurred.

This completes the initial PTY → engine → owned snapshot → private C → native window path with basic input. Next define coherent mode/cursor/scroll-region state and the minimal protocol/input behaviors needed beyond the clean dumb-shell case, then expand styling against the storage findings. Native marked-text/IME and accessibility remain explicit acceptance gates, not features implied by this basic `keyDown` preview. Shell job supervision, production scheduling and renderer performance also remain open.
