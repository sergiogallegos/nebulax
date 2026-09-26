# Owned history, alternate screen and resize — 2026-09-26

Status: implemented and locally verified within Phase 0A. Project architecture is accepted; full engine/native feasibility remains in progress. See [architecture status](../docs/architecture/STATUS.md).

## Outcome

The owned engine now matches **all 19 existing fixtures across 244 replays**, with no deferred fixtures, expectation differences or observed delivery differences. The five previously pending cases now execute history, alternate-screen restoration and primary resize/reflow. Original expected states and prior retained evidence were not changed.

Added bounded primary history, idempotent mode-1049 screen switching, primary reflow and alternate crop/pad. Reflow preserves grapheme owners, printed spaces, hard/soft line boundaries and the cursor for supported cases. Resizing while alternate is active also updates the saved primary. The engine retains **zero Cargo dependencies**, safe Rust and no I/O/native effects.

Evidence: [replay](../benchmarks/results/p0-06-owned-history/replay.json), [environment and core dependency tree](../benchmarks/results/p0-06-owned-history/environment.json), [source hashes](../benchmarks/results/p0-06-owned-history/sources.json), [workspace dependencies](../benchmarks/results/p0-06-owned-history/dependencies.json), [artifact hashes](../benchmarks/results/p0-06-owned-history/artifacts.json). The baseline normalized snapshot now includes history rows and the active-screen identity; row/cell expectations are still the unchanged independent corpus. Existing reports remain historical evidence of their exact source snapshots.

## Validation

`scripts/verify` passes Unicode input/table checks, format, Clippy with warnings denied, **36 Rust tests**, two Python adapter tests with one optional live Ghostty test skipped, the unchanged Alacritty known-gap replay and strict owned-engine replay. The 853 official Unicode 18 segmentation cases and 8,000 malformed UTF-8 triples remain covered by the core tests. No third-party lockfile entry changed.

The 14 added tests check:

- Row and cell history limits independently, oldest-row eviction and explicitly disabled history.
- Repeated alternate entry/exit, separate history and restoring a pending-wrap primary cursor.
- Inactive-primary reflow while alternate clipping removes both halves of a wide character.
- Repeated narrow/wide round trips preserving spaces, hard breaks, Indic/Hangul and emoji clusters.
- Height shrink/growth, exact-edge cursor movement, and cursor positions inside wide tails.
- History-budget changes with width, invalid/no-op resize atomicity and partial UTF-8/CSI across resize.
- Malformed private modes, explicit protection of primary text below the cursor, and extreme aspect ratios.
- Mixed feed/resize/screen-switch sequences with bytewise versus whole-feed state/diagnostic equivalence.

The earlier unknown-mode test now uses still-unsupported mode 1048 rather than newly implemented 1049. The no-history scrolling test explicitly disables history. These are test-scope updates for implemented features; no terminal fixture expectation was adjusted to match output.

## Limits and continuation

[ADR 0003](../docs/adr/0003-history-screen-and-reflow.md) records the retention, viewport and crop policies. Retained history is bounded in physical rows/cells, and reflow pads only retained rows to avoid huge intermediate grids. These are implementation bounds, not performance/RSS measurements or allocation-failure recovery evidence.

An important remaining edge is explicit: when a primary viewport would need to crop populated text below its cursor, `resize` returns `PrimaryContentWouldBeCropped` without changing state. This applies to saved primary state too. That limitation must be resolved or handled before native-window/PTY integration. Alternate cropping and limit-driven history eviction are reported rather than hidden.

Matching 19 cases does not establish complete terminal conformance. SGR/styles, broader cursor and erase operations, margins, tabs, additional modes, standalone zero-width/control-boundary compatibility, comprehensive width policy, text access and native lifecycle remain open. Next: expand bounded VT cursor/erase and SGR coverage against independent fixtures, and resolve the primary resize edge before presenting this as a window-ready engine. No hosted CI, performance benchmark or native app was run or released.
