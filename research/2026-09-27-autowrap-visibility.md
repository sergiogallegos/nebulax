# Autowrap and cursor visibility through the native path

Date: 2026-09-27. Builds on local tab/erase and insertion/deletion slices after pushed checkpoint `0fceb55`; all three subsequent slices remain uncommitted. [ADR 0015](../docs/adr/0015-autowrap-and-cursor-visibility.md) records exact state, edge and ABI policies.

Implemented screen-owned, saved/restored autowrap and terminal-wide cursor visibility. Disabled wrap overwrites the last column for narrow text and clamps complete wide owners into the last two columns. Primary resize still uses existing reflow. Visibility-only output publishes an owned frame without changing row damage, and AppKit updates the cursor overlay. No dependency/toolchain was added; core cells stay 16 bytes. Private ABI v3 extends frame metadata to 112 bytes with visibility and a reserved field.

## Validation

`scripts/verify` passes **160 Rust tests** on macOS, Unicode input/regeneration checks, formatting, warning-denied Clippy, Python/reference checks, strict owned replay, C/Swift checks and preview compilation. All original 19 fixtures / 244 delivery variants remain unchanged and matching. The configured portable set is 141 Rust tests; hosted CI was not inspected.

Ten new core tests check modes across byte partitions, widths 2–12 and all start columns, wide/selector ownership, pending wrap, saved/alternate/resize state, explicit scrolling, atomic invalid lists and immutable snapshots. The PTY package now has 25 tests: the new handshake waits for a hidden snapshot, then shows the cursor without printing or moving it, verifying a new generation with identical row versions and text. Held frames survive worker teardown.

The [boundary recording](../benchmarks/results/p0-16-modes-boundary/summary.json) contains 145 relevant Rust tests, compiled ABI v3 C/Swift checks and source/toolchain/binary hashes. The [native recording](../benchmarks/results/p0-16-modes-native/window.json) retains the existing styles, tabs, editing, modes/replies, resize and cleanup assertions. It additionally verifies disabled-wrap output at column 80 before resize and seven native events including the visibility acknowledgement.

The GUI captured [hidden](../benchmarks/results/p0-16-modes-native/hidden.png) and [visible](../benchmarks/results/p0-16-modes-native/window.png) cursor frames. Its pixel comparison requires a nonzero difference confined to the cursor cell, with identical text, row versions and cursor coordinates. Both captures were visually inspected. This verifies presentation correctness, not physical input latency or renderer performance.

## Limits and next step

The no-wrap wide-character clamp is an explicit product policy, not broad terminal-conformance evidence. Cursor shape/blink, insert mode, terminal resets, broader keyboard modes, native IME/accessibility and production lifecycle/performance remain open.

Next: define truthful device-attribute/status replies and test a controlled application-startup exchange. Advertised capabilities must match implemented behavior; the preview still makes no broad terminfo/TUI compatibility claim.
