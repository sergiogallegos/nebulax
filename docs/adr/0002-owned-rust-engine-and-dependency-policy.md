# 0002 — Owned Rust engine and minimal dependencies

Status: accepted by owner instruction on 2026-09-26. Supersedes ADR 0001's complete-engine reuse-first direction.

Subsequent decision: [ADR 0018](0018-standard-library-only-rust.md) supersedes the third-party-crate allowance below with a standard-library-only Rust goal. The owned-engine and isolated-reference directions remain in force; the original decision is preserved here as history.

## Decision

Nebulax will develop its terminal engine in Rust. Study existing engines, including Ghostty, for behavior, logic, architecture and design, then implement the required behavior in Nebulax. Ghostty libraries are not planned product dependencies. Alacritty remains an isolated research baseline; its presence in the experiment does not select it for the product.

Keep dependencies as few as practical. Prefer focused code we own when the implementation and maintenance burden are reasonable. A foundational crate such as Tokio is acceptable when justified, but mentioning it does not select it or authorize adding it without a present need. Assess each proposed dependency by purpose, direct and transitive footprint, build/runtime cost, correctness benefit and the cost of maintaining an equivalent implementation. A low direct-crate count alone is insufficient.

Use reference projects to understand algorithms and observable behavior, with source provenance recorded. Write our own implementation against explicit requirements and independent fixtures. Any actual copied or adapted source must retain its provenance and applicable notices; translating source to Rust is not automatically an original implementation.

The owner approves the proposed isolated Ghostty comparison, including the pinned development revision `6301810a48aaa3426887a4316668f18833a40138`, solely as a research reference. This resolves the pending experiment exception, not the stable-dependency policy for product code. Keep the reference build outside the production dependency graph and use stable Zig 0.16.0. No further permission for this comparison is needed.

## Consequences and next step

The engine direction is now owned Rust; its internal design remains to be demonstrated. This takes on parser/state, grapheme handling, editing/reflow, protocol compatibility, resource limits and ongoing maintenance. Few dependencies do not by themselves prove better performance or correctness. Build in bounded slices and preserve the full acceptance requirements.

Complete the headless Ghostty comparison described in the [assessment](../../research/2026-09-26-grapheme-engine-assessment.md), recording differences without treating the reference as the specification. Use the findings to define the first owned Rust state model and a small vertical slice covering incremental text input, cluster ownership, width changes and editing. Extend the existing independent corpus to validate that slice before widening protocol coverage.

Maintain one authoritative state model per engine. A comparison can run independent engines, but the product must not shadow a third-party grid to compensate for its gaps. Preserve existing baseline evidence. The approved Rust/Swift boundary, native accessibility, control policy and single-agent workflow remain in force.
