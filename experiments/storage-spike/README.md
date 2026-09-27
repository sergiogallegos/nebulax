# Storage and snapshot experiment

An isolated safe-Rust storage comparison, not a product engine, secondary authoritative grid, rendering backend or public ABI. No new third-party dependency is introduced: the research package reuses the workspace's pinned `serde_json` and the dependency-free core for layout/content checks.

```sh
cargo test -p nebulax-storage-spike --locked
scripts/storage-spike
scripts/storage-spike --samples 1 --warmups 0  # smoke run, not retained timing evidence
```

The recorder builds in release mode, rejects existing output directories and records source/binary hashes, the lockfile hash through the source manifest, host/toolchain/power metadata and raw samples. Defaults: three warmups and 30 measured repetitions for each scenario/variant, one fresh process at a time. Candidate order is shuffled within each scenario/repetition with seed 20260926. Scenario/operation phase order is fixed. Timings are exploratory host-local samples, not controlled product/competitor throughput, RSS, energy or latency evidence.

## Compared layouts

- Baseline: preserves the original 40-byte `Cell` and nested `Cluster`, with incremental `Vec<char>` growth for extra scalars. It uses a row deque with IDs/generations, so its row metadata/scroll machinery differs from the original engine. It has no style storage; this conservative comparison does not predict the cost of adding styles to it.
- Compact 8: a tagged 32-bit scalar/cluster-slot payload and 32-bit style ID.
- Compact 16: the same payload/style ID plus 32-bit hyperlink and attribute fields. Style/hyperlink/attribute fields are reserved and zero; dictionaries, interning and style churn are **not** implemented or measured.
- Both compact variants use lazily allocated chunks of up to 64 rows and a bounded cluster slot arena. Each live multi-scalar owner has one slot, with no deduplication. Scroll/overwrite releases slots; their text buffers remain available for reuse. An explicit trim frees unused text capacity but retains the slot directory and row chunks. Arena exhaustion asserts in this test prototype; production would need a typed resource policy and byte budgeting.

The experiment enforces source clusters of at most 64 scalars and an initial grid limit of four million cells. Tests verify the baseline layout and synthetic content against the actual engine, repeated wraparound reuse, side-slot bounds, cross-layout repacking, snapshot damage identity and snapshots surviving reclamation/grid destruction/thread transfer. The experiment supports no arbitrary PTY input.

## Workloads and measurements

| Scenario | Columns × retained rows | Steady scrolling |
|---|---:|---:|
| Visible ASCII | 80 × 40 | 200 rows |
| Large ASCII history | 200 × 10,040 | 1,000 rows |
| Mixed text, combining and emoji | 160 × 2,040 | 500 rows |
| Dense three-scalar emoji | 160 × 1,040 | 500 rows |
| 64-scalar clusters | 80 × 168 | 200 rows |

Every run measures fill, full-capacity scrolling, full/unchanged/one-dirty-row visible snapshots, a narrow/wide repack round trip and trimming free side text. Content checks run outside timed regions. Repacking preserves every retained owner and hard break; it is a storage experiment, **not** the terminal's cursor-anchored resize algorithm. It transfers compact cluster IDs within the same owning grid and counts old/new row storage simultaneously; it does not expose those IDs to snapshots. Repack scratch row capacity may exceed the original row limit and is retained, explicitly included in the reported capacity.

Snapshots copy visible cells to a common 16-byte cell format and flatten uncommon text into row-owned buffers. Immutable rows are held by `Arc`; unchanged rows are shared by ID/generation. Measurements cover up to two in-flight snapshots, counting shared row payloads once. Reflow gives rows fresh identities, invalidating the cache. No engine borrow/lock or cluster-arena reference survives capture. The baseline also measures native visible-cell deep cloning, including destruction, separately from packed snapshot extraction.

`capacity_bytes` accounts for owned `Vec`/`VecDeque` capacities, nested text allocations, row/chunk/slot directories, and snapshot row payloads. It excludes allocator metadata/size classes, private `Arc` headers, stack objects, executable/framework pages, inputs, OS overhead and RSS. `container_growths` counts observed instrumented capacity increases (plus `Arc` row payload creations); `requested_capacity_bytes` sums their new capacities, not resident bytes or global allocator events. Repacking's grid peak excludes separately reported in-flight snapshots. Instrumentation runs for all variants and adds overhead to these timings. Identical checksums and capacity/accounting results are required across repetitions and variants where applicable.

This spike establishes tradeoffs and API/lifetime constraints. It does not authorize removing resource caps or importing this prototype wholesale into the terminal core.

The later [core storage integration](../storage-integration/README.md) uses a different 16-byte directly owned cell, with no global arena. This experiment and its earlier raw results remain historical comparisons.
