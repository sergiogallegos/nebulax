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

`scripts/verify` checks Unicode hashes/generated tables, format, Clippy with warnings denied, workspace tests, the Alacritty replay with explicit known-gap tolerance and strict owned-slice replay. Owned replay executes all 19 fixtures through 244 replays with no deferrals; its exit 0 applies to this corpus, and `--allow-known-gaps` is rejected. No Unicode download is needed for normal builds or tests. This verifies the harness and preserves compatibility failures; it does not declare the candidate production-ready. Hosted CI uses Ubuntu 24.04 for headless checks and has not been run until pushed. That is not a Linux application support claim. Native lifecycle tests will be added with actual macOS code.

The storage experiment correctness tests run with the workspace tests. Run `scripts/storage-spike` separately for serialized release measurements (three warmups and 30 samples per scenario/layout); see [method and limits](../../experiments/storage-spike/README.md). Timing measurements are not CI pass/fail thresholds.
