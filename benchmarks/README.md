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

The [native-window GUI run](results/p0-10-native-window/window.json) retains event/resize/close observations, a [rendered grid bitmap](results/p0-10-native-window/window.png), source/binary hashes and command metadata. This is native correctness evidence, not physical input latency or renderer performance.

The [cursor/region native run](results/p0-11-cursor-regions/window.json) extends that path with origin-relative replies, scrolling between fixed rows and application-mode key encoding. [Findings](../research/2026-09-26-cursor-regions.md) describe the independent core/transport tests and compatibility limits.

The [compact core comparison](results/p0-12-compact-storage/summary.json) records 100 measured samples plus 20 warmups against the exact previous engine with an accounting-only overlay. [Findings](../research/2026-09-26-compact-storage.md) explain capacity savings, extra cluster allocations and resize regressions. The separate [native regression](results/p0-12-compact-native/window.json) verifies the integrated path after the representation change.

The [style boundary run](results/p0-13-style-boundary/summary.json) records 110 relevant Rust tests plus C/Swift ABI v2 and palette lifetime checks. The [style GUI run](results/p0-13-styles/window.json) verifies styled wide cells and background erasure after resize, with a [captured grid](results/p0-13-styles/window.png). [Findings](../research/2026-09-27-bounded-styles.md) describe supported behavior and limits; neither run is a performance measurement.

The [tab/erase boundary run](results/p0-14-tabs-boundary/summary.json) records 123 relevant Rust tests, including the added clean-shell PTY case, plus C/Swift checks. The [native run](results/p0-14-tabs-native/window.json) verifies tab positions and full-line erasure while retaining prior style/input/resize/cleanup checks. [Findings](../research/2026-09-27-tabs-and-erasure.md) explain scope and limits.

The [insertion/deletion boundary run](results/p0-15-edit-boundary/summary.json) records 134 relevant Rust tests plus C/Swift checks. The [native run](results/p0-15-edit-native/window.json) verifies ICH/DCH and IL/DL through a temporary row inside a scroll region, preserving prior regressions. [Findings](../research/2026-09-27-insertion-deletion.md) document scope.

The [autowrap/visibility boundary run](results/p0-16-modes-boundary/summary.json) records 145 relevant Rust tests plus ABI v3 C/Swift checks. The [native run](results/p0-16-modes-native/window.json) compares hidden/visible cursor pixels with unchanged text/row versions and verifies no-wrap edge output. [Findings](../research/2026-09-27-autowrap-visibility.md) describe the tested policies.

The [device-reply boundary run](results/p0-17-replies-boundary/summary.json) records 154 relevant Rust tests plus C/Swift checks. The [native run](results/p0-17-replies-native/window.json) requires exact identification/status/mode replies before startup, while retaining prior input, style, edit, visibility, resize and cleanup assertions. [Findings](../research/2026-09-27-device-replies.md) distinguish this controlled probe from real-TUI compatibility.

The [reset boundary run](results/p0-18-reset-boundary/summary.json) records 165 relevant Rust tests plus C/Swift checks. The [native run](results/p0-18-reset-native/window.json) validates soft/hard reset before startup while retaining existing native-event, rendering, resize and cleanup checks. [Findings](../research/2026-09-27-terminal-reset.md) document the reset policies and retained output semantics.

The [owned-binding SDK run](results/p0-19-owned-abi/summary.json) checks 39 ABI values and eight prototypes against active SDK headers. The [boundary run](results/p0-19-owned-boundary/summary.json) retains 166 Rust tests plus C/Swift checks and a three-package owned Cargo tree; the [GUI run](results/p0-19-owned-native/window.json) preserves prior native/reset/rendering/cleanup assertions. [Findings](../research/2026-09-27-owned-pty-bindings.md) record scope and platform limits.

The [paste boundary run](results/p0-20-paste-boundary/summary.json) retains 176 Rust tests plus C/Swift checks with exact real-PTY paste/reset exchanges. The [native run](results/p0-20-paste-native/window.json) adds one synthetic explicit paste to the existing native probes; [paste.png](results/p0-20-paste-native/paste.png) shows the acknowledged grid after the separate visibility-only pixel comparison. [Findings](../research/2026-09-27-bracketed-paste.md) describe bounds and unsupported clipboard/large-paste behavior.

The [IRM boundary run](results/p0-21-insert-boundary/summary.json) records 187 relevant Rust tests plus C/Swift execution. The [native run](results/p0-21-insert-native/window.json) inserts the `IRM OK` marker and verifies mode reports while retaining prior paste, cursor, drawing, resize and cleanup assertions. [Findings](../research/2026-09-27-insert-mode.md) document streaming grapheme policies and correctness scope.
