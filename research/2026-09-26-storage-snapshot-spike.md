# Storage and snapshot experiment — 2026-09-26

The storage spike is complete. Compact cells substantially reduce accounted storage for ASCII and mixed text, but the tested global cluster-slot arena loses that advantage on dense or maximum-length clusters. Row reuse and immutable snapshots with row-level damage are promising; the arena and snapshot capacity policies need refinement before product adoption. [ADR 0005](../docs/adr/0005-storage-and-snapshot-direction.md) records the consequence.

## Method and evidence

[Experiment source and method](../experiments/storage-spike/README.md), [summary](../benchmarks/results/p0-07-storage/summary.json), [all raw samples](../benchmarks/results/p0-07-storage/samples.jsonl), [environment](../benchmarks/results/p0-07-storage/environment.json), [host supplement](../benchmarks/results/p0-07-storage/host-supplement.json), [source hashes](../benchmarks/results/p0-07-storage/sources.json), [artifact hashes](../benchmarks/results/p0-07-storage/artifacts.json).

Three layouts × five synthetic scenarios × 30 measured repetitions = **450 samples**, plus 45 warmups. Runs used release Rust 1.98.1, sequential fresh processes, shuffled layout order per repetition and fixed seed 20260926. Host: Apple M2 Pro, 16 GiB RAM, macOS 27.0 arm64, AC power. CPU/RAM were read after the run because the sandbox initially denied those queries; the original metadata is preserved. Thermal telemetry was unavailable. No competing benchmark was launched, but background scheduling, frequency and thermal conditions were not controlled.

The baseline mirrors the engine's 40-byte cell and incremental cluster allocations, with a row deque and added IDs/generations. It is **not a timing measurement of `Terminal::feed`**. Compact variants have 8/16-byte cells, lazy 64-row chunks, recycled cluster slots and reserved zero-valued style fields. Both use the same arena design. No styles, hyperlink dictionaries, hashing/interning or native rendering are simulated. Sizes above the product's current history cap are synthetic storage probes, not newly supported engine configurations.

All scenarios require identical content checksums across layouts and repetitions. Checks cover steady scrolling, storage repack round trips, immutable snapshots after overwrite/reclamation/grid destruction, and reuse by row ID/generation. Six added Rust tests also compare initial synthetic content/layout to the real engine, exercise ring wraparound and arena exhaustion, and move an owned snapshot to another thread. Workspace verification passes **50 Rust tests**, Unicode/Clippy/format checks, reference adapter tests and the unchanged 19-fixture / 244-replay owned corpus.

## Accounted storage

MiB below means owned container capacity, including row/chunk/slot metadata and nested cluster buffers. It excludes allocator metadata, private Arc headers, stack/input/code/OS memory and RSS. This is capacity accounting, not a process footprint benchmark.

| Scenario | Retained cells | 40-byte baseline | Compact 8 | Compact 16 |
|---|---:|---:|---:|---:|
| Visible ASCII | 80 × 40 | 0.125 | 0.026 | 0.050 |
| Large ASCII history | 200 × 10,040 | 77.349 | 15.561 | 30.881 |
| Mixed combining/emoji | 160 × 2,040 | 13.167 | 5.161 | 7.651 |
| Dense three-scalar emoji | 160 × 1,040 | 7.711 | 6.564 | 7.834 |
| 64-scalar clusters | 80 × 168 | 3.806 | 3.888 | 3.990 |

Compact 16 saves **60.08%** for the large ASCII case and **41.89%** for mixed text, but uses **1.59% more** for dense emoji and **4.85% more** for maximum-length clusters. Compact 8 improves the first four cases but still exceeds the baseline for maximum-length clusters. The arena stores full cluster text, slot metadata and spare capacity; the baseline stores the first scalar inline. Repeated content is not interned, so these figures are not inflated by an unrealistically favorable deduplication workload.

