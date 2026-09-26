# Newest stable toolchain policy

Date checked: 2026-09-25. Historical evidence: the newest-stable requirement remains in force; [bootstrap verification on 2026-09-26](../docs/development/TOOLCHAINS.md) supersedes these observed versions and host details.

The owner explicitly requires beginning development with the newest stable versions of the languages, frameworks, and tools we select. This supplements the original specification. Do not select old tutorial versions, nightly compilers, beta SDKs, release candidates, or development Git branches as the default.

## Observed environment and upstream releases

| Component | Observed on this Mac | Upstream evidence at research time | Bootstrap action |
|---|---|---|---|
| Rust / Cargo | Both 1.98.1 | Official [Rust release index](https://blog.rust-lang.org/releases/) lists 1.98.1, released September 3, 2026 | Recheck; pin newest stable patch in `rust-toolchain.toml` |
| Rust edition | Not a compiler version | Rust 2024 is a stable edition; the [release index](https://blog.rust-lang.org/releases/) records its stabilization with 1.85 | Use edition 2024 unless a newer edition has stabilized by bootstrap; resolver 3 |
| Xcode | 26.6, build 17F113 | Apple's [SDK/toolchain table](https://developer.apple.com/xcode/system-requirements) lists Xcode 27 as a non-beta release, with Swift 6.4; 27.1/27.2 entries are marked beta | Use current stable Xcode for Swift/SDK compilation; verify actual available release/build before installation |
| Apple Swift | 6.3.3, target arm64-apple-macosx26.0 | Swift bundled with stable Xcode is the relevant Apple-platform compiler; [Swift's macOS install guide](https://www.swift.org/install/macos/) directs Apple development to Xcode | Use Swift 6 language mode and compiler bundled with selected stable Xcode; record the full compiler build |
| macOS host | 26.6.2, build 25G83 | Locally observed, not a claim about latest macOS | No operating-system upgrade is needed for research; host OS and app deployment minimum are separate decisions |
| AppKit / Metal / Core Text | Installed Apple SDK frameworks | Versioned with the selected SDK, rather than independent package versions | Record SDK build and deployment target; availability-check APIs newer than the minimum OS |

The Xcode difference is a setup prerequisite before Swift spikes. Nothing has been installed or switched globally. A newer SDK does not mean that the application must require the newest macOS version.

## Dependency observations, not an installed dependency set

The following official package documentation was inspected. These are versions observed on the linked pages, not a claim that a future bootstrap should reuse this snapshot blindly.

| Candidate | Observed release | Purpose |
|---|---|---|
| [`alacritty_terminal`](https://docs.rs/alacritty_terminal/latest/alacritty_terminal/) | 0.26.0 | Complete Rust terminal state candidate |
| [`wgpu`](https://docs.rs/wgpu/latest/wgpu/) | 30.0.1 | Comparative renderer spike, not selected production backend |
| [`toml_edit`](https://docs.rs/toml_edit/latest/toml_edit/) | 0.25.15+spec-1.1.0 | TOML edits preserving most presentation details |
| [`schemars`](https://docs.rs/schemars/latest/schemars/) | 1.2.2 | Generated JSON Schema |
| [Alacritty](https://github.com/alacritty/alacritty/releases) | 0.17.0 | Competitor baseline candidate |
| [Ghostty](https://ghostty.org/download) | 1.3.1 | Competitor baseline candidate |

Other prospective dependencies include `serde`, `serde_json`, `vte`, `mio`, `cbindgen`, and platform-specific bindings. They are not dependencies yet. At the first experiment that uses one, query its primary release registry, exclude prerelease/yanked versions, inspect the manifest and release notes, and save the chosen exact version and retrieval date. Do the same for GTK, `gtk4-rs`, libadwaita, `windows`, font libraries, MCP SDK, and website tools when their phases begin. Do not install Linux/Windows toolchains during macOS bootstrap merely to complete a version table.

Warning about documentation provenance: the GTK API page inspected reports 4.23.4. Documentation labeled “latest” can describe a development series. Verify actual stable release tags; do not promote a docs-page version into a stable dependency pin automatically.

## Reproducible use of current stable releases

1. Recheck official releases immediately before Phase D and each new platform phase. Record source URL, publication date, version, exact build, checksum where supplied, and check date.
2. Pin the Rust toolchain numerically, commit `Cargo.lock`, use workspace dependency declarations, and validate with `--locked`. Record the selected Xcode/SDK build in canonical development docs and CI configuration. Commit `Package.resolved` if Swift packages are introduced.
3. Select the newest compatible stable direct dependencies. Inspect dependency MSRVs and native-library constraints before coding against APIs. If current stable components conflict, explain the conflict and propose an alternative; do not silently downgrade a language or major framework.
4. Use exact action commit hashes and recorded action release versions for CI supply-chain reproducibility. Avoid floating tool installers in release jobs.
5. Update deliberately through small changes with appropriate conformance, schema, ABI, rendering, and benchmark checks. Newest stable at project start does not mean an unreviewed `cargo update` on every build.
6. Treat nightly-only fuzz/sanitizer tooling as an isolated developer test exception, pinned separately and documented. The shipped product and required normal build stay on stable Rust.

A crate's MSRV describes compatibility; it is not a direction to start this project on an older compiler. The initial project MSRV can equal its chosen compiler baseline and be revisited if distributing reusable crates becomes a goal.

## Local evidence

Read-only commands executed: `rustc --version`, `cargo --version`, `swift --version`, `xcodebuild -version`, and `sw_vers`. No code was compiled. Installed SDK headers also confirm `NSWindow.tabbingMode`, `NSTextInputClient`, and the event-driven drawing controls on `MTKView`.
