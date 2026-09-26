# 0004 — Successful resize with bounded, cursor-anchored retention

Status: implemented and verified 2026-09-26 within the approved Phase 0 work. Supersedes the resize-rejection policy in [ADR 0003](0003-history-screen-and-reflow.md); its history caps, buffer separation and reflow rules otherwise remain in force.

## Decision

`Terminal::resize` succeeds for every geometry supported by the configured limits. Remove `PrimaryContentWouldBeCropped`; invalid geometry still returns `InvalidGeometry` before any mutation, and an unchanged size remains a complete no-op. This does not add allocation-failure recovery or remove the existing grid cap.

Reflow the primary buffer in chronological order and map the cursor to its logical cell offset. Choose the latest viewport that still contains that cursor. Retain as many following rows as fit; preceding rows become history within the existing row/cell budgets. When the cursor and all subsequent content cannot fit together, crop only the excess suffix below that viewport. Never relocate later output into earlier history, clamp the cursor to unrelated text, or retain an unbounded hidden suffix. Growth can bring retained history back into view, but cannot recover content that was cropped or evicted.

For example, `abcdefgh` with the cursor on `a`, resized to two columns and one row, keeps `ab` and reports three cropped reflowed rows / six occupied cells. With enough height, the full text remains. This chooses cursor anchoring and bounded chronological storage over lossless shrink/grow in every case; it is an explicit Nebulax policy, not a claim that terminal standards mandate a particular viewport.

Apply the same primary policy while the alternate screen is active. The alternate screen continues physical crop/pad with a clamped cursor. Both buffers adopt the new geometry in one resize call. Clear the last retained row's soft-wrap link when its following rows are cropped; for primary reflow also remove structural padding that referred to a discarded wide owner. Whole grapheme ownership, retained printed spaces and hard boundaries remain intact.

`ResizeOutcome` sums both buffers' losses. `history_evicted` counts oldest primary rows dropped at the new reflowed width. `cropped_rows` counts primary reflowed rows or alternate old physical rows discarded below the viewport. `cropped_cells` counts occupied cells lost through cropping, including printed spaces and both halves of wide owners, excluding empty cells and structural padding. History loss is reported separately, not counted again as cropped cells. Unused trailing primary blank rows are layout slack and are not counted as content loss.

Geometry changes close the grapheme extension target but preserve partial UTF-8/CSI input. No PTY resize ordering, GUI callback, native effect, new dependency or ABI is introduced by this change.

## Evidence and remaining work

`scripts/verify` passes **44 Rust tests**, Unicode generation/conformance checks, Clippy, format, two Python tests (one optional live comparison skipped), the reviewed Alacritty baseline and all **19 owned fixtures / 244 replays** with unchanged expectations. The revised rejection test and eight additional tests cover exact crop counts, simultaneous history eviction, hidden-primary restoration, wide padding, combining clusters/printed spaces, partial input, hard-line state, repeated resize and an independent ASCII text/index oracle over **10,890 geometry/cursor/history combinations**. Mixed-stream tests now require each valid resize to succeed, rather than merely comparing two `Result`s.

Retained [replay](../../benchmarks/results/p0-06-owned-resize/replay.json), [source hashes](../../benchmarks/results/p0-06-owned-resize/sources.json) and [environment](../../benchmarks/results/p0-06-owned-resize/environment.json) identify the implementation. The original corpus and all preceding evidence remain unchanged. Native application compatibility, allocation/RSS behavior and physical-window integration still need evidence. The next foundation task is the storage/snapshot experiment in [NOW](../../plans/NOW.md).
