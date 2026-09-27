# Repository size accounting

The owner requested current LOC reporting for every code-changing task and whenever asked. Expected eventual scale is roughly 200–250k lines for the terminal core, 25k+ per native frontend, and around 30k for C-facing integration. These are planning expectations; capability, correctness and maintainability determine necessary implementation size. Swift/macOS is current; C#/Windows and C++/Linux are future frontend expectations, not existing implementations.

Run before editing:

```sh
scripts/repo-size --json-output target/repo-size/before-task.json
```

Run afterward to report current totals and task deltas:

```sh
scripts/repo-size --baseline target/repo-size/before-task.json --json-output target/repo-size/current.json
```

Without `--baseline`, deltas compare the working tree with HEAD. `scripts/verify` runs this default report and two tooling tests. The JSON contains per-file category, language, physical/nonblank lines, bytes and binary-file status, plus current/baseline totals. Outputs under ignored `target/` do not count themselves. Reports can be kept outside the repository for longer-term comparisons. A commit comparison includes all uncommitted work, not necessarily only the latest task; label it correctly.

## Metric and scope

Count LF-delimited physical lines, including blank and comment lines, plus nonblank lines as a separate column. An unterminated last line counts once. These are reproducible file-size metrics, **not parser-derived executable SLOC**. Inline unit tests and GUI self-test helpers remain within their source-file counts; dedicated test files are separate. No claim is made that every line under a prototype represents production implementation.

Scope is existing tracked files plus nonignored untracked regular files. Missing/deleted files contribute no current lines. Symlinks are not followed or counted. Ignored builds/caches, `.git`, installed dependencies and ignored reference-engine checkouts are excluded. Binary or non-UTF-8 files count toward bytes/files, with zero text lines.

Categories are disjoint: handwritten Rust core, prototype Rust PTY/runtime, Rust native/FFI bridge, C ABI headers, C implementation, each platform frontend, Python tooling/peers, other scripts, research/reference code, dedicated tests, generated source, vendored code/data/licenses, documentation/licenses, configuration/lockfiles, fixtures/patches and retained evidence. Evidence includes text logs/manifests and binary images; it is never added to product code. Generated Unicode tables are separate from handwritten Rust. Python test fixtures in dedicated test directories count as tests, with Python retained in their per-file language field.

The C-facing API is called the **C ABI / FFI boundary**. A Rust export layer and C header do not imply a large implementation written in C. Current C tests and the isolated Ghostty C adapter are counted under tests/research, respectively. Windows/Linux categories stay at zero until source exists. Future platform roots are `apps/macos`, `apps/windows`, and `apps/linux`; update the classifier when actual layout is chosen. New areas or generated files must be classified deliberately; review the `Other files` row and bump the JSON schema if category compatibility changes.

In final progress reports include the product-area counts, tests/tooling/support totals and task deltas. On an explicit LOC request, print the full category summary. Avoid comparing the repository total (which includes documentation, vendor data and evidence) with a handwritten-core target.
