# Bounded bracketed paste — 2026-09-27

The next bounded input slice implements [ADR 0021](../docs/adr/0021-bounded-bracketed-paste.md): explicit paste is separate from typed text/key input, mode 2004 is global and queryable, and one bounded frame stays immutable through short writes. No third-party dependency was added; the owned engine/session/bridge continue to use std plus our own crates/bindings.

The core validates nonempty UTF-8 up to 4,096 bytes, accepts TAB/CR/LF, rejects other control scalars and normalizes CRLF/LF to CR. This rejects embedded paste terminators before admission. Framing follows [xterm's documented mode 2004](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Bracketed-Paste-Mode); bounds, whole-event rejection, CRLF coalescing and DECSTR reset behavior are Nebulax policy. Paste mode does not enter saved cursor state or snapshots.

## Validation

- Six added core tests cover every two-way stream split, exact query bytes, normalization and multibyte limits, every disallowed C0/C1 control, global/saved/resize/reset ownership, malformed mode lists and 200 ordered replies under queue pressure.
- Three added portable runtime tests cover raw-plus-framing mailbox reservations and atomic rejection, every possible short-write cut through a Unicode paste, and a partial reply followed by paste negotiation, a partially written frame, reset, mode reply and a second unframed paste. No writer interleaving is allowed.
- One added macOS worker test and the Swift boundary caller independently drive a raw PTY peer through enabled, soft-reset and hard-reset paste exchanges. The child compares exact bytes before publishing its next marker; rejected payloads cannot produce extra bytes. C checks invalid pointer/length/UTF-8 arguments at the additive ABI entry point.
- The AppKit self-test retains its original grid/style/edit/resize/cursor checks and adds one fixed synthetic paste through Swift/C/worker/PTY, including Unicode, TAB and mixed line endings. It waits for the child to validate framing and publish `PASTE OK`; system clipboard access is never exercised.

Mode 2004 was previously a negative unknown-mode probe. The dedicated unknown-mode core test now uses unsupported mode 2005. Startup peers instead assert mode 2004 is recognized and reset. Original replay fixture expectations remain unchanged; this is new positive protocol coverage, not a reference-gap reclassification.

## Retained results

[Boundary evidence](../benchmarks/results/p0-20-paste-boundary/summary.json) records 176 core/runtime/bridge Rust tests plus compiled C and Swift callers, with source and artifact hashes. [Native evidence](../benchmarks/results/p0-20-paste-native/window.json) records eight native key events, one synthetic explicit paste, resize to 90×25, the prior cursor-only pixel comparison and asynchronous cleanup. `paste.png` is the subsequent visible-grid snapshot, separate from the unchanged-purpose hidden/visible comparison pair.

Full workspace verification passes 191 Rust tests, Unicode generation/data checks, formatting, Clippy, the SDK ABI/dependency guards, seven Python cases (six pass and one optional live-Ghostty skip), all 19 owned fixtures/244 replays, recorded Alacritty gap fingerprints, C/Swift execution and AppKit compilation. Portable configured coverage is 168 Rust tests; macOS adds 23. The PTY package has 32 tests (12 portable, 20 macOS-specific). No timing or product compatibility claim follows from these correctness tests.

## Limits and next step

The 4-KiB paste cap is deliberate for this slice; chunked large-paste admission/backpressure is future work. Accepting an event does not guarantee delivery after cancellation or a write failure. Clipboard/menu integration, IME, accessibility, extended keyboard modes, native GPU rendering and broad shell/TUI compatibility remain open. The next bounded protocol task is ANSI insert mode (IRM, mode 4), including wide-cell/grapheme behavior, reset/query semantics and PTY coverage; current implementation still reports mode 4 as unrecognized.
