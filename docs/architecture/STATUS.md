# Architecture decisions and current phase

The project-level design review is accepted. We are now in **Phase 0A: validating the engine and implementation details**, with working Rust code and reproducible research harnesses. Bootstrap is complete locally. This is neither an undecided architecture nor a finished application design.

| Area | Status |
|---|---|
| Product/platform | Nebulax, planned `nebulaxterm` CLI; macOS Apple Silicon first — accepted |
| Main boundary | Owned Rust core, Swift/AppKit native UI, private C boundary — accepted |
| Dependencies | Minimal justified dependencies; reference engines inform our design — accepted |
| Configuration/control | TOML/schema, shared policy, daemon-free v1, scoped CLI/MCP — accepted direction |
| Terminal state | Implemented Rust graphemes, bounded history, alternate screen and resize/reflow; broader protocol/compatibility work remains |
| Rendering/fonts | Metal/Core Text first to evaluate, wgpu comparison pending; implementation and performance choice open |
| Runtime/scheduling | Detailed PTY lifecycle, scheduling, backpressure and measured resource behavior remain to be proven |
| Native integration | IME, accessibility, Swift lifecycle/FFI and tested macOS deployment floor remain unimplemented |
| Product readiness | No GUI application, installation, release or production performance claim |

[ADR 0001](../adr/0001-approved-direction.md) records the initial accepted direction; [ADR 0002](../adr/0002-owned-rust-engine-and-dependency-policy.md) changes the engine direction to owned Rust. The [history/resize policy](../adr/0003-history-screen-and-reflow.md) records current implementation choices and a remaining resize limitation. These choices are tested within Phase 0 and can be revised when evidence warrants it; they do not reopen the already approved project direction.

The sequence is accepted architecture → bootstrap → **engine/native feasibility and implementation slices (current)** → native MVP after the relevant gates are met. Completing the current 19-fixture corpus is useful evidence, not completion of Phase 0. [Current work](../../plans/NOW.md) tracks the next bounded task, while the [original Phase 0 plan](../../review/PHASE_0_PLAN.md) retains the broader acceptance requirements.

The [foundation review](FOUNDATION_REVIEW.md) records the assessment of the owner-provided Opus feedback and the revised order: foundations and an early integrated native path before broad protocol expansion.
