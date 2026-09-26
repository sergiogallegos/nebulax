# Foundation review and implementation order

Reviewed 2026-09-26 against the owner-provided Opus architecture feedback and current source. The high-level direction remains accepted. The sequencing recommendation is sound: validate the engine's consumers and boundaries before substantially expanding protocol coverage. This revises the previous SGR-first next task; it does not implement the changes below or declare new storage/ABI choices final.

Follow-up: [ADR 0004](../adr/0004-cursor-anchored-resize.md) resolves the resize-rejection finding. The assessment below records the reviewed checkpoint; remaining work is tracked in NOW.

## Assessment

| Feedback | Finding and follow-up |
|---|---|
| Resize can fail for valid geometry, including because of hidden primary content | Confirmed. Treat `PrimaryContentWouldBeCropped` as an integration blocker. Replace it with an explicit bounded preservation/crop policy and diagnostics before PTY/window integration. Keep allocation/resource failures distinct from ordinary resize semantics. Test both buffers, cursor mapping and repeated shrink/grow. |
| Cell storage is expensive before styles | Confirmed on this arm64 build with `std::mem::size_of`: `Cell` 40 bytes, `Row` 32, `Cluster` 32. Run a measured storage spike before adding style state: compare current storage with compact cell/style IDs, bounded cluster side storage and chunked/recycled rows. Include allocation, reclamation, reflow and snapshot costs. An 8–16-byte cell is a candidate, not an automatic requirement or proof of lower total memory. |
| Parser mixes recognition with meaning | Confirmed: one `u16` CSI parameter and semantic actions cannot represent upcoming multi-parameter/subparameter protocols. Separate bounded syntax events from terminal operations. Specify empty/default parameters, intermediates, private prefixes, cancellation and overflow recovery. Add bounded string capture only for supported commands, with explicit overflow/discard behavior. A generic parser need not claim every VT protocol is implemented. |
| No reply/effect channel | Confirmed: `FeedOutcome` only carries flags. Design bounded typed replies/effects with ordering, capacity and backpressure semantics; no silent reply loss or reentrant native callbacks. Runtime policy mediates native effects, and PTY data never gains config/control authority. |
| Cursor, regions and modes need a coherent model | Agree. Define scroll margins, origin mode, saved cursor, tab stops and input-relevant modes together. Specify which state belongs to a buffer versus the terminal. Keep fullscreen history retention distinct from region scrolling. Implement the subset needed for integration, then widen coverage. |
| Borrowed grid and one changed flag are insufficient for rendering | Agree. Borrowed views are valid internal APIs but are not the planned cross-thread frame contract. Prototype bounded immutable snapshots, row identity/generations and changed-row extraction, including side-data ownership and in-flight lifetimes. Shaping and presentation must not hold the engine lock. |
| `Terminal` owns too much editing logic | Agree. Move grid edits and ownership invariants behind `Screen`/grid methods while keeping parsing, terminal semantics and orchestration separate. Preserve behavior tests through that refactor. |
| ASCII and row reuse | Confirmed opportunities: scalar processing performs property lookups and scrolling allocates blank rows. Controls/escape bytes do not all take the Unicode lookup path. Measure throughput before declaring a bottleneck; structure for ASCII runs and row reuse without breaking streaming boundaries. |
| Unsafe FFI exception and grid limits | Agree. Workspace lints are opt-in through crate inheritance; keep `forbid` in the safe core, and give a future narrow FFI crate its own audited unsafe policy. The 65,536-cell grid cap rejects 500×170; determine supported geometry and total byte budgets from native integration and storage measurements. Do not simply remove limits. |

Two qualifications matter. The review's 10,000×200×40 calculation is correctly 80 MB of cell storage, but its “100–1000×” comparison is overstated for that example: 2,000,000 cells is about 30.5× the current 65,536-cell history cap (10,000 rows at 80 columns is about 12.2×). Also, independent behavior fixtures protect refactoring; representation-specific tests may need changes, but adding valid behavioral evidence does not itself lock in storage.

## Revised order and exit evidence

1. **Foundations:** settle and test ordinary resize success; measure storage alternatives; define generic bounded parsing, typed effects/replies, state ownership and snapshot/damage contracts. Use small executable spikes and preserve the existing replay corpus. Do not build a complete framework in isolation.
2. **First integrated path:** one PTY → engine → bounded snapshot → minimal AppKit/Core Text/Metal window, with basic input back to the PTY. Include just enough cursor/erase/SGR support to exercise it. Validate resize ordering, FFI lifetime/teardown, thread ownership, backpressure and early IME/accessibility text boundaries. This is a feasibility gate, not a finished terminal.
3. **Coverage and hardening:** expand modes, regions, styles, edits and queries against independent fixtures; add isolated fuzzing around parser/state invariants. Profile ASCII, Unicode, scrolling and rendered workloads before optimizing further.

Keep one current task source in [NOW](../../plans/NOW.md); [HANDOFF](../../HANDOFF.md) records history/owner context and [STATUS](STATUS.md) records accepted versus open architecture. Avoid copying this task sequence into each. GitHub Issues have not been created. Original review documents and recorded evidence remain unchanged.
