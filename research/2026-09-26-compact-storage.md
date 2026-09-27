# Integrated compact storage — 2026-09-26

The cursor/region/input-mode checkpoint was committed and pushed as `3217fb0` before this work. [ADR 0011](../docs/adr/0011-compact-owned-cell-storage.md) records the representation and ownership policy. The core remains dependency-free and forbids unsafe code; Cargo manifests and lockfile are unchanged.

## Method

[Recorder and reproduction guide](../experiments/storage-integration/README.md), [summary](../benchmarks/results/p0-12-compact-storage/summary.json), [samples](../benchmarks/results/p0-12-compact-storage/samples.jsonl), [environment](../benchmarks/results/p0-12-compact-storage/environment.json) and [source hashes](../benchmarks/results/p0-12-compact-storage/sources.json).

The recorder rebuilds `3217fb0` plus an accounting/example-only overlay, and the current engine, with locked offline dependencies and release Rust 1.98.1. Both compile byte-identical driver source. Five workloads × two engines × ten measured repetitions produce **100 samples**, plus **20 warmups**. Fresh processes run serially in shuffled order with seed 20260926. Background scheduling, thermal state and CPU frequency are uncontrolled. No tests or other agent benchmarks ran concurrently.

Unlike the previous synthetic storage spike, these workloads call real `Terminal::feed`, resize and snapshot APIs. Capacity includes row metadata, spare cell capacity and live tail headers/buffers. It excludes allocator overhead, RSS, resize peaks and native renderer memory. Content hashes after scrolling and reflow, snapshot sizes and complete tail reclamation match across variants/repetitions. These checks supplement, rather than replace, the behavioral fixtures.

## Capacity results

Bytes below are accounted filled-grid capacity, with snapshots reported separately in the raw data.

| Workload | Previous 40-byte cells | New 16-byte cells | Reduction |
|---|---:|---:|---:|
| Visible ASCII | 129,280 | 52,480 | 59.41% |
| ASCII history | 2,731,264 | 1,103,104 | 59.61% |
| Mixed combining/emoji | 1,998,272 | 1,003,232 | 49.80% |
| Dense three-scalar emoji | 1,294,336 | 969,856 | 25.07% |
| 64-scalar clusters | 3,963,136 | 3,907,776 | 1.40% |

The new representation pays an extra boxed-vector header per live multi-scalar lead and doubles that cluster's live allocation count. It keeps ordinary cells small without a global directory. Replacement with ASCII leaves **zero cluster bytes and allocations** in all cases. Reused row vectors keep some reflow-created spare capacity: for example the new ASCII-history case ends at 1,434,496 bytes after resize/replacement rather than its initial 1,103,104. This capacity is counted; there is no claim that reuse retains exactly the initial allocation size.

## Timing observations

Median milliseconds per measured operation batch; ten samples each:

| Workload/operation | Previous | Compact |
|---|---:|---:|
| ASCII history, 200 scrolls | 0.770 | 0.718 |
| ASCII history, narrow/wide resize | 1.271 | 0.890 |
| Mixed text, narrow/wide resize | 1.069 | 1.034 |
| Dense emoji, narrow/wide resize | 0.868 | 1.152 |
| 64-scalar clusters, narrow/wide resize | 1.028 | 1.314 |

The dense and maximum-cluster resize regressions are material. Additional allocations during deep copying are a plausible explanation, but this run does not isolate their cost from layout/access differences. Snapshot medians are similar or slightly slower; its wire representation and copying approach were deliberately preserved. Treat timings as exploratory host observations, not product latency, competitor comparisons or proof of universal improvement.

## Correctness and next step

`scripts/verify` passes **113 Rust tests**, Unicode/hash/regeneration checks, formatting/Clippy, reference checks with unchanged fingerprints, all **19 owned fixtures / 244 replays**, C/Swift lifetime checks and AppKit compilation. The configured non-macOS Rust subset contains 97 tests; hosted CI was not inspected. Seven new storage tests cover direct reclamation, clone growth at every cluster length, buffer teardown/cropping, snapshots, steady row reuse and repeated edit/resize churn. Existing tests changed only to read logical cell views; their behavioral expectations remain unchanged. The old synthetic baseline's size assertion now explicitly keeps its historical 40-byte layout.

The separate [native run](../benchmarks/results/p0-12-compact-native/window.json) passes Unicode text, both cursor-key modes, origin-relative replies, region scrolling, resize/drawing and asynchronous cleanup. The [bitmap](../benchmarks/results/p0-12-compact-native/window.png) was visually inspected. Historical raw artifacts remain untouched.

Next: implement bounded style ownership and SGR against the reserved style ID, then carry styles into snapshots/native drawing. Measure style churn and retain the dense-cluster resize regression as a performance follow-up. Native IME/accessibility, broader VT behavior, production rendering and lifecycle remain separate acceptance gates.
