# Nebulax architecture review

Date: 2026-09-25  
Status: historical research/proposal archive. The owner approved the revised direction and bootstrap/Phase 0 on 2026-09-26; [ADR 0001](../docs/adr/0001-approved-direction.md) records acceptance. [The current plan](../plans/NOW.md) supersedes review-era status statements below. Original paths are retained to preserve links.

This directory contains the requested design deliverables for **Nebulax**, with **`nebulaxterm`** as its CLI. The project directory is `/Users/sergiogallegos/projects/nebulax`, and its GitHub repository is [sergiogallegos/nebulax](https://github.com/sergiogallegos/nebulax). At the owner's subsequent instruction, Git was initialized and the root README was pushed as commit `67a7237` on `main`. There is no application, Cargo workspace or accepted architecture yet. See [the handoff](../HANDOFF.md) for the current continuation state; these review documents remain local and untracked.

Read in this order:

1. [Owner decisions](OWNER_DECISIONS.md) — review findings, recommended decisions and approval gate.
2. [Architecture proposal](ARCHITECTURE.md) — all 21 requested Phase B sections.
3. [External research](EXTERNAL_RESEARCH.md) — inspected repositories, evidence, and limitations.
4. [Toolchain policy](TOOLCHAIN_POLICY.md) — the owner's newest-stable requirement and this Mac's observed environment.
5. [Phase 0 plan](PHASE_0_PLAN.md) — experiments, measurement protocol, and decision criteria; not executed.
6. [Original bootstrap specification](terminal_emulator_codex_bootstrap_prompt.md) — preserved unchanged from Downloads.

The owner selected **Nebulax / `nebulaxterm`** on 2026-09-25. The [naming decision and research](NAMING.md) records the choice, earlier candidates and search limitations.

Architecture recommendations remain **proposed**; the name and newest-stable toolchain policy are owner instructions. Numerical memory examples are allocation models, not measured product performance. No benchmark results have been fabricated or inferred from another project's marketing.

After approval, preserve this review package as historical research, record accepted decisions in ADRs, and bootstrap only the files that have a present purpose. Recheck stable versions at that time. Phase 0 results must confirm or supersede the relevant architecture choices before production implementation.
