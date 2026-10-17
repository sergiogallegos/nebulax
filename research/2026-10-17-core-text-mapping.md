# Core Text mapping and immutable text runs — 2026-10-17

[ADR 0023](../docs/adr/0023-immutable-text-runs-and-core-text-mapping.md) records a bounded research contract between owned terminal snapshots and native shaping. The Rust engine and ABI v3 are unchanged. Swift copies text runs with byte/UTF-16/cell ownership, releases the PTY session and source frame, and shapes solely from that owned presentation input.

## Observed behavior

The synthetic 80×12 terminal produced 15 text runs. Menlo/Times tests exercise combining marks, CJK, skin-tone ZWJ emoji, regional-indicator flags, keycaps, a 21-scalar cluster, style changes, empty gaps and a wide owner wrapping past edge padding. Every tested owner retains its exact original UTF-8 bytes and engine-provided columns/widths through UTF-16 conversion and fallback. No fixture glyph is missing. The mixed Hebrew line is rejected in both modes because RTL placement is outside this initial contract.

Times-Roman changes the prose fixture from 16 glyphs with optional ligatures disabled to 12 enabled. Multi-cell ligatures retain their complete logical owner spans. The optional JetBrainsMono-Regular check changes operator glyph identities while retaining 18 glyphs in each mode. This is direct evidence that glyph-count reduction is not a sufficient ligature test. The local font is version 2.304; exact source hashes for it and all resolved fallback fonts are retained. The optional file is registered in the probe process only; no new font or dependency is installed or redistributed.

The [offscreen diagnostic](../benchmarks/results/p0-22-core-text-mapping/shaping.png) shows natural Core Text positions inside terminal-run rectangles. JetBrains operators change visibly, while proportional Times and fallback text do not naturally align to fixed columns. String-index coverage is useful semantic metadata, but contextual alternates can involve neighboring glyphs without reducing their count. It does not establish independent per-owner draw groups. Grid fitting, contextual overhang and cursor/selection clipping remain the next bounded experiment; no direct Metal renderer is implemented here.

## Bounds and lifetime evidence

The copied plan charges **8,350 bytes**; the largest copied shaped run charges **6,328 bytes** on this host. These are explicit text/UTF-16/native-record payload charges, not RSS or total Core Text/font memory. Input plan defaults cap 65,536 cells / 2 MiB text and 8 MiB charged payload, with per-run owner/byte/unit limits. Shaping checks its 8,192-glyph / 64-font-slot limits and logical byte budget before allocating copied arrays. It retains resolved fonts with glyph IDs, but no native CTLine or borrowed native array.

The source session and frame are released before shaping. Checks then verify complete owner coverage and 64 repeated independent shapes after teardown. This establishes ownership independence and stable output, not absence of framework caching or memory leaks. Malformed wire counts, bounds, UTF-8, style references, lead/continuation structure and plan exhaustion fail without returning partial presentation. Configured run splits preserve complete owners; exact plan/shaped-payload limits accept, and one byte below rejects. Glyph/font limits reject explicitly.

An independent one-cell raster clip changes 123 pixels inside the allowed region and zero outside it. This verifies the tested offscreen clipping path, not final GPU geometry, font quality or native selection behavior. No performance comparison, deployment-floor claim or new protocol compatibility claim is made.

## Validation and retained evidence

[Retained run](../benchmarks/results/p0-22-core-text-mapping/summary.json) includes the input owner/style map, copied glyph output, resolved-font versions/hashes, source hashes, environment, native binary hashes, command logs and PNG. The optional installed-font run supplements the system-font baseline in `scripts/verify`; it is not a new CI prerequisite. Source stability is checked during recording and existing evidence directories are never overwritten.

Full `scripts/verify` passes: 202 Rust tests, Unicode data/regeneration, format/Clippy, SDK ABI and owned-dependency guards, strict 19-fixture / 244-replay owned acceptance, the existing reviewed Alacritty gap fingerprints, C/Swift boundary execution, AppKit preview compilation, this new system-font shaping baseline and seven Python cases (six pass, one optional live-Ghostty skip). The optional-font recording passes 1,035 contract assertions. The baseline is headless on macOS and explicitly skipped elsewhere; no new GUI run was needed because the existing preview drawing path is unchanged. Native IME/accessibility, clipboard/menu integration, production scheduling, glyph caches, Metal and public embedding API stability remain separate work.

The owner also raised [persistent/shared sessions and remote/web/iOS clients](../docs/architecture/PERSISTENT_SESSIONS.md). That proposal is recorded as future staged planning; this experiment adds no daemon, transport or remote authority and leaves daemon-free v1 in force.
