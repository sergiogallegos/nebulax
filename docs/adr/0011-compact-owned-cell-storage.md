# 0011 — Compact cells with directly owned cluster tails

Status: implemented and measured on 2026-09-26. Refines the integration candidate from ADR 0005; no protocol, native ABI or dependency change.

## Decision

Use a 16-byte `Cell` on the verified 64-bit targets. It holds an inline first scalar, optional thin owning pointer to a boxed tail vector, a reserved 16-bit style ID, kind and width. Only multi-scalar leads allocate. A borrowed `CellView`/`Cluster` exposes logical content without exposing storage fields or owning handles. Style zero is the only current value; no style dictionary or SGR behavior is introduced.

The boxed vector is intentional: its 24-byte header leaves ordinary cells and exists only for a live multi-scalar cluster. Its scalar buffer holds at most 63 additional scalars. Growth uses explicit bounded capacity, including after deep clones whose exact capacities are not powers of two. The configured cluster limit still applies before mutation. Cloning owns independent tails; clearing, overwrite, eviction, alternate-screen teardown and cropping drop their allocations immediately.

Do not introduce the spike's global cluster directory, interning, free-slot cache or shared arena lifetimes. This avoids retained metadata after all clusters disappear and keeps safe Rust ownership local. The tradeoff is two live heap allocations per multi-scalar cluster (header and scalar buffer), versus the old layout's one. Allocator overhead is not included in the capacity result.

Keep independently owned row vectors for this slice, reusing evicted/rotated buffers on steady full-history scrolling, no-history/partial-region scrolling and reverse index. Reflow/crop still create bounded replacement rows. Chunked row storage and row sharing remain separate optimizations; neither is needed to get compact cells or reclamation. Reused rows can retain the spare capacity acquired during reflow, and accounting includes that spare capacity. No geometry/history cap is increased.

Owned snapshots keep their existing exactly sized cell/text arrays, two-MiB payload bound and independent lifetime. No cell pointer, tail allocation or arena identifier reaches the C bridge. Snapshot generation, row-version comparison and the eight-byte wire cell remain unchanged. This slice does not implement shared snapshot rows, edit-time damage tracking or a final style-bearing ABI.

`storage_usage()` accounts for row-container capacity, cell-vector capacity, live boxed headers and scalar-buffer capacity across active/hidden buffers. It excludes parser/output queues, resize temporaries, allocator overhead and RSS. This is inspection support for resource tests and measurements, not a new allocator or an OOM recovery guarantee.

## Evidence and tradeoff

All original logical expectations remain intact; read adapters now call `Cell::view()`. Seven added tests cover size/style reservation, immediate reclamation, every cloned tail length, resize/buffer teardown, snapshot lifetime, row-buffer reuse and bounded churn. Full verification passes 113 Rust tests and the unchanged 19 fixtures / 244 replays; native C/Swift and the AppKit regression pass.

The [integrated comparison](../../research/2026-09-26-compact-storage.md) rebuilds the exact previous checkpoint with an accounting-only overlay and uses the same driver for both engines. Across 100 samples plus 20 warmups, accounted filled-grid capacity decreases by 59.61% for ASCII history, 49.80% for mixed text, 25.07% for dense emoji and 1.40% for maximum-length clusters. After replacement with ASCII, both engines retain zero cluster capacity.

Dense/long-cluster resize medians were approximately 33%/28% slower in this exploratory run; the two-allocation deep-clone path is a plausible contributor, not an isolated causal measurement. This is a memory/ownership decision with a measured cost, not a universal throughput improvement. Preserve the comparison harness for future ownership/move-based resize work. Styles must use bounded ownership/reclamation and be measured before expanding the reserved ID into a dictionary.
