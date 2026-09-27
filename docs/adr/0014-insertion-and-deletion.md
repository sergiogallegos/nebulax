# 0014 — In-place character and line edits

Status: implemented on 2026-09-27, building on the local tab/erase slice and pushed style checkpoint `0fceb55`. No dependency, toolchain or ABI change.

## Decision

Implement CSI @ (ICH), P (DCH), L (IL) and M (DL) in `Screen`; terminal semantics validates syntax and routes operations. Omitted/zero counts mean one. Extra fields, colon groups, private prefixes and intermediates are unsupported atomically; parser overflow never dispatches a truncated count. Counts clamp to remaining columns or rows, so work depends on retained geometry rather than a numeric parameter.

ICH shifts the current row's suffix right and discards overflow. DCH removes the requested columns and shifts the remaining suffix left. Use slice rotation and fill vacated cells with the current background-only erase style. No cluster cloning, row replacement or additional grid is needed. Surviving cells retain their original styles and owned tails; overwritten cells release their tails normally.

Counts mean physical columns. If insertion starts inside a wide character, clear that owner before shifting. Clear any wide owner cut by the right-hand insertion boundary. Deletion clears owners intersecting either edge of its deleted interval before shifting. Repair may clear a cell outside the nominal interval, but never increases the shift distance. An independent test maps complete original owner intervals to expected destinations rather than replaying the implementation's rotation algorithm.

Character edits preserve cursor row/column, cancel pending wrap and close grapheme attachment. Remove outgoing soft-wrap links and structural padding before shifting, so padding cannot become interior content. If the edit/repair reaches column zero, detach the preceding link as well. An interior suffix edit preserves the incoming link. Structural padding follows the existing default-style removal policy.

IL/DL operate only when the cursor is inside the vertical scroll region, moving rows from its physical position through the bottom margin. Outside the region they leave grid, cursor and pending wrap unchanged. Inside it, clamp counts, rotate existing row buffers and clear inserted/exposed rows with the current background-only style. Cursor position is retained and pending wrap cancelled. Origin mode determines physical positioning but does not expand the edited region.

Preserve soft-wrap links between surviving rows that still remain adjacent; detach links at the insertion/deletion boundary and the new bottom/blank boundary. Discarded rows never enter history, including full-screen IL/DL. Hidden primary state remains untouched during alternate-screen edits. History contents remain intact except a preceding structural wrap link/padding when editing begins at visible row zero. Successful line edits report the existing `scrolled_without_history` diagnostic.

The command meanings follow the [xterm reference](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) and DEC's [IL](https://vt100.net/docs/vt510-rm/IL.html)/[DL](https://vt100.net/docs/vt510-rm/DL.html) documentation. Wide-grapheme repair, wrap handling and background policy above are explicit Nebulax choices. Implementation is original safe Rust.

## Validation and limits

Ten new core tests cover independent ASCII expectations, every two-way split plus single-byte delivery, wide-owner cuts across small geometries/counts, styled multi-scalar movement, defaults and saturation, region/origin behavior, buffer reuse, tail reclamation, history/alternate isolation, snapshot lifetime/damage, wrap boundaries, pending wrap and malformed commands. A clean Bash PTY case verifies all four commands and background-only rows. The native probe checks character edits, inserts a temporary row and deletes it while preserving surrounding protocol markers.

Full verification passes 149 Rust tests, unchanged 19 fixtures / 244 replays, C/Swift lifetime checks and AppKit compilation; the separate GUI test passes. [Findings and raw evidence](../../research/2026-09-27-insertion-deletion.md) retain provenance. This does not add insert mode, horizontal margins, scrolling commands, protected cells or a complete terminfo profile. Native IME/accessibility and production performance remain separate gates.
