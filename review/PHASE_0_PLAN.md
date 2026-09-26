# Phase 0 experiment and benchmark plan

Status: authorized by the owner's 2026-09-26 approval. Bootstrap and the first bounded replay slice are complete locally; the remaining experiments are not executed. See [the current plan](../plans/NOW.md) and [initial results](../research/2026-09-26-terminal-replay.md). The methodology below is retained from the proposal with the 2026-09-26 staged-execution revision.

## Purpose and ordering

Resolve uncertain engineering choices using small runnable experiments. Avoid building a partial full product under the label “spike.” Every experiment records the question, hypothesis, versions/commits, method, raw results, findings, confidence, limitations, recommendation and ADR consequence.

Order: establish baseline tools/metadata → headless engine acceptance → PTY and native text/FFI feasibility → renderer/storage/scheduler comparisons → config round-trip and approval transaction proof → review results. Collect competitor baselines once their fixture and host prerequisites are ready. Benchmark activity is serialized on the host. Development tasks may be independent; they do not justify simultaneous performance runs.

## Staged execution — review revision 2026-09-26

| Stage | Work and decision gate |
|---|---|
| 0A — Engine viability | Build the minimal replay/metadata runner, then P0-06 with the relevant P0-07 resource observations. Produce a protocol/Unicode/resource gap matrix before choosing a production engine or custom storage. Capture initial P0-01/02 observations when fixtures are ready; incomplete physical latency instrumentation does not block headless correctness work. |
| 0B — Platform feasibility | P0-05 safe PTY lifecycle, P0-04 text mapping, and P0-09 batched FFI/native tab/input/accessibility boundary. Exercise a small vertical path before optimizing GPU or scheduling choices. Revisit the boundary if state ownership, text ranges or teardown cannot be made sound. |
| 0C — Measured alternatives | P0-03 renderer, P0-08 scheduler and remaining P0-07 storage comparison. Add challengers only where the baseline leaves a consequential unresolved choice. Finish reproducible competitor measurements and record unavailable instruments explicitly. |
| 0D — Config proof and synthesis | P0-10 representative transaction/approval proof, then ADR recommendations and measured targets. No production control service is needed for this proof. Do not close Phase 0 with missing evidence relabeled as a pass. |

First implementation task after Phase D: feed fixed fixtures into a pinned complete engine and emit deterministic normalized state plus run metadata. Include ASCII/cursor edits, split UTF-8 and combining sequences, emoji/ZWJ, wide-cell edge wrapping and resize/reflow cases. Check chunk-splitting equivalence against explicit expected state, not just agreement with another emulator. Record fixture hashes, engine/toolchain versions, repeatability and failure evidence. Mark the remaining public-v1 protocol cases as pending in the gap matrix. This first task establishes the runner and exposes integration risks; it does not claim full conformance or comparative performance.

For the full P0-06 gate, classify each required feature as supported, adapter work, upstream/fork work, or unresolved. Include bounded history/side data, input encoding and typed side effects, plus logical text needed for selection and OS accessibility. Document maintenance consequences of every required patch. Investigate an alternative engine before building a second authoritative grid. A limited `vte` semantic prototype tests a specific gap only; it is not a competing full engine project.

P0-10 starts with a cosmetic field, shell/argv, an environment override, a permission field and one included profile. Prove dry-run/revision conflicts, protected edits while stopped, first import, missing/corrupt approval state, restart, dependency changes after approval, one-document recovery and representative schema/migration behavior. Add a forced external-editor write between revision check and replacement to expose the uncoordinated-writer limitation. Preserve recoverable revisions and report precisely what conflicts can be detected; do not claim a universal no-lost-edit guarantee from rename alone.

Physical key-to-photon measurements still require a validated instrument. Software event-to-present results can guide development but cannot substitute for that metric. If the instrument or an older macOS test environment is unavailable, record the missing evidence and bring the affected target/support claim to the owner; do not block unrelated experiments or claim verification.

## Required experiments

