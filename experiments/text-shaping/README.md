# Immutable text runs and Core Text mapping

This macOS research experiment consumes a real owned Rust snapshot through private ABI v3. It copies bounded presentation runs, releases the frame and session, and then shapes using system Core Text. It does not change the engine, preview, private ABI or dependencies. No window server is needed.

```sh
scripts/text-shaping
scripts/text-shaping --output target/text-shaping/my-run
scripts/text-shaping --programming-font JetBrainsMono-Regular \
  --programming-font-file /path/to/existing/JetBrainsMono-Regular.ttf
```

The baseline requires exact system Menlo-Regular and Times-Roman faces. The optional programming-font check requires the exact PostScript name and changed operator glyph identities with ligatures enabled. An existing font file may be registered in the probe process only; no font is installed, downloaded, redistributed or globally activated. Failure to resolve a requested font fails the experiment rather than silently substituting it. Font names, versions and source-file hashes accompany the results. `scripts/verify` executes the baseline on macOS and explicitly skips it elsewhere. The installed programming font is local research evidence, not a CI or product dependency.

## Contract and coverage

`TextPlan.swift` copies row/style-separated lead-cell text and ownership from a borrowed frame. Empty cells, wrap padding, row ends and full style changes break runs; wide continuations belong to their lead. Owner UTF-8 source ranges and run-local UTF-16 ranges are retained alongside engine columns/widths. Source offsets are provenance, never pointers into a released frame. The plan has immutable value fields and owns its text. It cannot feed bytes, update cells or authorize effects. No terminal lock is held while shaping.

Default limits are 65,536 frame cells, 2 MiB source text, 256 owners / 4,096 UTF-8 bytes / 2,048 UTF-16 units per run, and 8 MiB charged plan payload. Split at owner boundaries; reject an individually oversized owner or a plan overflow without returning a partial plan. This is a research format, not a public SDK contract. Core Text output is checked before allocating copied arrays: at most 8,192 glyphs and 64 font-run slots per shaped input. Shaped output is also capped by the 8 MiB logical budget. Only one run is shaped at a time; callers must bound their own number of retained outputs.

`Shaper.swift` retains glyph IDs, resolved fonts, natural positions/advances and mappings to complete engine-owner spans. Multiple glyphs can map to one owner and one ligature can map to multiple owners. Fallback runs may split a combining/emoji owner. Character indices are UTF-16 offsets; neither Swift `Character` segmentation nor glyph count decides cell width. RTL, nonmonotonic and transformed runs are explicitly rejected by this first contract. This does not establish a bidi policy.

The synthetic PTY fixture exercises operator/prose ligatures on/off, combining accents, CJK, skin-tone ZWJ emoji, flags, keycaps, a 21-scalar cluster, style boundaries, empty gaps and wide-edge wrap padding. Checks cover exact UTF-8 preservation through UTF-16 conversion, independently expected columns/widths, every owner having glyph coverage, actual fallback, invalid wire metadata/text/continuations, run splitting and exact payload/glyph/font limits. The source frame/session are released before shaping, followed by 64 discard-and-repeat cycles. A separate pixel check confines all changed pixels to a one-cell clip rectangle.

## Artifacts and limits

Each new output directory contains input owners/styles, glyph mappings, a summary, source hashes, command logs, environment/binary hashes and an offscreen PNG. Existing directories are rejected. Source stability is checked. Failed attempts remain under ignored `target/`; retained successful evidence lives under `benchmarks/results/`.

Payload counts use native record strides plus explicit UTF-8/UTF-16 charges. They exclude allocator overhead, temporary/Core Text allocations, font backing stores, font caches and process RSS. Font hashing uses bounded streaming buffers. Repeated shaping is a correctness check, not a leak detector or throughput benchmark. No GPU atlas, cache, frame scheduler or native IME/accessibility acceptance is claimed.

The PNG deliberately draws Core Text's natural positions inside terminal-run rectangles. It exposes spacing drift for proportional/fallback fonts. A contextual programming font can change glyph identities without changing glyph count; string-index spans alone do not identify all contextual drawing dependencies. Grid fitting, overhang/group clipping and cursor/selection rendering through a ligature need a separate experiment before integrating Metal. See [ADR 0023](../../docs/adr/0023-immutable-text-runs-and-core-text-mapping.md).
