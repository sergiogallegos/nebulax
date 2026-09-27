# AppKit window and basic input preview

This local macOS preview displays the authoritative Rust terminal's owned snapshots with AppKit/Core Text. It accepts basic keyboard input through the private C bridge, resizes the PTY and closes asynchronously after worker cleanup. No external framework, copied terminal engine or new Cargo dependency is used.

```sh
scripts/native-window             # Interactive synthetic peer; typed text is never executed
scripts/native-window --shell     # Clean Bash with TERM=dumb, no user startup/history
scripts/native-window --check     # Compile only; no window
scripts/native-window --self-test --output target/native-window/my-run
```

The executable is built under ignored `target/native-window`. Python is only the synthetic demo/test peer and build tooling; the optional shell path runs macOS Bash through a clean environment. This is a preview, not an installed or packaged terminal application. The GUI self-test requires a logged-in WindowServer session; ordinary `scripts/verify` compiles the window but does not open it.

## Tested path

One main-thread `NSView` retains one immutable frame while acquiring its replacement, then releases the old frame. Drawing shapes each lead cluster through Core Text and clips it to the engine-provided one/two-cell span. Continuations and wrap padding never draw detached text. No engine/registry lock is held while shaping. Cursor, UTF-8 text and geometry come from the snapshot. A full redraw and a 60 Hz snapshot polling timer are prototype choices, not performance claims or a final renderer selection.

Text commits are valid non-control UTF-8, at most 4,096 bytes per event. Return, backspace, tab, escape, arrows, home/end/delete and C0 control chords use the engine-owned normal/application cursor-mode encoder. The native view does not stage an unbounded queue: an event is accepted completely or rejected with a visible busy indication and beep. The Rust mailbox caps 64 events and 16 KiB; the pump can additionally hold one staged input and one current write of at most 4 KiB each. Key reservations conservatively charge four bytes. Native text does not become local screen text: only the child's PTY output updates the grid.

The pump completes every current input/reply before selecting another, prioritizing already generated replies before a staged input. When a write blocks it may read/feed the bounded PTY buffer, so echo cannot deadlock a partially written input. Generated replies remain in the engine's bounded queue and cannot splice into that write. Effect backpressure still preserves event order. Acceptance is queuing, not a delivery guarantee after cancellation, EOF or transport failure.

Window resize coalesces through the worker. Close and Cmd-Q request cancellation, keep the AppKit run loop alive, then release frame/session handles and close only after worker completion. The direct-child wait remains on the worker. Shell descendants/job supervision and graceful shutdown deadlines remain open.

## Evidence and limits

The [retained GUI run](../../benchmarks/results/p0-10-native-window/window.json) sends five `NSEvent` key events through the window responder path, checks echoed/edited Unicode text and an arrow, resizes to 90 × 25, captures the [rendered grid](../../benchmarks/results/p0-10-native-window/window.png), and checks timer progress plus completed child cleanup before close. The bitmap contains only the grid view, not the desktop or window chrome. Sources, binary hashes and command logs accompany it. It is a programmatic native-event test, not a physical keyboard latency test.

The demo peer is a bounded 512-byte line echo fixture, not a Unicode text editor. Basic Bash evidence is one clean prompt/command/echo/exit test with `TERM=dumb`; it does not establish zsh/fish/tmux/vim compatibility. The view currently uses basic `keyDown` characters and key codes. IME marked text, native accessibility text, selection, clipboard/paste, scrollback navigation, modified cursor keys, broader protocols/styles, font fallback/clipping quality and final GPU rendering remain unimplemented or unproven. See [ADR 0009](../../docs/adr/0009-native-window-and-basic-input.md) and [findings](../../research/2026-09-26-native-window.md).

The [cursor/region follow-up](../../research/2026-09-26-cursor-regions.md) extends the GUI test to six events, origin-relative replies, scrolling between fixed outside rows and application-mode Up. That evidence is in [p0-11](../../benchmarks/results/p0-11-cursor-regions/window.json); the earlier run above remains historical.

The [style follow-up](../../research/2026-09-27-bounded-styles.md) adds copied default/indexed/RGB styles, basic font traits and decorations. Backgrounds are painted in a separate pass before glyphs, including empty cells and wide continuations. The fixed prototype palette is resolved by the view; the Rust core owns symbolic colors. [Current GUI evidence](../../benchmarks/results/p0-13-styles/window.json) verifies styles and erased backgrounds across PTY/ABI/resize and retains the [rendered grid](../../benchmarks/results/p0-13-styles/window.png).

The [tab/erase follow-up](../../research/2026-09-27-tabs-and-erasure.md) clears stale startup content with ED 2 and verifies HT/CHT positions after EL 2 through the native path. [Current GUI evidence](../../benchmarks/results/p0-14-tabs-native/window.json) retains the earlier style/input/region/resize/cleanup checks.

The [insertion/deletion follow-up](../../research/2026-09-27-insertion-deletion.md) builds an `EDIT OK` row through DCH/ICH, inserts a temporary row, then deletes it inside a bounded region. [Current GUI evidence](../../benchmarks/results/p0-15-edit-native/window.json) verifies the result and retains all prior checks.

The [autowrap/visibility follow-up](../../research/2026-09-27-autowrap-visibility.md) verifies no-wrap edge output and a visibility-only frame. The view draws the cursor only when the snapshot requests it. [Current GUI evidence](../../benchmarks/results/p0-16-modes-native/window.json) includes hidden/visible captures and a pixel comparison confined to the cursor cell after the seventh native event.
