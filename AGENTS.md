# Nebulax project instructions

For continuation, read [HANDOFF.md](HANDOFF.md), [plans/NOW.md](plans/NOW.md), and inspect Git state. The owner approved bootstrap and Phase 0 on 2026-09-26; [ADR 0001](docs/adr/0001-approved-direction.md) records the accepted direction. Do not request that approval again.

Use [the repository map](docs/architecture/REPOSITORY_MAP.md) and [build guide](docs/development/BUILD.md). `scripts/verify` is the required workspace check. `scripts/replay` records strict acceptance evidence; `--allow-known-gaps` tolerates only documented state hashes, never hides failures in reports. Never update expected results simply to make a test pass.

Start each selected toolchain/dependency at verified current stable, pin it, and update deliberately. Latest evidence is in [toolchains](docs/development/TOOLCHAINS.md); earlier version tables are historical.

The owner selected an owned Rust terminal engine with minimal justified dependencies; [ADR 0002](docs/adr/0002-owned-rust-engine-and-dependency-policy.md) supersedes reuse-first. Use Ghostty and other engines as design/behavior references, not planned product libraries. The pinned Ghostty development comparison is approved for isolated research only. Engine internals, renderer, scheduler and tested OS floor still need evidence. Preserve protocol/Unicode, native IME/accessibility and control-authority requirements. PTY bytes and fixtures must never execute config/control or native side effects. Do not build a second authoritative grid to hide engine gaps.

Use one agent by default. Parallel agents require explicit owner direction, separate worktrees/build directories and an interface owner. Serialize performance measurements; correctness replays are not benchmarks.

Human docs are canonical. Record decisions in ADRs and evidence in research; update the handoff when the stage changes. GitHub Issues own actionable status once created; do not duplicate a task database under `.agents`. Preserve the original specification and historical review. Releases, publishing and global toolchain replacement are separate from research approval.

The owner requires repository-size accounting on every code-changing task and whenever asked for current LOC. Before edits, run `scripts/repo-size --json-output target/repo-size/before-task.json`; afterward run it with `--baseline target/repo-size/before-task.json --json-output target/repo-size/current.json`. Include current counts and task deltas in the final progress report. `scripts/verify` also prints counts against HEAD. Use [the counting policy](docs/development/REPO_SIZE.md): physical/nonblank lines with generated, vendored, tests, research and evidence separated. Do not equate LOC with capability or add code merely to reach a size target.
