# ANSI insert mode — 2026-09-27

[ADR 0022](../docs/adr/0022-ansi-insert-mode.md) adds global ANSI mode 4, state-derived queries and reset semantics. Printing now shares the existing screen-owned column mover with ICH/DCH. New graphemes insert their display width; extensions adjust only width changes, preserve surviving owners/styles and cannot recover previously clipped content. No dependency, toolchain or private C ABI changed.

## Validation

Ten added core tests cover every two-way byte split and byte-at-a-time delivery of supported example streams; 462 independently mapped narrow/wide insertion cases across widths 2–12; combining/selector/ZWJ changes; wrap/clamp, incoming soft wraps and reflow; scroll-region/history isolation; global/saved/screen/reset ownership; atomic malformed lists; mode-only snapshot stability; styles, row-buffer reuse, evicted cluster release and immutable held frames; cluster limits; and 200 query-time mode replies under output pressure. Existing mixed-stream invariant coverage now also interleaves IRM, alternate exits, autowrap changes and text-presentation selectors.

The new macOS worker test runs a raw PTY peer that inserts a widening heart into an existing row, waits for exact IRM/CPR replies, checks replacement after soft reset and verifies hard reset from alternate screen. It retains prior snapshots across the changes and worker teardown. Mode-4 startup reports changed from unrecognized to recognized/reset; the negative ANSI mode probe uses still-unsupported mode 20. Existing replay fixture expectations are unchanged.

The AppKit probe inserts `R` into `IM OK`, waits for exact set/reset replies, then acknowledges its earlier synthetic paste. Swift checks every `IRM OK` cell. The earlier hidden/visible cursor comparison remains isolated; `paste.png` captures both final markers. System clipboard, input APIs and native frame layout are unchanged.

## Retained evidence

- [Boundary run](../benchmarks/results/p0-21-insert-boundary/summary.json): 187 engine/runtime/bridge Rust tests plus compiled C/Swift execution, raw logs and source/artifact hashes.
- [Native run](../benchmarks/results/p0-21-insert-native/window.json): eight native key events, one explicit paste, inserted marker/mode replies, prior rendering/protocol checks, resize to 90×25 and asynchronous child cleanup.

Full `scripts/verify` passes 202 Rust tests, Unicode regeneration/data checks, Clippy/formatting, SDK ABI and owned dependency guards, seven Python cases (six pass, one optional live-Ghostty skip), all 19 owned fixtures/244 replays and the recorded Alacritty gap fingerprints. C/Swift execute and AppKit compiles in verification; the GUI self-test is separate. The configured portable Rust set has 178 tests; macOS adds 24. The PTY package has 33 tests (21 macOS-specific, 12 portable). These are correctness checks, not performance measurements or full TUI acceptance.

## Next boundary

Protocol work remains open, but the next selected task returns to the native rendering risk: define the bounded text-run/cell-mapping contract and exercise Core Text shaping with ligatures in a focused experiment before the direct Metal integration. The core remains headless and authoritative; shaping must consume immutable owned inputs and never hold the terminal lock. The broader native IME/accessibility, clipboard/menu, GPU atlas/lifecycle and production scheduling gates remain open.
