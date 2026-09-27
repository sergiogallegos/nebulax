# Architecture decisions and current phase

The project-level design review is accepted. We are now in **Phase 0A: validating the engine and implementation details**, with working Rust code and reproducible research harnesses. Bootstrap is complete locally. This is neither an undecided architecture nor a finished application design.

| Area | Status |
|---|---|
| Product/platform | Nebulax, planned `nebulaxterm` CLI; macOS Apple Silicon first — accepted |
| Main boundary | Owned Rust core, Swift/AppKit native UI, private C boundary — accepted |
| Reusable library | Explicit third-party embedding goal: headless Rust engine plus optional text/render/session components; public stable API/ABI not released |
| Dependencies | Zero third-party crates across owned Rust product/library code; std + original code. Engine/session/bridge comply with an enforced manifest closure and SDK-checked bindings; reference harness dependencies remain research-only |
| Configuration/control | TOML/schema, shared policy, daemon-free v1, scoped CLI/MCP — accepted direction |
| Terminal state | Implemented Rust graphemes, bounded history, alternate screen, resize/reflow, cursor/region state bounded SGR styles, tabs, display erasure, insertion/deletion, autowrap/visibility/insert modes and soft/hard reset; broader protocol/compatibility work remains |
| Parser/output | Bounded generic syntax, separate semantics and ordered typed replies/effects plus minimal device/version identification and state-derived mode reports implemented with tested PTY startup transport; broader VT behavior remains open |
| Storage/snapshots | 16-byte directly owned cells, tail reclamation and bounded snapshots implemented; shared/chunked storage and performance tuning remain open |
| Rendering/fonts | Direct Metal + Core Text on macOS; ligatures required. Linux GTK4/OpenGL candidate, Windows direct backend API open. Preview uses Core Text drawing; production GPU/text implementation remains pending |
| Runtime/scheduling | Worker-owned PTY, bounded transport and asynchronous close verified; production scheduling/job trees remain open |
| Native integration | Private C/Swift lifetimes and minimal AppKit/basic-input and bounded explicit paste paths verified; IME/accessibility and deployment floor remain open |
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

[ADR 0011](../adr/0011-compact-owned-cell-storage.md) records measured compact-cell integration, bounded direct ownership and the dense-cluster resize tradeoff.

[ADR 0012](../adr/0012-bounded-styles-and-native-rendition.md) records the bounded style table, SGR subset, palette-owning snapshots, private ABI v2 and native color/attribute drawing. Broader protocol and typography acceptance remain open.

[ADR 0013](../adr/0013-tabs-and-erasure.md) records shared bounded tab stops and inclusive wide-aware line/display/history erasure, tested through a clean shell and native preview.

[ADR 0014](../adr/0014-insertion-and-deletion.md) records in-place character/line edits, wide-owner repair, margin/history isolation and native evidence.

[ADR 0015](../adr/0015-autowrap-and-cursor-visibility.md) records saved screen autowrap, terminal-wide visibility and metadata-only delivery through private ABI v3 to the native cursor overlay.

[ADR 0016](../adr/0016-device-and-mode-replies.md) records the minimal experimental identification profile, state-derived mode reports and bounded exact startup exchanges through PTY/native integration.

[ADR 0017](../adr/0017-reusable-library-and-native-backends.md) records reusable-library scope, native frontend ownership, optional shaping/render/session layers and the owner's direct-backend direction. It supersedes the earlier mandatory wgpu comparison; no new backend or stable public SDK is implied.

[ADR 0018](../adr/0018-standard-library-only-rust.md) tightens the Rust dependency direction to std plus owned implementation across the library, including platform bindings. Earlier minimal-dependency allowances are superseded; the former PTY libc migration gap is resolved by ADR 0020 below.

[ADR 0019](../adr/0019-soft-and-hard-terminal-reset.md) records reset semantics, output ordering, resource ownership and PTY/native startup evidence.

[ADR 0020](../adr/0020-owned-macos-pty-bindings.md) records original macOS ABI declarations, independent SDK checking, inherited-signal validation and the enforced std-only owned Cargo closure.

[ADR 0021](../adr/0021-bounded-bracketed-paste.md) records global paste-mode/query/reset semantics, bounded native admission, control rejection, newline normalization and serialized PTY framing, with C/Swift/AppKit evidence. System clipboard/menu integration remains separate.

[ADR 0022](../adr/0022-ansi-insert-mode.md) records ANSI insert-mode ownership, streaming width adjustments and shared column movement, with real PTY and native preview validation. The next bounded experiment addresses immutable text runs and Core Text shaping/cell mapping before Metal integration.
