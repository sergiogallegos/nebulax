# Build and validation

Prerequisites: rustup, the pinned Rust 1.98.1 toolchain with rustfmt/Clippy, Git and Python 3.11+. Python uses only the standard library. Python orchestrates metadata capture; it is not a product runtime dependency. See [toolchain verification](TOOLCHAINS.md).

```sh
rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy
scripts/verify
```

Run from a Git checkout. Cargo resolves the checked-in `Cargo.lock` with `--locked`. The Rust replay package builds without Swift/Xcode app compilation. On macOS, a working native linker/SDK is required. The owner-updated Xcode 27.0/Swift 6.4 with macOS SDK 27.0 is now verified. The initial Xcode license block was resolved before the expanded replay build.

```sh
scripts/replay
scripts/replay --allow-known-gaps
scripts/replay --allow-known-gaps --output target/replay/manual-review
scripts/replay --engine owned
cargo tree --locked -p nebulax-terminal
cargo tree --locked -p nebulax-replay
```

Strict replay returns 1 for unmet expectations. The explicit known-gap option returns 0 only if ordinary fixtures pass, checkpoint chunk equivalence holds, and each documented gap still fails with its reviewed observed-state SHA-256. New mismatches, changed gap states and unexpected passes require investigation. Invalid invocation/data or recording failure returns 2. Existing artifact directories are never overwritten.

The CLI emits deterministic JSON; the wrapper builds the pinned executable and records source/dependency/corpus/executable hashes, compiler, host, dirty state, exit status and timestamps. Results live in `target/replay/` unless a new output directory is supplied. No user environment, hostname, home directory or terminal content is captured. Use only synthetic fixtures and inspect artifacts before sharing.

`scripts/verify` checks Unicode hashes/generated tables, format, Clippy with warnings denied, workspace tests, the Alacritty replay with explicit known-gap tolerance and strict owned-slice replay. Owned replay executes all 19 fixtures through 244 replays with no deferrals; its exit 0 applies to this corpus, and `--allow-known-gaps` is rejected. No Unicode download is needed for normal builds or tests. This verifies the harness and preserves compatibility failures; it does not declare the candidate production-ready. Hosted CI now configures Ubuntu 24.04 and macOS 15 runners; results remain unverified until pushed and inspected. The macOS build runs twenty-two PTY/worker/input tests, three bridge Rust tests and compiled C/Swift smoke checks; Ubuntu runs portable Rust tests and explicitly skips native checks. Local macOS verification totals 125 Rust tests; the configured Ubuntu set totals 109. Neither is a Linux application or older-macOS support claim.

The storage experiment correctness tests run with the workspace tests. Run `scripts/storage-spike` separately for serialized release measurements (three warmups and 30 samples per scenario/layout); see [method and limits](../../experiments/storage-spike/README.md). Timing measurements are not CI pass/fail thresholds.

Run `scripts/pty-session --output target/pty-session/manual` on macOS to retain the twenty-two-test lifecycle/worker/input experiment with source/executable hashes and environment metadata. It uses synthetic Python children plus one clean Bash test with startup files disabled; see [method and limits](../../experiments/pty-session/README.md).

`scripts/native-bridge` compiles the private dynamic library and actual C/Swift 6 callers, with module caches under ignored `target/`. Pass `--output <new-directory>` to also record the 110 relevant Rust tests and retain source/native-binary hashes. Full `scripts/verify` runs the C/Swift checks automatically on macOS; no AppKit window is opened.

`scripts/verify` also compiles the AppKit preview (`scripts/native-window --check`). Run `scripts/native-window` for the synthetic interactive demo, or add `--shell` for the clean dumb-shell preview. The separate `--self-test --output <new-directory>` opens a window and needs a logged-in macOS GUI session; it records a grid bitmap and native-event/resize/close evidence. GUI execution is not part of routine CI.

Run `scripts/storage-integration --output <new-directory>` separately for the serial full-engine compact-storage comparison (100 measured samples plus 20 warmups by default). It rebuilds the previous Git checkpoint with a recorded accounting-only overlay, compiles the same driver for both layouts, and records capacity/timing results. See [method and exclusions](../../experiments/storage-integration/README.md). Do not run tests or other benchmarks concurrently with measurements.
