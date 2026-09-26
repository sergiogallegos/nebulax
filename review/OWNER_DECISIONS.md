# Owner review: decisions before bootstrap

Status: revised direction explicitly approved by the owner on 2026-09-26 ("yes i approve"). [ADR 0001](../docs/adr/0001-approved-direction.md) is the current acceptance record. This checklist is retained as review history; statements below about pending approval describe the pre-approval gate. Bootstrap and Phase 0 are authorized. Final experimental choices still require evidence.

The report is [ARCHITECTURE.md](ARCHITECTURE.md). Its recommendations can be accepted as a bundle, with any listed exceptions. No response is treated as approval.

## Review recommendation — 2026-09-26

Accept the direction below with the refinements now incorporated into the proposal and Phase 0 plan. This approves an evidence-gathering approach; it does not select the final engine, renderer or scheduler ahead of the experiments.

| Finding | Refinement prepared for approval |
|---|---|
| Engine reuse must satisfy the product's required behavior | Make Unicode/graphemes, protocol coverage, history resource bounds, and text extraction for selection/accessibility explicit early acceptance checks. Record native support, adapter work, upstream/fork work or unresolved evidence per requirement. No second authoritative grid to conceal a mismatch. |
| File replacement and protected-setting approval need more precise guarantees | Serialize cooperating writers; document the remaining race with arbitrary external editors. Keep protected settings pending across restart, including first import and missing approval records. Approval applies to the resolved protected values and their dependencies. |
| Ten experiments are too broad for the first implementation task | Start with reproducible headless replay and the terminal-engine gap matrix. Stage PTY, native text/FFI, rendering/scheduling and config proof afterward. Keep physical latency measurements separate from instrumented software timing. |

These are design-review findings, not failures observed in a prototype. The original public-v1 protocol matrix, native IME/accessibility and CLI/MCP before public release remain requirements. The latest-stable version verification still happens when implementation begins.

## Must decide before bootstrap

| Decision | Recommendation | Consequence |
|---|---|---|
| Initial platform/support scope | macOS Apple Silicon Tier 1, provisional macOS 14 minimum; Intel deferred | Focuses native input/accessibility and benchmark validation on a testable matrix |
| Core/platform design | Rust domain/runtime/config; Swift AppKit shell; C ABI only for Swift; Metal/Core Text candidates | Accepts a small two-language boundary without freezing future GPU/OS abstractions |
| Config/control design | TOML, Rust types and JSON Schema; one shared transaction/policy implementation; local IPC and an on-demand MCP stdio helper | Approves the format and authority direction; crash recovery, external-editor handling and client compatibility require proof |
| Terminal engine strategy | Reuse-first Phase 0: evaluate stable alacritty_terminal before committing to vte + custom state or a fork | Avoids silently accepting the full VT/Unicode maintenance burden |
| Performance commitment | Alacritty-relative optimization goals; quantitative release thresholds after baseline evidence | No unsupported promise to beat every metric |
| v1/control scope | Daemon-free; no screen read or keystroke injection; protected actions enforced in-app; full agent control in Phase 2 before public release | Preserves the product differentiator with a bounded security surface |
| Repository/ownership model | Small monorepo, MIT original code with preserved dependency notices, canonical human docs, ADRs, Issues and worktrees | No placeholder crate tree or duplicated agent task system |

The owner's newest-stable requirement is already accepted as an instruction: check official stable releases at bootstrap, pin exact versions, and update deliberately. [Evidence recorded on 2026-09-25](TOOLCHAIN_POLICY.md) reported Rust 1.98.1 installed and a stable Xcode release newer than installed Xcode 26.6. Recheck the installed environment and official releases at bootstrap; reconcile setup before Swift implementation.

The owner selected **Nebulax** for the terminal and **`nebulaxterm`** for the CLI on 2026-09-25. See the [naming decision and research](NAMING.md). No indexed matches surfaced for the exact CLI spelling; Nebulax has existing software uses. The GitHub repository is now `sergiogallegos/nebulax`; public package names, bundle IDs and domains remain unreserved. The name is settled; it does not constitute architecture approval.

macOS 14 is a provisional deployment target, not a verified support promise. Intel, Linux and Windows are deferred from the initial implementation. Metal and Core Text are first candidates; their implementation language and the comparison with wgpu stay open through Phase 0.

## Can defer until the end of Phase 0

| Decision | Evidence needed |
|---|---|
| Exact VT engine / need for owned storage | Protocol/gap matrix, adaptation effort, replay correctness, footprint and licenses |
| Metal vs wgpu; renderer adapter language | Matched workload memory/latency/build comparison and lifecycle complexity |
| Core Text cache/shape strategy | Golden/text corpus, native fallback behavior, glyph-to-cell mapping and resource growth |
| Shared reactor vs PTY threads/hybrid | Multi-session fairness, flood latency, memory and wakeups |
| Cell/style/scrollback representation | Full cost including uncommon data, resize, mutation and reclamation |
| Final ABI and native-tab mapping | Ownership/teardown, detach/merge, callback and snapshot evidence |
| Config transaction implementation and schema parity | Round-trip, conflicts, recovery/migration and protected-change proof |
| Quantitative targets and confirmed OS minimum | Controlled competitor baselines and actual deployment testing |

## Can defer until the relevant implementation/release phase

Linux GTK/libadwaita vs raw shell and packaging; Windows support minimum/GPU/installer; updater/Sparkle; signing credentials; domain and website build; optional icons/audit depth; public Rust library stability; graphics, splits, compression, process-preserving daemon and broader automation. Windows 11-first and GTK-first are recommendations, not irrevocable bootstrap commitments.

## Approval boundary and next action

The [original specification](terminal_emulator_codex_bootstrap_prompt.md), section 78, says: “Only after explicit approval should you continue to **Phase D: Repository Bootstrap** and later **Phase 0 implementation research**.” The owner's subsequent Git commands authorized the completed README/Git/remote setup. Architecture acceptance and implementation research remain pending; the later instruction did not approve those technical choices.

After approval: record accepted decisions and any changes; move this review into historical research; write accepted/proposed ADRs accurately; recheck latest stable versions; add the small Cargo research workspace to the existing repository with real build/validation commands; then run the authorized Phase 0 experiments. No release/publication, global toolchain replacement or domain purchase is implied by architecture approval.