| ID / question | Small implementation or procedure | Evidence and exit criterion |
|---|---|---|
| P0-01 Alacritty baseline | Current stable release; fixed shell/font/grid; static idle, filled scrollback, output flood, resize and TUI workloads | Raw first-window/multiple-window memory, CPU/energy, startup, throughput and latency records. No app comparison until matched settings are reproducible |
| P0-02 Ghostty comparison | Current stable release; same workload/config intent; 1/2/5/10 tabs plus separate windows | Same records, with native-tab and feature differences explained; no winner inferred from one metric |
| P0-03 Metal text renderer | Draw identical prepared glyph runs via direct Metal and feature-restricted wgpu Metal; initial native shell baseline, shared atlas and event-driven scheduling | Startup, stripped size, dependency count, steady/peak CPU/GPU, frame distribution and idle wakeups. Choose direct Metal only with a documented cost/maintenance rationale |
| P0-04 Core Text | Shape/rasterize ASCII, ligatures, combining marks, emoji ZWJ, CJK, PUA, variable fonts and fallback; map glyphs to cell ranges | Golden samples, fallback correctness, first-use latency, cache/object growth, scale transitions. Compare Rust components only for explicit gaps/costs |
| P0-05 PTY throughput/lifecycle | One child emitting fixed corpora; compare audited native launch/PTY path and portable-pty where useful | Bytes/sec, partial-write handling, echo responsiveness during floods, memory, startup errors/signals/reaping. Pass safe spawn/teardown before throughput optimization |
| P0-06 VT reuse | Pin stable alacritty_terminal; adapt headless feed/snapshot; compare vte+small semantic slice and Ghostty/reference behavior | License/dependency/Unicode/protocol gap matrix and replay metrics. Prefer reuse if gaps have bounded upstream/adaptation paths. Do not compare parser-only speed with full emulation as equivalent work |
| P0-07 Cells/styles/history | Benchmark direct-style cells, 16-byte interned candidate and 8-byte page candidate, or measure selected engine if reuse wins | Allocation count, bytes/live cell including extras, scrolling/reflow/mutation time and high-entropy styles. Layout size alone cannot select winner |
| P0-08 IO/threading | Thread per PTY vs shared mio reactor vs hybrid; optional Tokio challenger with minimal features | 1/10/100 sessions; quiet, mixed, and one/two flooders; p95/p99 input responsiveness, wakeups, committed stack, fairness and shutdown complexity. Select lowest-complexity model meeting targets |
| P0-09 Swift/Rust FFI and native tabs | Minimal AppKit surface driving batched Rust state, snapshot lifecycle and actual native tab detach/merge | C header drift, allocations, retain/release, callback teardown, UI thread rules, no per-cell boundary calls, native title/restore feasibility. Compare Rust/Swift renderer adapter effort |
| P0-10 Config/schema transaction | Real partial/resolved Rust models for representative fields; serde/schemars/toml_edit; one-document atomic edit and journal | Comment preservation limits, schema/type parity, include cycle limits, migrations, concurrent edits, denied/protected mutation and crash injection. No approval token accepted for a changed candidate |

For each spike, start with a one-page plan and a bounded work allowance proposed in the tracking issue. Stop after the planned comparison, or report the missing evidence and specific next experiment. Do not expand into graphics protocols, splits, a multiplexer, or three production platform shells.

## Baseline fixture contract

Use a dedicated benchmark configuration directory, fixed locale, fixed shell initialization and a deterministic prompt marker. Pin font files/versions and render settings; disable transparency/background images and unrelated integrations. Document both a matched static configuration (cursor blink off) and ordinary defaults (blink may be on). Record screen refresh, resolution, scaling, fullscreen/window state, physical drawable size and actual character grid separately.

Run plugged in with the same energy mode and a settled thermal state. Close only experiment-owned processes, never the user's working terminals. Record other significant host activity. Do not run builds, another benchmark, screen recording or profiling at the same time unless that measurement explicitly requires it. Profiling overhead is a separate run.

Use current stable Alacritty and Ghostty artifacts as of the experiment date; save release URL, version, artifact SHA-256 and source commit if identifiable. Snapshot proposed-product commit and dirty diff hash. If a vendor binary has unknown compiler flags, record unknown rather than inventing them. Linux/Windows methods are planned for their later phases, not simulated on this Mac.

## Measurement recipes

### Memory and idle behavior

Ten independent launch/close runs per configuration initially. Wait for deterministic prompt readiness, then 30 seconds of quiescence. Sample memory at 30, 60 and 120 seconds; report temporal samples within a run as correlated rather than thirty independent launches. Repeat with exactly 10,000 produced scrollback rows, alt-screen use, mixed glyph/font activation, and 1/2/5/10 tabs/windows. Record retained memory after tabs close and atlas eviction; distinguish steady from peak.

Local command syntax was confirmed with installed tool help during research. Example commands for the future harness, with PID/result paths supplied by the harness:

```sh
footprint -p "$terminal_pid" -f bytes -j "$run_dir/footprint.json"
vmmap -summary "$terminal_pid" > "$run_dir/vmmap.txt"
ps -p "$terminal_pid" -o pid=,ppid=,etime=,time=,rss= > "$run_dir/process.txt"
```

No such process measurement was executed in Phase A. If OS permissions prevent detailed accounting, record the denied instrument and alternate metric; do not relabel RSS as physical footprint. Capture primary GUI, shells, children and optional helper separately, and report their scoped aggregate. Obtain CPU utilization from CPU-time deltas over a fixed five-minute interval; preserve normalized core units. Compare against a no-application control interval for energy, noting noise. Event/timer wakeup counts complement CPU percentage.

