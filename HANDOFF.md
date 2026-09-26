# Nebulax session handoff

Updated: 2026-09-26, after the owner-provided Opus review and authorization to checkpoint all progress on main.

## Resume here

Read this file, [plans/NOW.md](plans/NOW.md), [ADR 0001](docs/adr/0001-approved-direction.md), and inspect Git state. The owner explicitly approved the revised architecture direction and proceeding with Phase D/0 ("yes i approve"). **Do not ask for architecture/bootstrap/Phase 0 approval again.** [ADR 0002](docs/adr/0002-owned-rust-engine-and-dependency-policy.md) supersedes reuse-first: build an owned Rust engine with minimal justified dependencies. See [architecture status](docs/architecture/STATUS.md) for accepted boundaries versus open details. Engine compatibility, renderer, scheduler and tested OS floor still need evidence.

## Confirmed owner decisions

- App: **Nebulax**. Planned CLI: **`nebulaxterm`**. Directory: `/Users/sergiogallegos/projects/nebulax`.
- GitHub: [sergiogallegos/nebulax](https://github.com/sergiogallegos/nebulax); remote `git@github.com:sergiogallegos/nebulax.git`.
- macOS Apple Silicon first; Rust core + Swift/AppKit, private C boundary; evaluate Metal/Core Text and wgpu; owned Rust terminal engine with minimal justified dependencies. Existing engines are behavior/design references, not planned product libraries.
- TOML/schema with shared config/control policy; daemon-free v1; narrow CLI/MCP before public release; no arbitrary agent screen reads or keystroke injection. Native accessibility remains required.
- Small MIT-original-code monorepo, canonical human docs/ADRs/Issues. Verified newest stable versions at implementation start; exact pins and deliberate updates.
- Foundational crates such as Tokio are acceptable when justified, not automatically selected. Prefer focused owned code where practical and assess transitive costs. The pinned Ghostty development comparison is approved only as isolated research, with no product adoption.
- Performance comparisons remain evidence-driven goals. No permission for global toolchain replacement or public release is implied.

## Completed work

- Research and the revised proposal are preserved in `review/` as historical documents, keeping original links and the unchanged original specification. ADR 0001 records current acceptance.
- Created a real Cargo research workspace with one package, `nebulax-replay`, plus fixtures, verification/recording scripts, build/toolchain guides, governance/contribution/security/license files, repository map and a pinned CI workflow. No empty production crate hierarchy.
- Verified Rust/Cargo 1.98.1 and Python 3.14.7 against official current stable releases. Pinned Rust and CI Python. Queried crates.io for latest non-yanked stable dependencies: alacritty_terminal 0.26.0, serde 1.0.229, serde_json 1.0.151, sha2 0.11.0. `Cargo.lock` preserves resolved versions/checksums.
- Implemented nine synthetic expected-state fixtures with whole-feed, repeated, fixed-size chunks and every interior two-way split index: **125 replays** per suite. All observed operation checkpoints are chunk-equivalent. Seven fixtures match all expected final states.
- Preserved two explicit observations: a required emoji ZWJ cell-width gap and a provisional resize viewport-policy difference. Strict replay exits 1; `--allow-known-gaps` tolerates only reviewed failing-state fingerprints. Changed states, unexpected passes or chunk mismatches fail verification.
- Saved reproducible raw JSON, source/dependency/executable hashes and environment metadata in [benchmarks/results/p0-06-initial](benchmarks/results/p0-06-initial). [Research findings](research/2026-09-26-terminal-replay.md) and the [gap matrix](research/ENGINE_GAPS.md) distinguish observed behavior from pending checks.
- Extended the corpus to **19 fixtures / 244 replays**, preserving the original nine fixtures and initial evidence. Eight expectations match; eleven reviewed differences remain (nine grapheme/emoji-policy gaps and two provisional policy questions). All tested delivery partitions remain equivalent at observed checkpoints. Added skin-tone, flag, variation-selector, keycap, erase, edge-wrap and narrow-resize cases. Retained the expanded evidence in [benchmarks/results/p0-06-graphemes](benchmarks/results/p0-06-graphemes).
- Completed a [grapheme maintenance and alternative-engine source assessment](research/2026-09-26-grapheme-engine-assessment.md), with exact source revisions and hashes. The observed gap affects engine state and editing, not just rendering. At that assessment stage, Ghostty and WezTerm were inspected only; the subsequent Ghostty execution is recorded below. WezTerm remains unexecuted. The owner subsequently approved the pinned Ghostty comparison as a research reference and selected an owned Rust implementation; ADR 0002 records this change.

- Built the approved Ghostty revision with stable Zig 0.16.0, without upstream patches, under ignored `target/ghostty-reference`. The original C adapter and standard-library Python tooling live in `experiments/ghostty-reference`; no Cargo dependency changed. Verified all 5,901 regular source files against the pinned archive and captured fetched-package/license-file hashes.
- Executed **19 fixtures / 244 replays** with explicit grapheme mode: **19 matches, zero differences**, all observed operation checkpoints delivery-equivalent. A separate mode-off control performs another 244 replays: nine matches and ten emoji differences, also delivery-equivalent. Saved [primary evidence](benchmarks/results/p0-06-ghostty) and [control evidence](benchmarks/results/p0-06-ghostty-legacy), plus the [comparison findings](research/2026-09-26-ghostty-reference-comparison.md).
- Defined the [first owned Rust engine slice](docs/architecture/OWNED_ENGINE_SLICE.md): explicit grapheme ownership, incremental decoding, width transitions and cluster-aware editing with bounded resources. The next session implemented this bounded design as recorded below.

- Implemented `crates/terminal` (`nebulax-terminal`) with **zero Cargo dependencies**, no unsafe code and no I/O. It owns incremental UTF-8, bounded parsing, fixed primary-grid state, grapheme clusters, width transitions, wrapping and basic editing. No Ghostty/Alacritty algorithm was copied into the core.
- Verified final Unicode **18.0.0** and UAX #29 revision 49. Added unmodified official data/test files and Unicode License V3, a hash manifest, deterministic offline generator and original constant-space segmentation. All **853 official grapheme tests** pass. The core declares MIT AND Unicode-3.0 for original code plus generated data.
- Added `experiments/owned-replay` and `scripts/replay --engine owned`: **14 matching fixtures / 166 replays**, zero observed checkpoint-delivery differences; **five fixtures explicitly pending** for history, alternate screen and resize. Retained [owned-slice evidence](benchmarks/results/p0-06-owned-slice) and [findings/policies](research/2026-09-26-owned-engine-slice.md). Original corpus and reference artifacts are unchanged.

- Implemented bounded primary history, independent alternate screen (DEC 1049), primary reflow and alternate crop/pad with zero new dependencies. All **19 fixtures / 244 replays** now pass with no deferrals or changed expectations. Added 14 independent integration tests. See [policies and the explicit primary resize limitation](docs/adr/0003-history-screen-and-reflow.md), [findings](research/2026-09-26-history-screen-reflow.md) and [retained evidence](benchmarks/results/p0-06-owned-history).

## Verification

Current `scripts/verify` passes Unicode input/hash/regeneration checks, formatting, Clippy with warnings denied, **36 Rust tests**, two Python tests (one optional live Ghostty test skipped), recorded Alacritty replay with its exact known-gap fingerprints and strict all-19 owned-engine replay. The core tests include 853 official segmentation cases, 8,000 malformed UTF-8 triples, streamed edit/edge cases, bounded cluster/parser behavior, mixed-stream invariants and inline ASCII storage. Source/artifact hash and current-document local-link checks also pass. No new third-party Cargo dependency was added.

A separate source snapshot with a fresh Cargo target directory passed `scripts/verify` for the initial nine-fixture slice using cached pinned dependencies with Cargo offline; its normalized report matched the initial evidence. After the Xcode update, workspace verification passed again with the expanded 19-fixture corpus. Swift AppKit/Core Text/Metal imports also type-check with the updated toolchain. Hosted CI is configured but has not run. No benchmark results or complete emulator-conformance claim have been made.

## Current stage and next step

Phase D bootstrap, reference comparisons and the owned grapheme/history/alternate/reflow slices are implemented. Architecture direction is accepted; Phase 0A validation remains open. The owner provided an Opus review highlighting integration risks. Source review confirmed its main findings; [the assessment](docs/architecture/FOUNDATION_REVIEW.md) records qualifications and revises the implementation order. Foundations and an early integrated native path now precede broad SGR/protocol expansion. [NOW](plans/NOW.md) is the current-task source.

Current resize still rejects when populated primary cells below the cursor would be lost, including inactive primary state. This is an integration blocker to resolve, not finished window behavior. Existing Phase 0 and owned-engine approvals remain valid; do not request them again.

Remaining Phase 0 work includes full protocol/resource/text-access acceptance, competitor baselines, PTY/lifecycle, native text/FFI, renderer/scheduler and config/approval proof. Follow the staged [Phase 0 plan](review/PHASE_0_PLAN.md).

## Environment and Git

Host reports macOS 27.0 (26A428), arm64. The owner updated Xcode: verified **Xcode 27.0 (27A266a), Apple Swift 6.4 (`swiftlang-6.4.0.34.1`), macOS SDK 27.0 (26A425)**. Initial license/first-launch setup blocked compilation but is now resolved; Rust verification and the native import smoke check succeeded. No global toolchain was changed or installed by this session.

The owner explicitly requested adding, committing and pushing all progress to `main`. Repository history starts with README-only `67a7237`; use `git log` and `git status -sb` for the implementation checkpoint and remote synchronization state. Earlier retained result metadata truthfully records the pre-checkpoint dirty source snapshots; preserve it. GitHub issues have not been created; `plans/NOW.md` remains the temporary current-task source. Hosted CI status should be checked separately from local validation.

There is no terminal application, production PTY, GUI, control server, website, release or background process to restore. Normal replay outputs and builds are under ignored `target/`; six retained evidence sets are under `benchmarks/`. The ignored Ghostty reference build/cache is disposable and can be reproduced using its experiment README.
