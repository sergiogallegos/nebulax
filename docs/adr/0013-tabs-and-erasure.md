# 0013 — Shared tab stops and inclusive erasure

Status: implemented on 2026-09-27. Extends the cursor/region and style policies in ADRs 0010/0012. No dependency or private ABI change.

## Tabs

Store tab stops once per terminal, shared by primary and alternate screens and independent of saved cursor state. Use an owned vector of 64-bit words, growing only with the greatest requested width, capped at 1,024 words / 8 KiB by the existing geometry limit. Capacity is included in `storage_usage().tab_bytes`. Clones own independent bitmaps.

Initial stops occur at zero-based columns 8, 16, 24, and so on. HT and CSI I (CHT) move forward; CSI Z (CBT) moves backward. Omitted/zero counts mean one. Each command scans at most the visible width, regardless of count. Missing stops clamp to the respective edge. Movement does not print blanks, overwrite content, wrap, scroll or change the row; it closes grapheme attachment and cancels pending wrap. Landing on a wide continuation is allowed; a subsequent edit repairs the whole owner.

ESC H (HTS) sets the current column; CSI g/0g clears it; CSI 3g clears all. These edits preserve cursor/wrap and do not damage visible rows. Shrinking hides stops without deleting them. Growth exposes retained stops and initializes new words with the eight-column pattern. Clear-all also disables defaults for future growth; later HTS restores only explicitly selected stops. This resize policy is deliberate, rather than a full terminal-reset implementation. Invalid geometry leaves the entire terminal unchanged.

## Erasure

CSI K (EL) supports mode 0 from cursor through right edge, mode 1 from left edge through cursor and mode 2 for the whole line. CSI J (ED) supports mode 0 from cursor through page end, mode 1 from page start through cursor and mode 2 for the visible page. Ranges include the cursor cell and ignore origin/scroll margins after resolving its physical position. Position, margins, origin, current/saved rendition and tab stops remain intact; EL and visible ED cancel pending wrap.

Use the existing background-only erase style. Clearing either wide half clears its complete owner, including a half just outside the requested range. No erased data is pushed to history. When an erased range reaches a physical row start, detach the preceding soft-wrap link; when it reaches the end, detach the outgoing link. Removed structural padding uses the existing default-style policy. This can change the last history row's wrap metadata, but visible ED preserves its text/cells except structural padding. It prevents later reflow from joining surviving text across an erased boundary.

ED 3 clears saved lines of the active primary only. It preserves the visible grid, cursor and pending wrap. On alternate screen it is a no-op: hidden primary history is not modified. This explicitly bounds the scope of application-issued history clearing. Cell/tail ownership is released normally; the row container and style table can retain their bounded capacity/cache.

Unsupported modes, private selective-erasure forms, colon groups, extra parameters and intermediates produce `unsupported` without partial edits. ECH retains its existing behavior. Protected-cell selective erase, tab reports/reset commands and full DEC/ANSI compatibility remain open. The [xterm reference](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) supplies the standard command meanings; state ownership, resize and wrap policies above are Nebulax decisions implemented in original Rust.

## Evidence

Twelve new core tests check handwritten cell/cursor expectations across all two-way byte splits and single-byte delivery, defaults/custom/shared stops, cloned bitmap growth to maximum geometry, invalid-resize atomicity, inclusive erasure, wide/style ownership, margins/origin, history isolation, pending wrap and reflow boundaries. A clean Bash PTY case disables output processing to deliver literal tabs and emits UTF-8 via octal escapes. The AppKit probe checks HT/CHT positions and EL 2 after resize while retaining the previous style/input/lifecycle assertions.

Full verification passes 138 Rust tests, all 19 existing fixtures / 244 replays, native C/Swift checks and preview compilation. The separate GUI run passes. [Research evidence](../../research/2026-09-27-tabs-and-erasure.md) records sources and limits. Insertion/deletion, broader replies/modes and native IME/accessibility remain later work.
