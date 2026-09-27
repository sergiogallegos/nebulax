# Tabs and line/display erasure

Date: 2026-09-27. Builds on style checkpoint `0fceb55`, committed and pushed at the owner's request before starting this slice. [ADR 0013](../docs/adr/0013-tabs-and-erasure.md) defines the protocol subset and state policies.

Implemented terminal-wide tab stops with HT/CHT/CBT movement, HTS/TBC editing, bounded bitmap storage and explicit shrink/grow behavior. Added EL 0–2 and ED 0–3 with inclusive ranges, current-background erasure, wide-owner repair, wrap-boundary detachment and active-primary history clearing. Core cells remain 16 bytes; private ABI v2, dependencies, toolchain pins and original replay fixtures are unchanged.

## Validation

`scripts/verify` passes **138 Rust tests** on macOS, Unicode regeneration/hash checks, formatting, warning-denied Clippy, Python/reference checks, strict owned replay and real C/Swift callers. The unchanged owned corpus matches all 19 fixtures under 244 delivery variants. The configured portable Rust subset is 121 tests; hosted CI was not inspected.

Twelve new core tests use independent expected rows/cursors and delivery partitions. Coverage includes the maximum geometry, bitmap bounds after cloning/resizing, malformed commands, wide continuation edits, background-only styles, history/alternate isolation and logical wrap boundaries. A new real-PTY shell test brings the PTY package to 23 tests. Its first draft sent a literal wide character into a locale-free interactive shell; Readline did not preserve those input bytes. The corrected fixture sends an ASCII command whose `printf` emits the intended UTF-8 bytes with octal escapes. Expected terminal cell positions were retained. Output processing is disabled so HT reaches the engine unchanged.

The [boundary recording](../benchmarks/results/p0-14-tabs-boundary/summary.json) retains 123 relevant Rust tests plus C/Swift checks, raw command logs, source hashes and native binary hashes. The [GUI recording](../benchmarks/results/p0-14-tabs-native/window.json) verifies HT and counted CHT at columns 9 and 17 after full-line erasure and resize to 90 × 25. Startup uses ED 2 to clear stale content. Existing Unicode/style, normal/application arrows, regions/replies and asynchronous-close checks remain. The [captured grid](../benchmarks/results/p0-14-tabs-native/window.png) is correctness evidence, not a latency or rendering benchmark.

## Limits and next work

Stops are retained to the greatest allocated width, capped at 8 KiB; clear-all disables future default stops. Normal erasure has no protected-cell behavior. ED 3 deliberately leaves hidden primary history intact while alternate is active. No terminfo capability profile or broad shell/TUI compatibility claim follows from the clean `TERM=dumb` shell case.

Next: insert/delete characters and lines using existing wide-owner invariants, region boundaries and background policy. Native IME/accessibility, production scheduling, job-tree lifecycle and performance remain separate acceptance work.
