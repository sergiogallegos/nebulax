# Nebulax

Nebulax is a terminal-emulator project targeting macOS Apple Silicon first, with a Rust core and Swift/AppKit UI. The planned user command is `nebulaxterm`.

**Current stage:** architecture approved; Phase 0A engine validation in progress, with owned grapheme/history/alternate-screen/reflow slices implemented. A local AppKit preview is available; there is no packaged app to install yet. The owner selected an [owned Rust engine with minimal dependencies](docs/adr/0002-owned-rust-engine-and-dependency-policy.md). Engine internals, renderer and scheduler remain under evaluation.

## Run the research workspace

Install the pinned Rust toolchain through rustup and use Python 3.11 or newer. No Python packages are required.

```sh
scripts/verify
scripts/replay --allow-known-gaps
scripts/replay --engine owned
```

Verification checks formatting, Clippy, tests and reproducible replay. Known compatibility gaps remain visible and are tolerated only when their reviewed state hashes match. Strict `scripts/replay` exits 1 while these gaps remain. Output is saved under `target/replay/`.

The Alacritty baseline runs 19 fixtures through 244 replays: eight expectations match and eleven reviewed differences remain. See the [grapheme assessment](research/2026-09-26-grapheme-engine-assessment.md) and [initial experiment](research/2026-09-26-terminal-replay.md) for evidence and limits. The [executed Ghostty reference comparison](research/2026-09-26-ghostty-reference-comparison.md) matches all 19 expectations with explicit grapheme mode and informs the [first owned Rust slice](docs/architecture/OWNED_ENGINE_SLICE.md). These are correctness observations, not performance results or full terminal conformance.

The [owned Rust engine](research/2026-09-26-history-screen-reflow.md) has **zero Cargo dependencies**. It matches all 19 existing fixtures over 244 replays and passes 853 official Unicode 18 grapheme cases. [Valid-geometry resize now succeeds](docs/adr/0004-cursor-anchored-resize.md) with explicit crop/eviction reporting. [Bounded syntax and typed replies/effects](docs/adr/0006-bounded-parser-and-output.md) are implemented. Storage migration, broader protocol coverage and native integration remain open.

## Project navigation

- [Current work and next step](plans/NOW.md)
- [Architecture status: accepted decisions and open details](docs/architecture/STATUS.md)
- [Accepted architecture direction](docs/adr/0001-approved-direction.md)
- [Build and verification](docs/development/BUILD.md)
- [Repository map](docs/architecture/REPOSITORY_MAP.md)
- [Historical design review](review/README.md)
- [Contributing](CONTRIBUTING.md), [security](SECURITY.md), [governance](GOVERNANCE.md)

Original code is [MIT licensed](LICENSE). Dependencies retain their own licenses; see [third-party notices](THIRD_PARTY_NOTICES.md).

The [storage/snapshot experiment](research/2026-09-26-storage-snapshot-spike.md) compares compact cells and immutable snapshots with retained raw evidence. It informs the next integration design; the terminal core representation is unchanged.

A [single-PTY lifecycle experiment](research/2026-09-26-pty-session.md) now connects the engine to a deterministic macOS child, testing replies, backpressure, resize and cleanup. This remains headless research; a native window and interactive input are next.

The [owned snapshot and native boundary](research/2026-09-26-native-boundary.md) now has real C/Swift lifetime checks, bounded frame handles and worker-owned PTY cleanup. A minimal AppKit window and bounded basic input are next.

Run `scripts/native-window` for the [interactive AppKit preview](experiments/native-window/README.md), or `scripts/native-window --shell` for a clean basic-shell session. The native-event/resize/close test is recorded in the [window findings](research/2026-09-26-native-window.md); IME, accessibility and broad shell/TUI compatibility remain open.
