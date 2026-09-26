# First owned Rust engine slice — 2026-09-26

Status: implemented and locally verified. This is a bounded terminal-state slice, not an installable app or complete emulator.

## Outcome

Created `nebulax-terminal` with **zero Cargo dependencies** and no unsafe code. Original Rust implements streaming UTF-8, bounded escape/control parsing, Unicode 18 grapheme segmentation, primary-grid ownership, width changes, wrapping and basic editing. Reference engines remain isolated research tooling. No terminal algorithm was copied or translated from Ghostty.

The existing independent corpus now gives **14 matching fixtures across 166 replays**, with equivalent normalized state at every tested fixture-operation checkpoint. Five fixtures are visibly pending for history, alternate-screen state and resize/reflow. The corpus and earlier Alacritty/Ghostty result sets are unchanged. Matching counts cannot be directly treated as a full-engine ranking because this Rust slice intentionally covers less scope.

Evidence: [replay](../benchmarks/results/p0-06-owned-slice/replay.json), [source hashes](../benchmarks/results/p0-06-owned-slice/sources.json), [environment and dependency tree](../benchmarks/results/p0-06-owned-slice/environment.json), [workspace dependency inventory](../benchmarks/results/p0-06-owned-slice/dependencies.json), [artifact checksums](../benchmarks/results/p0-06-owned-slice/artifacts.json). The workspace inventory includes comparison harness dependencies; the core's `cargo tree` contains only `nebulax-terminal` itself.

## Unicode choice

Used final Unicode 18.0.0 data and UAX #29 revision 49, verified against the official UCD ReadMe and the stable annex. The revision changes the Indic conjunct rule, so blindly reusing a previous version's algorithm would be incorrect. Sources and their hashes are recorded in [the data manifest](../third_party/unicode/manifest.json). See [data provenance and the dependency tradeoff](../third_party/unicode/README.md).

An offline standard-library Python generator produces 2,054 compact property ranges from versioned upstream data. Original constant-space Rust state handles segmentation; all **853 official extended-grapheme test cases** pass. Terminal column width is a separate explicit profile, tested against our independent expectations. Passing segmentation tests is not proof of full terminal Unicode/shaping compatibility.

## Verification

`scripts/verify` passes: deterministic Unicode regeneration/hash checks, formatting, Clippy with warnings denied, **22 Rust tests**, two Python reference-adapter tests (one optional live Ghostty test skipped), unchanged Alacritty known-gap replay and strict owned-slice replay. No new third-party Cargo package was resolved; only local workspace packages were added.

The core tests include all official segmentation cases, 8,000 malformed byte triples checked against Rust's lossy UTF-8 decoder, partial UTF-8 across calls, whole/split/chunked feed equivalence, width changes at the right edge, erasing/overwriting either half of a wide cluster, control boundaries, Indic/Hangul clusters, ambiguous-width configuration, cluster overflow, oversized control payloads and geometry overflow. Deterministic mixed streams check structural invariants after every byte. An ASCII-storage test confirms no per-cell text-buffer allocation for single scalars; this is not a throughput or footprint benchmark.

The default caps are 65,536 grid cells, 64 stored scalars per cluster and 64 CSI parameter/intermediate bytes. Unsupported string-control payloads are not buffered. Beyond the cluster cap, segmentation continues in bounded state while the visible cluster retains a prefix; the next boundary resumes storage. Diagnostics aggregate into flags. These implementation bounds are not a full resource/lifecycle audit.

## Limits and continuation

See [implemented policies](../crates/terminal/README.md). Standalone zero-width input is discarded with a diagnostic. Controls close the extension target; SGR and styles are unsupported. Bottom scrolling discards old rows and reports missing history. The read API uses safe borrowed views, but dirty-row tracking, text selection, accessibility, full modes/protocols, allocator-failure behavior and the Swift boundary remain unfinished. The width profile has not been tested exhaustively against all emoji sequences or applications. Hosted CI, sanitizers and performance benchmarks were not run.

Next bounded step: implement retained history with explicit limits, separate primary/alternate screen state and resize/reflow while preserving cluster ownership and hard/soft line boundaries. Activate the five deferred fixtures only when their operations exist, and add independent round-trip/content-preservation tests rather than copying reference viewport choices blindly.
