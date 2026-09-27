# Repository map

| Task | Owner/source | Validation |
|---|---|---|
| Continue current work | `HANDOFF.md`, `plans/NOW.md` | Inspect Git and recorded evidence |
| Review accepted versus open architecture | `docs/architecture/STATUS.md` | ADRs and current evidence |
| Change history, screens or reflow | `crates/terminal/src/screen.rs`, `docs/adr/0003-history-screen-and-reflow.md` | History/resize integration tests and owned replay |
| Change architectural direction | `docs/adr/` | Owner decision; update affected docs |
| Design public embedding/native backends | ADR 0017; existing headless crate, private native bridge and AppKit preview | Separate public API/lifetime and Metal/ligature acceptance; no released SDK yet |
| Replay terminal bytes and compare delivery | `experiments/terminal-replay/src/lib.rs` | `cargo test --workspace --locked` |
| Compare Ghostty as a research reference | `experiments/ghostty-reference/` | Pinned build, strict replay, explicit mode control |
| Change owned Rust engine | `crates/terminal/` | Unicode conformance, stream/edit invariants, owned replay |
| Change VT syntax, meaning or output flow | `crates/terminal/src/parser.rs`, `semantic.rs`, `output.rs`, ADR 0006 | `crates/terminal/tests/parser_and_output.rs`; existing owned replay |
| Change cursor/regions/input modes | `crates/terminal/src/screen.rs`, `semantic.rs`, `input.rs`, ADR 0010 | `cursor_and_regions.rs`, PTY peer and AppKit protocol probe |
| Change compact storage/ownership | `crates/terminal/src/cell.rs`, `storage.rs`, `screen.rs`, ADR 0011 | Seven storage tests; `scripts/storage-integration` serial before/after measurements |
| Change terminal reset | `screen.rs`, `lib.rs`, `semantic.rs`, ADR 0019 | Ten core tests; PTY frame/key/reply handshake and native startup probe |
| Change autowrap/cursor visibility | `screen.rs`, `semantic.rs`, `snapshot.rs`, private ABI v3, ADR 0015 | Ten core mode tests; visibility-only worker and GUI pixel checks |
| Change insertion/deletion | `crates/terminal/src/screen.rs`, `semantic.rs`, ADR 0014 | Ten editing tests, wide-owner oracle, shell and native probe |
| Change tabs/erase ranges | `crates/terminal/src/tabs.rs`, `screen.rs`, `semantic.rs`, ADR 0013 | Twelve tab/erase tests, clean shell and native probe |
| Change styles/SGR | `crates/terminal/src/style.rs`, `semantic.rs`, `snapshot.rs`, ADR 0012 | Twelve style tests, C/Swift palettes and AppKit style probe |
| Change owned snapshots | `crates/terminal/src/snapshot.rs`, ADR 0008 | Lifetime/Unicode, skipped-frame versions, byte/generation bounds |
| Change ANSI insert mode | `crates/terminal/src/lib.rs`, `screen.rs`, `semantic.rs`, `query.rs`, ADR 0022 | Independent owner oracle, streaming width/reset/query tests, PTY and native marker |
| Change paste framing/mode | `crates/terminal/src/input.rs`, `semantic.rs`, `query.rs`, `experiments/pty-session/src/input.rs`, ADR 0021 | Core validation/reset/query tests; partial-write/queue tests; real PTY, C/Swift and synthetic native paste |
| Change native view/basic input | `experiments/native-window/`, `crates/terminal/src/input.rs`, `experiments/pty-session/src/input.rs` | `scripts/native-window --self-test`; queue/ordering/duplex tests |
| Change native worker/ABI | `experiments/pty-session/src/worker.rs`, `experiments/native-bridge/` | Worker/handle tests; compiled C and Swift via `scripts/native-bridge` |
| Change owned OS declarations/dependency policy | `experiments/pty-session/src/os/bindings.rs`, `scripts/pty-abi`, `scripts/check-product-deps`, ADR 0020 | Independent SDK oracle; isolated inherited-signal test; Cargo declaration guard |
| Exercise PTY lifecycle/transport | `experiments/pty-session/`, `scripts/pty-session`, ADR 0007 | Thirty-three tests on macOS including workers/input/shell/startup/reset/paste/IRM; portable pump/queue tests also on Ubuntu |
| Measure storage/snapshot alternatives | `experiments/storage-spike/`, `scripts/storage-spike` | Six correctness tests; serialized release samples with capacity accounting |
| Replay owned engine | `experiments/owned-replay/`, `scripts/replay --engine owned` | All 19 fixtures strict / 244 replays; no deferrals |
| Update Unicode data/rules | `third_party/unicode/`, `scripts/generate-unicode.py` | Stable verification, checksums, regeneration, official tests |
| Change CLI exit/report behavior | `experiments/terminal-replay/src/main.rs`, integration tests | `scripts/verify` |
| Add expected behavior | `tests/fixtures/terminal-replay.json` | Review expected state independently; strict replay |
| Capture environment, artifacts and hashes | `scripts/replay` | Inspect run metadata; rerun and compare normalized reports |
| Change dependency/toolchain pins | root Cargo files, `rust-toolchain.toml`, toolchain guide | Official stable verification; `scripts/verify` |
| Review findings and open compatibility gaps | `research/`, original `review/` archive | Trace claims to raw artifacts/fixtures |
| CI | `.github/workflows/verify.yml` | Same `scripts/verify`; hosted execution separately reported |

There is one owned core crate, three Cargo research runners, a Cargo PTY lifecycle experiment, a private native-bridge experiment and one isolated C/Python reference adapter. The core is a bounded research slice, not a complete production emulator. One macOS PTY experiment and a local AppKit preview are implemented; there is no installed app, production runtime, control server, website or release pipeline yet. The planned `nebulaxterm` command is not the research binary `nebulax-replay`.

Historical review paths are retained to preserve existing links. [ADR 0001](../adr/0001-approved-direction.md) and its engine-direction supersession [ADR 0002](../adr/0002-owned-rust-engine-and-dependency-policy.md) record acceptance; review-era statements about approval pending are historical.
