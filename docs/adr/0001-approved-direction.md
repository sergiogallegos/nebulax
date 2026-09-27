# 0001 — Initial architecture and research scope

Status: accepted by the owner on 2026-09-26.

Subsequent decision: [ADR 0002](0002-owned-rust-engine-and-dependency-policy.md) supersedes the complete-engine reuse-first direction with an owned Rust engine and minimal dependencies. Other decisions remain in force.

Later clarification: [ADR 0017](0017-reusable-library-and-native-backends.md) makes reuse by third-party applications an explicit goal and selects direct platform GPU backends, superseding the required Metal/wgpu comparison. The original decision below remains historical.

## Context and decision

The owner explicitly approved the revised architecture direction and proceeding with repository bootstrap and Phase 0. This satisfies the original specification's Phase C gate. The acceptance follows the technical review recorded in [the decision checklist](../../review/OWNER_DECISIONS.md).

- Begin with macOS Apple Silicon. macOS 14 is a provisional deployment target requiring actual testing; Intel, Linux and Windows are later work.
- Rust owns domain state, runtime, configuration and control policy. Swift/AppKit owns native macOS UI. Use a private C boundary for Swift; Rust callers use Rust APIs.
- Evaluate Metal/Core Text first and compare wgpu. Final renderer and adapter language remain open.
- Evaluate complete terminal-engine reuse first. `alacritty_terminal` is an experiment candidate, not an accepted production dependency. Preserve the public-v1 protocol/Unicode requirements and native accessibility.
- Use TOML, Rust types and JSON Schema, with one transaction/policy implementation for GUI, CLI and MCP. Use local IPC and an on-demand stdio MCP helper. Approval must survive restart correctly; test external-editor conflicts and crash recovery.
- Keep v1 daemon-free, without CLI/MCP screen-reading or keystroke injection. OS accessibility remains supported. Deliver the scoped CLI/MCP capabilities before public release.
- Treat Alacritty-relative performance as an optimization goal. Establish quantitative gates from reproducible evidence.
- Maintain a small monorepo, MIT original code, dependency notices, canonical human docs, ADRs and GitHub Issues. Use one agent by default; parallel work requires explicit direction and isolated worktrees.
- Start with verified newest stable toolchains/dependencies and exact pins. Do not replace global toolchains or publish a release as part of this approval.

## Alternatives and consequences

An all-Rust native shell, a universal C API, an owned VT engine from the outset, a daemon and simultaneous multi-platform development were considered in the proposal. The approved direction bounds initial maintenance while retaining evidence-based alternatives for engine, rendering and scheduling. It introduces a Rust/Swift lifecycle boundary that must be validated before production work.

The first research slice is deterministic headless replay with independent expected states and a requirement gap matrix. Engine/storage, scheduler, renderer, ABI details, quantitative thresholds and the tested OS floor remain proposed until Phase 0 evidence supports separate ADRs. No benchmark or conformance result is implied by this decision.

## Sources

- [Revised architecture proposal](../../review/ARCHITECTURE.md)
- [Phase 0 plan](../../review/PHASE_0_PLAN.md)
- [Original specification](../../review/terminal_emulator_codex_bootstrap_prompt.md)
