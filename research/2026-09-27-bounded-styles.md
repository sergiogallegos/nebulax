# Bounded SGR styles through the native path

Date: 2026-09-27. This extends pushed compact-storage checkpoint `ece28c1` with original Rust style ownership and AppKit rendering. No dependency, toolchain, original replay fixture or historical result changed. [ADR 0012](../docs/adr/0012-bounded-styles-and-native-rendition.md) is the policy source.

## Observations

The core remains dependency-free and safe Rust, with 16-byte cells. A capped 1,024-entry table owns style values; old frames copy their own palettes and remain readable after engine/session destruction or ID reuse. Exhaustion retains current rendition and reports a diagnostic. Tests exercise both ordinary saturation and the case where a new rendition needs two entries but only one is available.

Supported styling covers default/indexed/RGB colors and the listed basic attributes/resets. Twelve added core tests verify independently written expected values, streaming partitions, erasure/reflow, history/hidden/saved references, bounded churn and snapshot damage. The original unsupported-control test now uses `999m` because `31m` is intentionally supported; new positive tests cover that behavior. Existing 19 fixtures still match under all 244 replay variants.

Full `scripts/verify` passes **125 Rust tests** on the macOS host, Unicode data/regeneration checks, format, warning-denied Clippy, Python/reference checks, strict owned replay, actual C/Swift callers and AppKit compilation. The configured portable Rust subset is 109 tests; hosted CI was not checked. Core tests include all 853 official Unicode 18 segmentation cases. The [native-boundary evidence](../benchmarks/results/p0-13-style-boundary/summary.json) records 110 relevant Rust tests plus C/Swift checks, raw logs, source hashes and toolchains.

The [GUI run](../benchmarks/results/p0-13-styles/window.json) carries real PTY SGR output through engine, snapshot, C ABI and AppKit. It verifies RGB foreground, indexed background, bold/italic/underline bits, matching wide-owner styles and erased-background values after resizing to 90 × 25. The [captured grid](../benchmarks/results/p0-13-styles/window.png) shows the demo's indexed colors, font traits, underline/inverse and the RGB/wide probe. Six programmatic native key events, cursor/region/reply behavior and asynchronous close after child cleanup remain covered. This is not physical keyboard latency or a renderer benchmark.

Private ABI v2 is intentionally incompatible with v1: cells are 12 bytes, frame metadata 104 bytes and copied styles 12 bytes each. Core cell size remains 16 bytes. Snapshot payload accounting includes the palette and still enforces two MiB. Row damage compares style values rather than just recycled IDs.

## Limits and follow-up

The table is a bounded cache with linear lookup and pressure-triggered scans of retained state. No style throughput measurement has been made. Capacity exhaustion is visible through the core diagnostic; the preview does not surface every protocol diagnostic in its UI. The prototype theme is fixed. Decorations and font traits use fixed cell metrics; shaping/fallback/clipping quality, IME and accessibility remain open.

No historical storage measurement was rerun or reinterpreted. The storage comparison recorder now requires equal content checksums across engines and deterministic payload sizes within each engine, because the snapshot transfer format changed independently of core storage. Its older baseline still has an eight-byte wire cell; a future current-engine comparison includes styles and is not an isolated repetition of the earlier compact-cell experiment.

Next bounded protocol work is tab stops and complete erase-line/display variants, tested through a clean shell interaction. Insertion/deletion, broader modes/replies and application compatibility follow with explicit fixtures. These do not replace the separate native text/accessibility gates.
