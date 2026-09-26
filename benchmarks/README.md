# Experiment artifacts and future benchmarks

An isolated synthetic storage/capacity experiment has run; no product/competitor throughput, RSS, energy, startup or physical latency benchmark has been run.

The first [headless correctness report](results/p0-06-initial/replay.json) is stored with [environment metadata](results/p0-06-initial/environment.json), [source hashes](results/p0-06-initial/sources.json), [dependency checksums](results/p0-06-initial/dependencies.json) and [artifact hashes](results/p0-06-initial/artifacts.json). The synthetic input and expected states live in `tests/fixtures/terminal-replay.json`.

The [expanded grapheme report](results/p0-06-graphemes/replay.json) retains the 19-fixture / 244-replay follow-up with its own [environment](results/p0-06-graphemes/environment.json) and [artifact hashes](results/p0-06-graphemes/artifacts.json). Original evidence is unchanged.

Run `scripts/replay` for fresh evidence in `target/replay/`. Timestamps, executable hashes and Git dirty state may differ; normalized replay JSON should be identical for the same source/dependencies. The source manifest identifies dirty/untracked code; it does not claim the initial commit contains the harness.

Follow [Phase 0 methodology](../review/PHASE_0_PLAN.md) before performance work. Serialize controlled runs, distinguish parsing from presentation, and never call software timing physical key-to-photon latency. Small sanitized evidence can be committed; large traces require checksummed external artifacts.

The [Ghostty mode-on reference](results/p0-06-ghostty/replay.json) matches 19/19 expectations across 244 replays. Its [mode-off control](results/p0-06-ghostty-legacy/replay.json) matches 9/19 across another 244. Both retain build manifests, source hashes and raw operation checkpoints. See the [method and limits](../experiments/ghostty-reference/README.md); these comparisons introduce no product engine dependency.

The [owned Rust slice](results/p0-06-owned-slice/replay.json) records 14 matching fixtures / 166 replays and five visibly pending fixtures. Its environment records the empty core dependency tree, while the workspace inventory still includes research-only comparison dependencies.

The [owned history/resize follow-up](results/p0-06-owned-history/replay.json) records all 19 matching fixtures / 244 replays, with zero deferrals and unchanged expectations. Its [source manifest](results/p0-06-owned-history/sources.json) identifies that implementation; the preceding owned-slice result remains historical evidence. See [findings and limits](../research/2026-09-26-history-screen-reflow.md).

The [successful-resize follow-up](results/p0-06-owned-resize/replay.json) preserves all 19 matches / 244 replays with unchanged expectations. Its [source manifest](results/p0-06-owned-resize/sources.json) includes the revised crop policy and regression tests; [ADR 0004](../docs/adr/0004-cursor-anchored-resize.md) explains the evidence and tradeoff.

The [storage/snapshot experiment](results/p0-07-storage/summary.json) records 450 measured samples plus 45 warmups for three layouts and five workloads. [Raw samples](results/p0-07-storage/samples.jsonl), [environment](results/p0-07-storage/environment.json) and [findings](../research/2026-09-26-storage-snapshot-spike.md) distinguish owned-capacity accounting and exploratory operation timings from process/product performance. Run `scripts/storage-spike` serially; it is deliberately outside routine CI verification.

The [parser/output follow-up](results/p0-06-owned-parser-output/replay.json) preserves all 19 grid matches / 244 replays and unchanged expectations. Its source manifest includes twelve separate parser/output tests; the existing replay corpus does not itself exercise the new reply/effect protocols. [ADR 0006](../docs/adr/0006-bounded-parser-and-output.md) records those tests and limits.

The [PTY lifecycle run](results/p0-08-pty-session/summary.json) retains eleven tests (six real macOS lifecycle/cleanup and five portable transport tests), raw logs, source/executable hashes and dependency trees. [Findings](../research/2026-09-26-pty-session.md) distinguish actual PTY behavior from forced short-write tests; this is not a performance benchmark.

The [native-boundary run](results/p0-09-native-boundary/summary.json) records 70 relevant Rust tests plus compiled C and Swift lifetime checks, raw commands, source hashes and native binary hashes. [Findings](../research/2026-09-26-native-boundary.md) distinguish the owned snapshot/worker contract from still-open rendering and performance work.
