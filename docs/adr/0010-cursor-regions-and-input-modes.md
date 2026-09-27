# 0010 — Cursor state, scrolling regions and input modes

Status: implemented and locally verified on 2026-09-26. Extends ADRs 0003, 0006 and 0009 within the existing research profile.

## Ownership and behavior

`Screen` owns cursor movement, origin, inclusive top/bottom margins, one saved cursor, cluster-aware erasure and region scrolling. `Terminal` routes parsed actions and still owns decoding, grapheme printing and the terminal-wide application cursor-key flag. No additional grid, dependency or FFI layout is introduced.

Support CSI A/B/C/D, E/F, G, d, H/f; ESC 7/8 (save/restore), D/E/M (index/next line/reverse index); CSI r (vertical margins); and DEC private modes 1, 6 and existing 1049. Zero/omitted movement and position parameters default to one. Addressing is one-based; origin mode makes rows relative to the scrolling region and clamps them there. Setting/resetting origin or valid margins homes the cursor. CPR replies use the origin at query time. Relative vertical movement stops at the applicable margin; it does not scroll. Positions outside a region remain possible with origin disabled.

Forward scrolling enters history only for the full primary viewport. Partial-region and alternate scrolling discard displaced rows with `scrolled_without_history`; reverse index never pulls from history. Rows outside the region retain their cells. Changed adjacency severs stale soft-wrap links, including a link from history to a replaced first visible row. Internal wraps survive when their neighboring content still moves together.

Each buffer has independent margins, origin and a saved physical cursor/origin/pending-wrap tuple. Restore before save restores home with origin disabled. Restore clamps to current margins when restoring origin mode. The existing DEC 1049 policy preserves the complete primary screen, including its separate save slot, and creates a fresh alternate screen. Repeated entry/exit remains idempotent. This is our explicit screen-switch policy, not a claim of every xterm mode interaction. SGR attributes and character sets must join saved state when those features exist.

Any actual geometry change resets margins to the full new viewport on both buffers, retains origin and clamps saved physical coordinates. Only the live cursor participates in primary reflow; saved positions do not acquire logical text anchors. Pending wrap survives for saved positions only when their original column is still the new right edge. Same-size resize remains a complete no-op; invalid geometry remains atomic.

Application cursor mode is terminal-wide and persists across buffer switches. Arrows and Home/End use CSI in normal mode and SS3 in application mode. Encoding happens when the worker selects a queued key for writing. An already encoded partial write remains immutable even if later PTY bytes change the mode; newly selected keys use the updated state. Unprocessed PTY bytes have no universal ordering against native input.

Bounded mode lists are validated in full, then applied in wire order. A missing, colon-separated or unsupported mode rejects the entire list with `unsupported`; there is no partial mutation. This deliberate fail-closed research policy is narrower than terminals that ignore unknown list entries independently. Invalid numeric margins are ignored without moving the cursor; regions require at least two lines. A one-line viewport retains its initial full-screen region.

## Evidence and limits

[Thirteen core tests](../../crates/terminal/tests/cursor_and_regions.rs) use hand-written expectations, every two-way split and bytewise delivery, plus resize interleavings and snapshot row versions. Two transport tests add dispatch-time versus partial-write mode ordering and a live PTY handshake. The [native run and findings](../../research/2026-09-26-cursor-regions.md) verify the same behavior through the AppKit responder, bridge, worker and engine. Existing 19 replay fixtures remain unchanged.

This does not establish general TUI compatibility. Tab stops, erase-display/line variants, insertion/deletion, SGR, wrap-mode control, broader mode queries, IME and accessibility remain open. Compact storage and bounded cluster/style ownership should be settled before adding styles to every cell.

Behavior references: [DEC VT100 manual, Chapter 3](https://vt100.net/docs/vt100-ug/chapter3.html), especially CUP/CUU/CUD, DECOM, DECSTBM and IND; [xterm control sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html), especially DECCKM and cursor-key encodings. References informed behavior only; implementation is original Rust.
