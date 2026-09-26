# Owned engine replay

This research-only Rust adapter depends on `nebulax-terminal`, `serde_json` and `sha2`. Serialization/hashing remain outside the dependency-free engine. It consumes the unchanged 19-fixture corpus used for the reference comparisons.

```sh
scripts/replay --engine owned
scripts/replay --engine owned --output target/replay/owned-review
cargo tree --locked -p nebulax-terminal
```

All 19 fixtures are executed through 244 replays: whole delivery, repeated whole, chunks of 1/2/3/7 bytes and every interior two-part split index. Each fixture-operation checkpoint captures geometry, visible/history text, active-screen identity, width/continuation roles, row wrap, cursor and pending wrap. Known-gap allowances from the Alacritty fixture metadata are not used. Any expectation or delivery mismatch fails; unsupported/limit/orphan/history-loss diagnostics also fail. Fixture resizes must succeed without eviction or cropping. Every feed must consume all input and emit no output events; unexpected replies/effects or backpressure fail explicitly. Parser/output protocols are tested separately in the core. The wrapper rejects `--allow-known-gaps` for the owned engine.

History, alternate-screen restoration and resize operations now execute with no deferred fixtures. Exit 0 means the existing corpus passed, not full emulator conformance. A regression test locks the 19 matched / zero pending counts; adding unsupported operations fails rather than silently skipping them.

The adapter maps equivalent semantics to the existing expected flags (32 lead-wide, 64 continuation, 1024 edge padding, 16 last-cell soft wrap). No renderer or second authoritative grid is involved. The report preserves baseline operation checkpoints and per-delivery checkpoint-sequence hashes. Separate core integration tests compare full cloned engine state, including decoder/parser state, under arbitrary feed cuts and bytewise delivery. No performance measurements are made.

The shared recorder includes core source, generated tables, original Unicode inputs, test sources, lockfile, toolchains and executable hashes. Existing reference evidence remains immutable. The latest [parser/output regression evidence](../../benchmarks/results/p0-06-owned-parser-output/replay.json) preserves the same grid results; [ADR 0006](../../docs/adr/0006-bounded-parser-and-output.md) records separate output coverage. Earlier results and limits are in the [history/resize findings](../../research/2026-09-26-history-screen-reflow.md). The [first-slice report](../../research/2026-09-26-owned-engine-slice.md) preserves its earlier 14-match result.
