# Contributing

Start with the [current plan](plans/NOW.md) and [build guide](docs/development/BUILD.md). This is an experimental workspace, not a shipped terminal.

Use focused branches and PRs describing the problem, resulting behavior and validation. Link the relevant issue and ADR. Run `scripts/verify`; for a behavior change, add an independent expected-state fixture. A known gap requires an explanation and reviewed observed-state hash. Do not copy engine output into an expectation and call it conformance.

Changes to ownership, language boundaries, engine, renderer, threading, persisted formats, authority or support promises need an ADR. Performance claims require serialized controlled runs, raw artifacts, units, environment and limitations. Headless correctness replay is not a throughput or latency benchmark.

Do not include personal terminal data, credentials or machine paths in fixtures or public artifacts. Contributions are licensed under the project's MIT license unless an explicitly identified third-party license applies. Preserve provenance and dependency notices. Follow the [code of conduct](CODE_OF_CONDUCT.md).
