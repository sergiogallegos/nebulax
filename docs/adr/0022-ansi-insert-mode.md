# ADR 0022: ANSI insert mode and streaming graphemes

Date: 2026-09-27. Status: accepted for the bounded research slice.

## Decision

Support ANSI IRM through `CSI 4 h` and `CSI 4 l`, with state-derived `CSI 4 $ p` replies (`CSI 4 ; 1 $ y` when set, `CSI 4 ; 2 $ y` when reset). Validate the entire ANSI mode list before applying it; repeated 4 is accepted, unknown/omitted/colon parameters reject the whole list. DEC private `?4` is a different unsupported mode. Other ANSI modes remain unrecognized.

IRM belongs to the terminal, defaults off and is independent of saved cursors, active screen and resize. Both DECSTR and RIS clear it. Mode changes close grapheme attachment but preserve cursor/pending-wrap and visible cells; no snapshot damage is reported just for toggling it. Input encoding is unaffected: IRM applies to received printable output.

New printable graphemes insert their initial display width (one or two physical columns) at the final position after existing delayed-wrap/wide-edge handling. Existing owners shift right on the same row; content crossing the right boundary is discarded without entering history or spilling onto another row. Starting in a wide continuation or clipping a wide owner clears the complete owner. Styles and cluster allocations move with surviving cells. Vacated cells retain only current background.

## Streaming width changes

Same-width extensions, including combining marks and ZWJ continuations, do not insert again. For a cluster changing width at the same position, shift the suffix by only the width difference. Shrinking pulls surviving suffix cells left and blanks the rightmost column; previously evicted content is not resurrected. The engine keeps no undo/shadow row for this purpose. A rejected extension at the cluster scalar limit makes no further shift.

A narrow cluster widening at the last column uses existing wide-owner placement: wrap to the next row when autowrap is enabled, otherwise clamp to the final two columns. Its old last-column slot is vacated, and its full new width is inserted at the resolved destination. These width-change/loss decisions are Nebulax policy; classical character-terminal documentation does not define Unicode grapheme behavior.

The screen-owned column mover is shared with ICH/DCH without cloning cells. Explicit column edits retain their existing wrap-detachment policy. Implicit insertion preserves the incoming soft-wrap link so normal printed runs remain reflowable, while clearing outgoing soft-wrap/padding before physical column movement. Width adjustments use the same rule. Ordinary automatic wrapping retains existing region/history behavior.

## Validation and limits

Ten dedicated core tests cover syntax/query distinction, all stream splits, independent complete-owner mapping over 462 small-grid insertion cases, narrow/wide/combining/selector/ZWJ behavior, edge relocation, global/reset ownership, styles, retained snapshots, row-buffer reuse, cluster eviction, controls and reply pressure. The mixed-stream invariant test also includes IRM, screen switches, wrap toggles and narrowing selectors. A real PTY peer verifies inserted/replaced snapshots, CPR/mode replies and resets. The native preview inserts `R` into `IM OK`, verifies mode replies and checks each resulting cell.

[Findings and retained evidence](../../research/2026-09-27-insert-mode.md) record the checks. No dependency, C ABI, fixture expectation or toolchain changed. No claim of complete VT/TUI compatibility or insertion throughput follows.

## References

- [DEC VT220 Programmer Reference Manual, section 4.6.4](https://www.zx.net.nz/computers/dec/vt220/doc/vt220-rm/chapter4.html): insert/replace syntax, rightward shifting and loss beyond the right margin (primary manual mirrored by zx.net.nz).
- [Xterm control sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html): ANSI SM/RM and DECRQM syntax. Both references accessed 2026-09-27; global ownership and Unicode policies above are explicit implementation decisions.
