# ADR 0023: Immutable text-run and Core Text mapping experiment

Date: 2026-10-17. Status: implemented research contract under the approved native-backend direction; not a stable public API or production renderer.

## Decision

Keep shaping outside the headless Rust engine. First copy immutable bounded presentation runs from owned snapshots, then release the native frame/session before exercising Core Text. The Rust cell grid remains the sole authority for bytes, grapheme ownership, styles and column widths. This experiment adds no engine state, product dependency or C ABI field and does not replace the AppKit preview's existing drawing path.

A text run is a contiguous sequence of complete lead-cell owners on one row with equal full style values. Empty gaps, wrap padding, row ends and style changes break runs. Configured limits also split runs at owner boundaries. Every owner records its original UTF-8 range, run-local UTF-16 range and authoritative column span. Text is copied without normalization. Oversized owners, invalid metadata and budget exhaustion fail the complete plan rather than returning silently truncated presentation.

The research limits are 65,536 input cells / 2 MiB text; 256 owners / 4,096 UTF-8 bytes / 2,048 UTF-16 units per run; 8 MiB logically charged plan payload; and 8,192 copied glyphs / 64 resolved font-run slots per shaped input. The shaped result also has an 8 MiB logical cap, checked before allocating copied arrays. Process one native line at a time. These are owned payload/resource-count limits, not allocator, Core Text internal memory or RSS guarantees. Retaining multiple plans/results requires a caller-level bound.

For each supported Core Text run, copy glyph IDs, UTF-16 source indices, positions, advances and its actual resolved font. Derive a glyph's source interval from distinct sorted indices and the native run end, then map every intersecting input owner to its complete terminal span. Multiple glyphs may share an owner; ligatures can cover several owners; fallback can split an owner into native subruns. Retained fonts outlive the native line; borrowed Core Text arrays do not escape. Reject RTL, nonmonotonic or transformed runs until their placement policy is separately designed.

## Findings and next gate

The [experiment](../../experiments/text-shaping/README.md) verifies typographic ligatures, a local programming font, Unicode/fallback mapping, exact text preservation, invalid-input/resource failures, post-teardown shaping and offscreen clipping. [Findings and evidence](../../research/2026-10-17-core-text-mapping.md) record actual fonts and limits. Requested fonts must resolve exactly. Existing optional font files are registered only within the probe process, without changing installed fonts.

JetBrains Mono's operator ligatures changed glyph identities while the glyph count stayed constant. Consequently, neither counting glyphs nor simply grouping equal character indices captures every contextual dependency. Natural Core Text advances also do not guarantee terminal-cell alignment. The diagnostic retains these natural positions to expose that gap; this decision does not choose scaling, per-glyph snapping or per-owner clipping as a solution.

The next bounded gate is grid fitting and contextual draw-group handling: preserve engine-cell cursor/selection/hit-test geometry while accommodating proportional/fallback advances and glyph overhang. Validate style edges, ligatures enabled/disabled, wide owners and clipping before a direct Metal slice. GPU caches/atlas, scheduling, IME/accessibility, public embedding API and broad typography remain separate acceptance work.

## Sources

- [Core Text overview](https://developer.apple.com/documentation/coretext): layout objects belong to one operation/queue/thread.
- [CTRunGetStringIndices](https://developer.apple.com/documentation/coretext/ctrungetstringindices(_:_:_:)) and [CTRunGetStringRange](https://developer.apple.com/documentation/coretext/ctrungetstringrange(_:)): native source-index access.
- [CTRunStatus](https://developer.apple.com/documentation/coretext/ctrunstatus): direction/reordering/transform flags.
- [String attributes](https://developer.apple.com/documentation/coretext/string-attribute-name-constants) and the installed SDK's `CTStringAttributes.h`: ligature value 0 retains essential shaping; 2 requests available ligatures. The probe also toggles the font's `calt` feature for contextual programming alternates.

Apple documentation and installed SDK headers were consulted on 2026-10-17. These document the API; the retained run establishes only the tested behavior on the recorded host.
