# Third-party dependencies

Original Nebulax code is MIT licensed. Dependencies keep their upstream licenses; the project license does not relicense them. No upstream terminal implementation code has been copied into Nebulax source in this experiment.

| Direct dependency | Pinned version | Registry-declared license | Purpose |
|---|---|---|---|
| alacritty_terminal | 0.26.0 | Apache-2.0 | Isolated Alacritty research baseline |
| serde | 1.0.229 | MIT OR Apache-2.0 | Fixture/report serialization |
| serde_json | 1.0.151 | MIT OR Apache-2.0 | JSON corpus and report |
| libc | 0.2.189 | MIT OR Apache-2.0 | macOS PTY OS ABI bindings; already in the workspace lockfile |
| sha2 | 0.11.0 | MIT OR Apache-2.0 | Fixture and observed-state fingerprints |

`Cargo.lock` records resolved versions and registry checksums. `cargo metadata --locked --format-version 1` exposes package license metadata and manifest locations; `scripts/replay` also saves a dependency version/checksum inventory. Inspect exact package license/NOTICE files and include required texts before shipping any binary. Registry metadata alone is not a completed distribution-license audit. The current repository does not vendor dependency source or ship an application artifact.

The approved Ghostty reference is downloaded/built only under ignored `target/`. No Ghostty source or binary is distributed in project artifacts. Its pinned revision, fetched-package hashes and license-file inventory are recorded in the [comparison](research/2026-09-26-ghostty-reference-comparison.md) and [build manifest](benchmarks/results/p0-06-ghostty/build.json). That inventory is not a complete redistribution audit. The C API adapter is original Nebulax code; no terminal algorithm has been copied or translated into product code.

The owned `nebulax-terminal` crate has no Cargo dependencies. It includes generated Unicode 18 property data from the unmodified inputs in [third_party/unicode](third_party/unicode/README.md), under [Unicode License V3](third_party/unicode/LICENSE.txt). Its original implementation is MIT; its package license expression is `MIT AND Unicode-3.0`. The official grapheme test data retains the same attribution. These are data/test assets, not copied third-party engine algorithms.