Use Metal Instruments/GPU allocation tracking in separate attributed runs. On Apple Silicon, CPU and GPU often refer to shared backing: report logical resource sizes and physical attribution without summing duplicates. On later Linux runs use `/proc/<pid>/smaps_rollup` PSS and report GPU measurement availability; on Windows report private working set plus commit and relevant GPU process counters.

### Throughput and parser/state work

For pinned vtebench, the [documented entry point](https://github.com/alacritty/vtebench) is `cargo run --release -- --dat results.dat`; invoke from its pinned source checkout inside the terminal under test. Add `--locked` for the controlled build. Save exact executable/workload hashes and raw `.dat`; do not label its PTY-read result as render latency.

Use three warmups and 30 timed runs per workload for internal replay/PTY measurements. Randomize candidate order by recorded seed and alternate candidates in blocks to reduce thermal/order bias. Corpus classes: printable ASCII, many short lines, long lines, random cursor/edit operations, color churn, mixed UTF-8, combining/emoji, OSC and resize events. Record bytes and final state/checksum so a faster run cannot “win” by dropping work.

Expose internal parser/state completion in the headless experiment. For black-box terminals, send a supported response query after the workload and await its reply to establish parser progress; verify query behavior and handle timeout. A response is not proof of display presentation. A distinct final visual marker/camera or instrumented renderer provides the display-completion measurement. Keep raw flood/drain timing, parsed completion and presented completion as separate metrics.

### Input latency and frame pacing

Use a physical input generator or synchronized LED/input marker and a camera with recorded frame rate, shutter and display position. First validate instrument uncertainty against known timing behavior. Start with 200 inputs per condition in three separated blocks, at idle and during a flood. Report empirical median/p95/p99 with sample count and camera quantization; do not overinterpret sparse tail estimates. Include display scanout position and refresh in results. Use Typometer only after validating it on the platform.

Instrument event arrival → PTY write → resulting damage → GPU submit → presentation independently for diagnosis. These timestamps are not a replacement for physical key-to-photon. Capture frame-time distributions, dropped/missed presentation opportunities and latency under Neovim scrolling and representative agent CLI redraws. Record app versions, scenario scripts, corpus and screen size; redact any personal terminal captures.

### Startup, size and development cost

Measure process launch → first presented frame and process launch → deterministic usable prompt separately. Directly launch the app binary or use a controlled OS launch path consistently; `open -a` return time is not readiness. Run 30 warm launches. Label first-launch-after-install, first-launch-after-reboot and warm-cache conditions honestly; do not call repeated launches cold. Five independent reboot/first-use observations may establish an initial cold range, but have insufficient samples for strong tail claims. Do not globally purge caches or reboot a working host as part of routine automation.

Report stripped/unstripped binary, total app bundle, bundled fonts, optional helper and debug symbols separately. Save `cargo tree` and resolved package counts with target/features; distinguish direct/transitive/build dependencies and native frameworks. Optional developer cost: clean/incremental build time on fixed changes with independent target/DerivedData paths, compiler version and cache state recorded.

## Result artifacts

Proposed layout after implementation:

```text
benchmarks/
  README.md
  harness/
  workloads/<name>/manifest.json
  baselines/alacritty/<version>/<run-id>/
  baselines/ghostty/<version>/<run-id>/
  results/<experiment>/<commit>/<run-id>/
    environment.json
    samples.jsonl
    summary.md
    artifacts.json
research/<date>-<experiment>.md
```

Required `environment.json` fields: schema version; run ID/UTC date; tool/harness commit; product name/version/commit/dirty state; artifact/config/workload/font hashes; OS name/version/build; CPU/GPU/RAM/architecture; display resolution/refresh/scale; window physical size/grid; shell/version/initialization; terminal options; compiler/SDK; process scope; measurement tool/version/arguments; metric units; repetitions/warmups/order seed; power/thermal state and exclusions.

Each sample records run number, monotonic timestamps, measured value/units, completion condition, success/failure and raw artifact links. Large files live in artifact storage with checksums; small sanitized fixtures and summaries remain in Git. A summary lists all runs including invalid ones with reasons, median/tails/spread, paired comparisons/intervals where meaningful, unknowns and interpretation. Do not publish private cwd, environment variables or screen content in public metadata.

## Decision and regression rules

Correctness, privacy, authority boundaries and responsiveness are hard requirements. Performance ranking follows only after those pass. Avoid optimizing a parser benchmark at the cost of user input starvation or dropping font correctness.

After baselines, propose concrete targets per workload with units, allowed variability, measurement method and feature settings. Choose regression tolerances above the observed measurement noise and justify them; there is no invented universal “5%” gate today. A worse memory result can be accepted for a documented native feature with owner review. Release marketing may quote only reproducible public conditions.

Final Phase 0 review accepts/supersedes the VT, renderer, text, IO, FFI and config ADRs, records tested macOS minimum and toolchain setup, and creates the first macOS MVP implementation issue. Inconclusive or inaccessible measurements stay explicit; they are never converted into a passed result.
