# 0018 — Standard-library-only Rust is the dependency goal

Status: owner clarification on 2026-09-27. Supersedes ADR 0002's allowance for justified third-party product crates and ADR 0017's allowance for third-party Rust text/binding components. Owned-engine, native-shell and direct-backend decisions remain intact.

## Decision

Owner's explicit rule: **“No crates in Cargo.toml. Rust’s std/core is allowed.”** For owned Nebulax implementation, this means no third-party Cargo dependency entries: normal, build, dev/test, optional, feature-gated, target-specific, registry, Git or vendored path dependencies. Rust's toolchain-provided `std`/`core` require no dependency entry. Our own Nebulax workspace components are internal code organization, not third-party implementation.

Write Nebulax's Rust implementation ourselves, using `std`/`core` and our own workspace components. Zero third-party crates is the required direction across the reusable engine, session/runtime, rendering/font adapters and application support, not just the headless crate. New owned tests and tooling follow the same rule. Tokio, serde, libc, GPU/font wrapper crates and third-party build/procedural-macro crates are excluded. Moving a dependency behind an optional feature or into a different crate does not satisfy this rule. Vendoring or translating somebody else's implementation is not an owned-from-scratch substitute.

Use original implementations informed by documented protocols and reference behavior. Preserve provenance and notices for actual copied material; preserve the existing Unicode data/license distinction. Splitting our own code into internal crates remains compatible with zero third-party dependencies. Any future departure must be presented as a change to the owner's direction, not justified silently under the older minimal-dependency policy.

Native OS integration remains necessary under the accepted architecture. The product can call platform services through narrow owned ABI declarations or our own C/Swift/platform adapters rather than third-party Rust wrapper crates. Platform APIs/frameworks are dependencies in the system sense; this policy does not claim a terminal can operate without its OS. Metal/Core Text/AppKit remain the macOS direction. GTK4 is still an explicit Linux toolkit candidate, not part of Rust's standard library or a newly selected Rust dependency. A universal from-scratch font shaper or OS toolkit is not implied by avoiding Rust crates.

## Current gap and follow-up

At this policy decision, the local Cargo graph showed:

- `nebulax-terminal`: zero third-party crates already.
- `nebulax-pty-session`: macOS `libc = 0.2.189`; the native bridge inherits it through the session experiment. This is one existing migration gap, not a permanent exemption under this policy.
- Reference/replay research retains third-party comparison and harness dependencies. Keep it isolated from shipped library/application graphs and label it honestly; the whole research workspace is not zero-dependency today. Existing reference approvals remain research-only, not permission to reuse those crates in product code.

Before promoting PTY/runtime code into the reusable library, replace the libc crate with the required owned platform boundary, verified against SDK declarations and existing lifecycle tests. Audit structure sizes/alignments, constants, function signatures, signal handling, descriptor ownership and post-fork behavior. Preserve the narrow unsafe boundary and native integration tests; do not mechanically remove a binding dependency without equivalent ABI evidence. Record the target-specific product dependency closure as part of acceptance. No dependency or implementation is changed by this documentation decision.

Migration completed: [ADR 0020](0020-owned-macos-pty-bindings.md) removes the PTY libc dependency and enforces the owned three-package manifest closure. Reference dependencies remain isolated research. The earlier inventory above records the starting point.
