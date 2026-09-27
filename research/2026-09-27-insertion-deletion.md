# Character and line insertion/deletion

Date: 2026-09-27. Extends the verified local tab/erase work; both slices remain uncommitted after pushed checkpoint `0fceb55`. [ADR 0014](../docs/adr/0014-insertion-and-deletion.md) records precise state and boundary policies.

ICH/DCH now shift physical columns with complete wide-owner repair. IL/DL move whole rows only between the cursor and bottom scroll margin. Existing cell/row buffers are rotated in place; surviving text/styles retain direct ownership, and dropped tails are freed. Blanks use current background only. No dependency, toolchain, core-cell size, snapshot layout or private ABI changed.

## Evidence

Full `scripts/verify` passes **149 Rust tests** on macOS, Unicode hash/regeneration, formatting, warning-denied Clippy, Python/reference checks, all 19 owned fixtures under 244 delivery variants, compiled C/Swift checks and preview compilation. The configured portable set contains 131 Rust tests; hosted CI was not inspected.

Ten added core tests include an independent owner-interval mapping oracle for widths 2–12, three mixed-width layouts, every cursor column and counts through/beyond the row width, plus maximal parameters. Other cases verify exact rows/cursors, all two-way input splits, byte-at-a-time delivery, styled multi-scalar ownership, buffer pointer reuse, immediate dropped-tail reclamation, margins/origin, history/alternate isolation, wrap boundaries and immutable older snapshots.

The PTY package now has 24 tests, including three clean Bash interactions. The new interaction disables output processing and exercises ICH/DCH and IL/DL, with independent final row/style assertions. It remains a controlled `TERM=dumb` scenario, not broad shell/TUI compatibility evidence.

The [boundary run](../benchmarks/results/p0-15-edit-boundary/summary.json) records 134 relevant Rust tests plus C/Swift lifetime checks with command logs, source/toolchain and binary hashes. The [native run](../benchmarks/results/p0-15-edit-native/window.json) creates the `EDIT OK` row through character edits, then inserts and deletes a temporary row within margins. Assertions verify the resulting cell positions and preserved outside/protocol rows after resize. The [grid capture](../benchmarks/results/p0-15-edit-native/window.png) was visually inspected. Existing styles, tabs, input modes, replies, resizing and asynchronous close remain covered.

All historical artifact sets and original replay fixtures are preserved. No performance comparison was rerun; in-place movement is a code/ownership property, not a measured speed claim.

Next: autowrap/cursor visibility and their saved-state/snapshot/native behavior, then truthful device-attribute replies and a controlled application-startup probe. Insert mode, horizontal margins, native text/accessibility and production scheduling remain open.
