# Native window and input — 2026-09-26

The verified parser/PTY/snapshot progress was committed and pushed to `main` as `bf93a6b` before this slice. The next local implementation connects that bridge to an AppKit/Core Text grid with a bounded input path. [ADR 0009](../docs/adr/0009-native-window-and-basic-input.md) records the decisions and [the preview README](../experiments/native-window/README.md) gives launch commands.

## Evidence

The [retained GUI run](../benchmarks/results/p0-10-native-window/window.json) opens a real window and routes five constructed `NSEvent` objects through its responder path: text, backspace, more Unicode text, Return and Up. The deterministic child echoes `You typed: Rust 界` and reports `Key: Up`. Resizing the content view yields a 90 × 25 engine snapshot. The grid is drawn and captured before closing; a later timer tick observes completed worker cleanup and releases the handles before closing the window. The [grid bitmap](../benchmarks/results/p0-10-native-window/window.png) was visually inspected for readable text, combining/CJK/emoji examples, input results and cursor placement.

Source hashes, preview/bridge binary hashes, raw commands and environment accompany the image. This is a capture of the AppKit grid view, not the whole window/desktop or proof of physical key-to-photon latency. The GUI test is local and separate from routine headless verification, which compiles the preview without opening a window.

Six Rust tests add independent evidence for normal key/C0 encoding, queue byte/event limits and recovery, native-input/reply ordering across forced one-byte writes, echo progress with a four-byte simulated transport buffer, real-PTY text/key input, and a clean Bash prompt/command/echo/exit. The duplex test exposed the need to drain readable echo while a partial write is blocked; the implementation now does so within the existing input/output bounds. The basic Bash case runs with `TERM=dumb`, startup files disabled and history directed to `/dev/null`; no user shell config/history is read or written.

Full `scripts/verify` passes 91 Rust tests, Unicode/data checks, formatting/Clippy, reference checks with unchanged known gaps, all 19 owned fixtures / 244 replays, C/Swift ABI programs, and AppKit compilation. The configured non-macOS Rust subset is 76 tests. Hosted CI and GUI tests on other macOS versions were not inspected. No new third-party package/version was added, and the owned engine remains dependency-free.

## Limits

The core remains the sole grid owner; the native view holds immutable frames and creates transient Core Text lines. The view currently redraws the whole visible frame. No Metal/wgpu choice, throughput, energy or latency result follows from this correctness run.

The interactive demo is a bounded echo fixture, not a production line editor. Normal keys work through basic `keyDown`; IME/marked-text, native text accessibility, selection/paste, scrollback UI and modifier/application keyboard modes remain future work. One clean Bash command does not establish advanced shell/TUI compatibility. Direct-child cleanup is tested, but shell job trees and graceful escalation remain open.

Next address coherent mode/cursor/scroll-region state and the protocol/input subset needed beyond this baseline, preserving the native lifecycle test as the integration regression. Native input/accessibility and final rendering still require their own acceptance evidence.
