# Executed Ghostty reference comparison — 2026-09-26

Status: approved isolated comparison completed. No Ghostty product adoption, Rust engine implementation or performance claim.

## Results

The pinned Ghostty development reference matches **19 of 19 independent fixture expectations** with grapheme mode 2027 explicitly enabled. All **244 replays** preserve the observed state at every fixture-operation checkpoint across tested byte-delivery partitions. No VT processing error or unexpected mode change was observed. The original corpus and Alacritty evidence are unchanged.

| Configuration | Matching fixtures | Reviewed differences | Replays | Observed checkpoint equivalence |
|---|---:|---:|---:|---|
| Alacritty 0.26.0 baseline | 8 | 11 | 244 | Pass |
| Ghostty pinned reference, mode 2027 on | 19 | 0 | 244 | Pass |
| Same Ghostty reference, mode 2027 off (control) | 9 | 10 | 244 | Pass |

The control differs on all ten emoji-related fixtures, including width/ownership, erasure, edge wrap and resize. It still matches the soft-line resize expectation that Alacritty differs on. This supports the conclusion that mode-dependent engine semantics explain the grapheme results. It does not make a universal compatibility claim about either emulator or settle the product's default width mode.

Evidence: [mode-on replay](../benchmarks/results/p0-06-ghostty/replay.json), [mode-off control](../benchmarks/results/p0-06-ghostty-legacy/replay.json), [build/toolchain/dependency manifest](../benchmarks/results/p0-06-ghostty/build.json), [execution environment](../benchmarks/results/p0-06-ghostty/environment.json), [harness source hashes](../benchmarks/results/p0-06-ghostty/sources.json), [artifact checksums](../benchmarks/results/p0-06-ghostty/artifacts.json). Each report preserves raw baseline checkpoints and hashes of every replay checkpoint; mismatching delivery checkpoints would be retained in full.

## Build and method

Used exactly `6301810a48aaa3426887a4316668f18833a40138` (application source version `1.3.2-dev`, VT package metadata `0.1.0-dev`) with stable Zig 0.16.0, Xcode 27.0 and the macOS 27 SDK on arm64. Built the standalone library in ReleaseSafe, disabled XCFramework generation and left other upstream build defaults intact. Statically linked the small original C adapter using Clang with warnings denied. The final adapter's dynamic dependency listing contains libSystem only. Static third-party code remains present; this is not a dependency-free binary.

Source archive SHA-256: `9e3653be4beef36a408d64f6ad0b8abfacc2ee32ffe584f4c3d3ed20b2a5c487`. All 5,901 regular source files were checked against the archive before and after building; no upstream patch was made. The reference lives under ignored `target/`, outside the Cargo workspace. Network access was needed for upstream's pinned build packages; no global compiler or library was installed.

The [adapter method](../experiments/ghostty-reference/README.md) defines the exact state boundary and flag normalization. Read text/width/cursor/history through documented APIs, with grid references used only until the next mutation. Every replay owns and frees its terminal. No native effects or PTY are connected. Set the fixture history-line bound, but Ghostty documents page-granular limits; this experiment does not prove strict retention bounds. No throughput, latency, memory or leak measurement was performed.

Seven existing Rust harness tests plus three Python adapter tests passed under `scripts/verify` with the live adapter enabled. Tests include deliberately wrong cursor/cell expectations, an intermediate-only delivery mismatch, grapheme-mode control and invalid adapter commands. The unchanged Alacritty report still requires its eleven exact reviewed gap fingerprints. Hosted CI and sanitizers were not run for the reference.

## Reference design observations and implications

These are source-informed design lessons, not copied code or a plan to translate Ghostty line for line.

| Observed design | Implication for Nebulax |
|---|---|
| Cluster text belongs to a lead cell; wide tails are separate structural cells. The stream can extend prior content after a later feed. | Store grapheme ownership in the engine and keep decoder/segmentation state across calls. Renderer shaping cannot repair incorrect cell ownership. |
| Segmentation and width effects are separate operations; variation selectors can widen or narrow an existing cluster. | Model width changes explicitly. Update cluster, continuation cells, cursor and wrap state together. |
| The printer handles extending a cluster while wrap is pending and moving a widened cluster at the right edge. | Test attachment before consuming pending wrap. Width changes at the edge need atomic state repair and their own fixtures. |
| Page growth may invalidate cell addresses; code reacquires references after mutations. | Start with safe Rust ownership and indices. Avoid exposing grid references that survive feed/resize; defer compact pages and pointer-heavy optimizations. |
| Grapheme and cell metadata live beside row wrap information; erasure and reflow operate on engine state. | Share cluster-aware edit helpers across erase/overwrite/resize so a continuation cell cannot become orphaned. Preserve hard/soft line boundaries explicitly. |
| Grapheme behavior is a mode, separate from the terminal parser and UI. | Make width/segmentation policy explicit and test compatibility with applications before selecting a product default. |

Pinned source anchors: [stream printing and width repair](https://github.com/ghostty-org/ghostty/blob/6301810a48aaa3426887a4316668f18833a40138/src/terminal/Terminal.zig), [segmentation and width effects](https://github.com/ghostty-org/ghostty/blob/6301810a48aaa3426887a4316668f18833a40138/src/unicode/grapheme.zig), [cell/row storage](https://github.com/ghostty-org/ghostty/blob/6301810a48aaa3426887a4316668f18833a40138/src/terminal/Page.zig), [grid-reference lifetime](https://github.com/ghostty-org/ghostty/blob/6301810a48aaa3426887a4316668f18833a40138/include/ghostty/vt/grid_ref.h), [terminal API and history limits](https://github.com/ghostty-org/ghostty/blob/6301810a48aaa3426887a4316668f18833a40138/include/ghostty/vt/terminal.h).

## Dependency and license observations

The build fetched eight package archives. The manifest records their URLs, Zig package hashes, archive SHA-256 and discovered license-file hashes. These are fetched/build inputs, not eight established runtime dependencies.

| Fetched input | Observed license material / role |
|---|---|
| uucode | MIT license file; Unicode properties and segmentation support |
| translate-c | MIT license file; C translation build tooling |
| aro | MIT and Unicode license files; translation tooling dependency |
| highway | Apache-2.0 and BSD-3-Clause license files; SIMD support |
| wuffs | MIT and Apache-2.0 license files; pixel/graphics support |
| pixels | CC0 license file; build input of the wuffs package |
| zlib | zlib license file; fetched library input |
| Ghostty themes | No matching standalone license file found in this fetched archive; licensing scope remains unresolved here, and fetching does not prove it is linked |

Ghostty itself includes an MIT license. Source-vendored inputs such as simdutf and Unicode data are also part of the source archive and are not additional downloaded packages. The package cache inventory is deliberately not called a complete SBOM or redistribution audit. No third-party source or reference binary is added to tracked project paths. Any future copied/adapted source would need its own provenance/notices review; current work implements only an original API adapter.

## Limits and next step

Nineteen cases are insufficient for protocol conformance, malformed-stream handling, exhaustive Unicode, mode transitions, strict resource limits, native shaping, selection/accessibility, inactive-screen state or production performance. The reference snapshot omits styles, most modes and parser internals. Matching the provisional viewport and VS15 expectations does not turn those preferences into standards requirements.

The comparison supports the owned Rust direction in ADR 0002. The next artifact is an executable bounded Rust slice following [this concrete design](../docs/architecture/OWNED_ENGINE_SLICE.md), with independent Unicode/streaming/edit invariants. Keep the reference as evidence; add no Ghostty product dependency.
