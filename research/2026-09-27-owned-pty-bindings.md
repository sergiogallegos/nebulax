# Owned PTY bindings and dependency closure

Date: 2026-09-27. Local work after checkpoint `c069a6b`; not committed. [ADR 0020](../docs/adr/0020-owned-macos-pty-bindings.md) records the ABI, dependency and safety contract.

Removed the libc crate from the macOS PTY manifest and its dependency edge from Cargo.lock. No package version changed. Owned `os/bindings.rs` supplies only the required macOS user-space declarations; std retains resource and process ownership. The engine, session and bridge now have zero third-party Cargo dependencies. The reference workspace still contains external dependencies, including libc; no reference library is linked into the owned bridge.

## Evidence

The [SDK ABI recording](../benchmarks/results/p0-19-owned-abi/summary.json) compares 39 SDK C/Rust values and checks eight function signatures. Probe sources import the actual binding file and independent SDK headers. The result retains transitive SDK-header hashes, binary hashes, compiler versions and commands; no copied SDK implementation/header payload is committed. Verification also runs a manifest-level guard for all dependency kinds/targets and the two guard regression tests.

The PTY suite has **28 Rust tests**, including **19 macOS-specific** and nine portable tests. Its added isolated signal test confirms altered inheritance before testing our PTY hook. The SDK peer checks the post-hook empty mask/default dispositions and controlling-terminal identity. Existing signal/resize, descriptor, transport, reply, EOF, exit and cleanup tests remain unchanged and pass.

`scripts/verify` passes **181 Rust tests**, seven Python test cases (six passed, one optional Ghostty test skipped), Unicode generation/hash checks, formatting, warning-denied Clippy, strict owned replay, reference replay with unchanged known-gap fingerprints, ABI checks and AppKit compilation. The configured portable set remains 159 Rust tests. All **19 owned fixtures / 244 replays** still match unchanged expectations. Hosted CI was not inspected; runtime evidence is macOS 27 arm64 with Xcode 27.

The [native-boundary recording](../benchmarks/results/p0-19-owned-boundary/summary.json) retains **166 relevant Rust tests** plus compiled C/Swift lifetime/layout checks, along with a bridge Cargo tree containing only three owned packages. The [GUI regression](../benchmarks/results/p0-19-owned-native/window.json) retains reset/startup, seven native input events, styles, tabs, edits, region/application-key behavior, no-wrap output, resize, cursor-only pixel comparison and asynchronous cleanup. [Hidden](../benchmarks/results/p0-19-owned-native/hidden.png) and [visible](../benchmarks/results/p0-19-owned-native/window.png) captures were visually inspected. Artifact hashes and current recording source hashes were checked.

## Limits and next step

This removes third-party Cargo bindings, not the operating system's C library, font services or GUI frameworks. Maintaining ABI declarations is now our responsibility; future SDK/platform additions must pass the independent check. Arm64 was executed locally; Intel and older OS floors are not newly established. Production job-tree shutdown, scheduler performance and the private-to-public SDK transition remain open.

Next add bracketed-paste mode and a bounded paste-input event through the engine/runtime, with exact framing, reset/query tests and write ordering. Native clipboard access remains an explicit frontend policy; this does not authorize clipboard reads or writes from PTY output.