Reclamation matters independently of the live cell count. In the dense case, after repacking and replacing all text with ASCII, Compact 16 retains 8,338,432 bytes. Trimming unused text buffers reduces this to 7,007,232 bytes, while the baseline retains 6,754,304 bytes. The free-slot directory and row chunks still occupy memory. In the long-cluster case, trimming reduces Compact 16 from 4,215,488 to 774,848 bytes, versus the baseline's 549,888. Do not adopt an indefinitely retained global slot directory as the final product design.

The large ASCII repack's maximum simultaneously live grid capacities were 156.214 MiB baseline, 31.542 MiB Compact 8 and 62.360 MiB Compact 16. This includes old/new row storage; the compact arena is counted once because IDs transfer within its owning grid. Snapshots are reported separately. Repack retains all content and has no terminal cursor/viewport policy.

## Scrolling and repack observations

Median milliseconds for each full operation batch (30 samples per cell):

| Scenario / operation | Baseline | Compact 8 | Compact 16 |
|---|---:|---:|---:|
| ASCII history: 1,000 scrolls | 1.480 | 0.464 | 0.485 |
| ASCII history: narrow/wide repack | 36.434 | 10.836 | 13.078 |
| Mixed text: 500 scrolls | 0.710 | 0.235 | 0.239 |
| Mixed text: narrow/wide repack | 8.074 | 1.603 | 1.969 |
| Dense emoji: 500 scrolls | 1.034 | 0.500 | 0.538 |
| Dense emoji: narrow/wide repack | 4.830 | 0.567 | 0.780 |

Both compact variants recorded zero container capacity growth during steady scrolling in all five fixed workloads. The baseline recorded 1,000 growths in the ASCII-history batch, 10,500 in mixed, 40,500 in dense and 80,200 in the long-cluster batch. These are instrumented container growths, not global allocator call counts. They demonstrate reuse under repeated workload shape, not allocation-free behavior for arbitrary new input.

The timing differences combine cell size, allocation/reuse, row layout and deep versus ID copying. They do not isolate the value of cell packing alone. Instrumentation adds overhead; the fixed phase order and uncontrolled host noise prevent product-speed or competitor claims. Raw minima/maxima are in the summary.

## Snapshot findings

All layouts export the same independently owned 16-byte snapshot-cell format with row-owned uncommon text. Source slots can be reused or freed without invalidating old snapshots. A previous frame lets the producer reuse immutable rows by ID/generation; repacking invalidates row identities. A test confirms 39/40 rows are shared after a single-row change, and unchanged rows remain reusable after scrolling.

For 200 × 40 ASCII cells, one packed frame occupies **128.12 KiB**; two frames with one row changed occupy **131.63 KiB**, counting shared payloads once. Native baseline deep cloning occupies 323,072 bytes (315.5 KiB). Compact 16 median extraction was 29.13 μs for a full frame, 1.27 μs for one changed row and 1.19 μs for an unchanged frame. This shows a useful incremental API shape, not renderer/frame-latency performance.

Snapshot text capacity also needs care: at the 64-scalar limit, one packed frame occupies **1,333.12 KiB**, exceeding the baseline deep clone's 937,472 bytes (915.5 KiB). Geometric growth of the flat text vectors dominates. Exact sizing or a bounded page strategy needs evaluation before production; do not infer that flattening always saves memory. Two in-flight snapshots must be charged against a combined budget, including their text, not just cell bytes. Arc control blocks/allocator overhead remain unmeasured.

## Conclusion and next step

Use **16-byte cells as the working integration candidate**, with 8-byte cells retained as a measured alternative. This preference leaves explicit space for hyperlink/attribute state; it is a feature-capacity/complexity judgment, not evidence that 16-byte cells are faster or smaller. Final style/side-data costs can change the choice. Favor lazy reusable row chunks, row identity/generation and bounded immutable snapshots. Revise side-storage reclamation and snapshot text sizing before migrating the core; do not copy this global arena wholesale.

The next bounded task is the generic VT parser/event boundary and bounded reply/effect contract. That can proceed with the current core representation while preserving the tested snapshot/ownership constraints. Then exercise these boundaries in the early native path rather than expanding a complete storage framework in isolation. The terminal core still has zero dependencies and its representation is unchanged by this experiment.
