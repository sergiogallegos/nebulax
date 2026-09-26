# Toolchains verified at bootstrap

Checked 2026-09-26. Versions in the earlier review remain historical evidence.

| Component | Verified version / decision | Evidence |
|---|---|---|
| Rust/Cargo | 1.98.1; installed and pinned, edition 2024/resolver 3 | [Official release index](https://blog.rust-lang.org/releases/); `rustc -Vv`, `cargo -V` |
| alacritty_terminal | 0.26.0; exact experiment pin, MSRV 1.85.0 | [Registry](https://crates.io/api/v1/crates/alacritty_terminal), release 2026-04-06 |
| serde | 1.0.229; exact pin, MSRV 1.56 | [Registry](https://crates.io/api/v1/crates/serde), release 2026-07-18 |
| serde_json | 1.0.151; exact pin, MSRV 1.71 | [Registry](https://crates.io/api/v1/crates/serde_json), release 2026-07-20 |
| sha2 | 0.11.0; exact pin, MSRV 1.85 | [Registry](https://crates.io/api/v1/crates/sha2), release 2026-03-25 |
| Xcode / Swift / SDK | Owner updated Xcode to 27.0 (27A266a), Swift 6.4 (swiftlang-6.4.0.34.1); macOS SDK 27.0, build 26A425 | `xcodebuild -version`, `xcrun swift --version`, SDK queries; matches the [Apple stable table](https://developer.apple.com/xcode/system-requirements) |
| Host macOS | 27.0 (26A428), arm64 | `sw_vers`; differs from the pre-restart observation |
| Python | Stable 3.14.7 installed and pinned for CI; metadata tooling requires 3.11+ | [Official release list](https://www.python.org/getit/source/); standard library only |
| CI checkout action | v7.0.1, commit `3d3c42e5aac5ba805825da76410c181273ba90b1` | [Official release](https://github.com/actions/checkout/releases/tag/v7.0.1) and GitHub tag-ref API, checked 2026-09-26 |
| CI Python setup action | v7.0.0, commit `5fda3b95a4ea91299a34e894583c3862153e4b97` | [Official release](https://github.com/actions/setup-python/releases/tag/v7.0.0) and GitHub tag-ref API, checked 2026-09-26 |

The crates.io API was queried directly, excluding prerelease and yanked entries; exact selected package checksums are in `Cargo.lock`. The engine package manifest and local source were inspected before use. These initial Rust dependencies use stable releases; no Rust nightly, global Rust default change or Swift toolchain replacement was used. The later reference-only Ghostty development revision is the explicit exception recorded below.

The initial build used Xcode 26.6 as host linker infrastructure. After the owner updated Xcode, its license initially blocked compiler tools; a subsequent check succeeded and Rust linking worked. Xcode 27.0/Swift 6.4 and SDK 27.0 are now verified. No agent-installed or globally switched toolchain was used. A Swift import/type-check smoke test covers AppKit, Core Text and Metal; this is not a native app or deployment-target validation. Framework behavior and older-OS compatibility still require actual experiments.

Installed Zig is 0.16.0, verified as current stable using the [official download index](https://ziglang.org/download/index.json). Zig built only the approved isolated Ghostty reference; no Zig product component has been added. Ghostty 1.3.1 requires Zig 0.15.x; its inspected development source supports 0.16.0 and is approved solely for the pinned isolated reference experiment under [ADR 0002](../adr/0002-owned-rust-engine-and-dependency-policy.md), not as a product dependency. See the [engine assessment](../../research/2026-09-26-grapheme-engine-assessment.md).

Recheck stable releases when introducing later dependencies (renderer, font, MCP, config tooling) and pin those only when used. Record updates with affected correctness and performance evidence. Compile-time research support does not establish the product's deployment minimum.

The owned Rust slice uses final Unicode 18.0.0 data and stable UAX #29 revision 49, verified 2026-09-26. [Unicode provenance](../../third_party/unicode/README.md) records the final-data/stable-annex evidence and exact data hashes. Its generator and rule implementation introduce no third-party Rust dependency; the inherited toolchain remains pinned.
