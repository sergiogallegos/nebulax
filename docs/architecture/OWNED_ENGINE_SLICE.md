# First owned Rust engine slice

Status: first bounded slice implemented on 2026-09-26. See [findings](../../research/2026-09-26-owned-engine-slice.md) and [implemented policies](../../crates/terminal/README.md). The design below is retained as context; the core now uses generated Unicode 18 tables, owns its segmentation rules and has zero Cargo dependencies. `FeedOutcome` currently reports a whole-screen changed flag rather than dirty-row tracking. The subsequent history/alternate/resize slice is now implemented; see [ADR 0003](../adr/0003-history-screen-and-reflow.md) and [current findings](../../research/2026-09-26-history-screen-reflow.md). References to those features as future work below describe the original first-slice scope.

## Scope and ownership

Create one small `nebulax-terminal` Rust crate when implementation starts. It owns decoding, the supported parser subset, cell state and editing. Use synchronous `feed(&[u8])` calls; the engine needs neither Tokio nor a scheduler. Keep the existing serialization/report crates in the research runner. No Ghostty, Alacritty, C/Zig bridge, GUI or PTY dependency belongs in this crate.

First milestone: fixed-size primary screen, streaming UTF-8, printable graphemes, CR/LF/BS, CSI cursor-left, erase-to-end-of-line and erase-N-cells, plus pending wrap. Extend to history, primary/alternate switching and resize/reflow as the next bounded slice. The initial slice must explicitly report unsupported features in its test adapter; it cannot claim all 19 fixtures pass by ignoring resize or screen switching.

## Proposed representation

| Type | Responsibility |
|---|---|
| `Terminal` | Own parser, screen, cursor, limits and explicit width policy; sole mutation boundary |
| `Decoder` | Hold up to three incomplete UTF-8 bytes between feeds; malformed-sequence recovery independent of chunking |
| `Parser` | Bounded state machine for the supported controls; cap parameters and discard unsupported string controls without buffering their entire payload |
| `Screen` | Rows with exactly `columns` cells; row-level hard/soft boundary metadata |
| `Cell` | `Empty`, `Lead { cluster, width }`, `Continuation`, or right-edge wrap padding; a continuation always belongs to the preceding width-two lead |
| `Cluster` | Owned scalar sequence plus enough segmentation context to extend incrementally; keep single scalars inline and use bounded extra storage only when needed |
| `Cursor` | Row, column and separate pending-wrap bit; no out-of-bounds column as a hidden sentinel |
| `WidthPolicy` | Explicit ambiguous/emoji/text-presentation choices, independent of Unicode segmentation |

Begin with straightforward safe Rust storage and borrowed read views scoped to `&self`. A mutable feed or resize cannot coexist with those views. Use indices during mutation, and reacquire cells after row movement. Defer arenas, packed bitfields, SIMD, page compression and renderer damage optimization until evidence justifies their complexity. Do not implement a second authoritative grid.

Suggested API shape (not a promised public ABI):

```rust
Terminal::new(size: Size, limits: Limits, policy: WidthPolicy) -> Result<Terminal, Error>
Terminal::feed(&mut self, bytes: &[u8]) -> FeedOutcome
Terminal::screen(&self) -> ScreenView<'_>
Terminal::cursor(&self) -> Cursor
```

`FeedOutcome` reports bounded diagnostics and dirty rows; it does not execute native effects or return an unbounded event list. Do not finalize the Swift ABI around this research representation.

## Incremental text and editing rules

1. Decode and parse incrementally. Ending a `feed` is never a Unicode or escape-sequence boundary. Incomplete UTF-8 waits for later bytes; an explicit end-of-stream action is separate.
2. On a printable scalar, determine whether it extends eligible prior cluster content before consuming pending wrap. Segmentation answers ownership; width policy answers occupied columns. A later positive-width scalar can still extend the same cluster.
3. For extension, compute the proposed new width, preflight storage/geometry, then update the lead, tail, cursor and pending-wrap state as one mutation. If widening at the edge requires a move, move the whole cluster and retain soft-wrap metadata. Narrowing releases the old tail.
4. For a new cluster, resolve pending wrap, clear any intersected old cluster and place a lead with its continuation when needed. A wide cluster that cannot fit at the last column moves intact to the next row.
5. Share a helper that expands an edit range to include any intersected wide owner. Erase or overwrite either half without leaving orphaned tails or detached cluster text. BS remains column movement, not Unicode-scalar deletion.
6. Cursor/edit controls must invalidate or resolve the prior extension target explicitly. Add cases for marks after cursor movement and style/control boundaries before generalizing this behavior; Ghostty's choice is reference evidence, not an implicit specification.

Use a small configurable cluster-scalar cap and fixed parser limits from the outset. A starting research cap of 64 scalars is a proposed bound, not a Unicode maximum. Define deterministic overflow behavior and test that discarded suffixes cannot cause unbounded allocation, diagnostics or work across reads. Geometry multiplication must be checked and bounded. A later scrollback limit must be exact for the selected policy rather than assumed from another engine's page-granular behavior.

## Unicode data and dependency decision

Implement engine/state/editing logic ourselves. Unicode properties are versioned data, not a handful of emoji-specific cases. Before coding segmentation, verify the current stable Unicode release and compare two narrow options: generated, checked-in property tables plus an owned rule implementation, or a focused stable Unicode crate. Prefer generated data if the generator, conformance suite and update process remain small and maintainable. Record data URLs, checksums and notices. Implementation follow-up selected official Unicode 18.0.0 data with an offline generator and original Rust rules; see [provenance](../../third_party/unicode/README.md).

Whatever the implementation choice, validate segmentation with that release's official grapheme-break tests and width behavior with separately documented terminal expectations. Never hardcode only the 19-fixture codepoints. A targeted dependency is acceptable under ADR 0002 if it substantially lowers correctness/maintenance costs; evaluate its transitive footprint before adding it. The goal is justified minimal dependencies, not an untested claim of zero-dependency completeness.

## Completion criteria

- All in-scope existing fixtures pass their independent expectations under the same byte partitions. Out-of-scope fixtures remain visibly pending until their operations exist.
- Add independent cases for malformed and incomplete UTF-8, cluster-width changes at the last column, partial wide-cell erasure/overwrite, cursor/control boundaries, long combining runs and limit overflow. Compare every operation checkpoint, including cursor and cell ownership.
- Assert grid invariants after each operation: row length, valid cursor, lead/tail consistency, bounded clusters and no detached text in continuation cells. Repeated replay must be deterministic.
- Pass official segmentation tests for the selected data version; keep terminal-width policy tests separate. Verify ASCII paths do not pay an allocation per cell before optimizing layout.
- Preserve the Alacritty and Ghostty baseline artifacts. Keep the Rust product dependency graph separate from reference tooling and record any new dependency rationale.

Then implement history, screen switching and resize against the remaining fixtures, with tests that preserve logical content and hard/soft boundaries. Passing this slice still leaves protocol coverage, resource accounting, lifecycle/FFI, rendering, IME and accessibility work open.
