# Architecture decisions and current phase

The project-level design review is accepted. We are now in **Phase 0A: validating the engine and implementation details**, with working Rust code and reproducible research harnesses. Bootstrap is complete locally. This is neither an undecided architecture nor a finished application design.

| Area | Status |
|---|---|
| Product/platform | Nebulax, planned `nebulaxterm` CLI; macOS Apple Silicon first — accepted |
| Main boundary | Owned Rust core, Swift/AppKit native UI, private C boundary — accepted |
| Dependencies | Minimal justified dependencies; reference engines inform our design — accepted |
| Configuration/control | TOML/schema, shared policy, daemon-free v1, scoped CLI/MCP — accepted direction |
| Terminal state | Implemented Rust graphemes, bounded history, alternate screen, resize/reflow and cursor/region state; broader protocol/compatibility work remains |
| Parser/output | Bounded generic syntax, separate semantics and ordered typed replies/effects implemented with tested PTY transport; broader VT behavior remains open |
| Storage/snapshots | Bounded owned visible snapshots implemented; compact grid storage/reclamation remains experimental |
| Rendering/fonts | Metal/Core Text first to evaluate, wgpu comparison pending; implementation and performance choice open |
| Runtime/scheduling | Worker-owned PTY, bounded transport and asynchronous close verified; production scheduling/job trees remain open |
| Native integration | Private C/Swift lifetimes and minimal AppKit/basic-input path verified; IME/accessibility and deployment floor remain open |
| Product readiness | Local interactive preview; no installation, release or production performance claim |

[ADR 0001](../adr/0001-approved-direction.md) records the initial accepted direction; [ADR 0002](../adr/0002-owned-rust-engine-and-dependency-policy.md) changes the engine direction to owned Rust. The [history/resize policy](../adr/0003-history-screen-and-reflow.md) records the initial implementation; [ADR 0004](../adr/0004-cursor-anchored-resize.md) replaces its resize rejection with explicit bounded cropping. These choices are tested within Phase 0 and can be revised when evidence warrants it; they do not reopen the already approved project direction.

The sequence is accepted architecture → bootstrap → **engine/native feasibility and implementation slices (current)** → native MVP after the relevant gates are met. Completing the current 19-fixture corpus is useful evidence, not completion of Phase 0. [Current work](../../plans/NOW.md) tracks the next bounded task, while the [original Phase 0 plan](../../review/PHASE_0_PLAN.md) retains the broader acceptance requirements.

The [foundation review](FOUNDATION_REVIEW.md) records the assessment of the owner-provided Opus feedback and the revised order: foundations and an early integrated native path before broad protocol expansion.

[ADR 0005](../adr/0005-storage-and-snapshot-direction.md) records the measured storage/snapshot direction and its unresolved production limits.

[ADR 0006](../adr/0006-bounded-parser-and-output.md) records syntax/output bounds and the runtime consumption contract.

[ADR 0007](../adr/0007-pty-lifecycle-experiment.md) records the tested PTY boundary and remaining native-lifecycle work.

[ADR 0008](../adr/0008-snapshot-worker-and-private-ffi.md) records owned snapshot, worker cleanup and private C lifetime contracts.

[ADR 0009](../adr/0009-native-window-and-basic-input.md) records the minimal window/input path and its limits.

[ADR 0010](../adr/0010-cursor-regions-and-input-modes.md) records screen-owned cursor/region behavior and terminal-wide application cursor-key encoding, verified through the native path.
