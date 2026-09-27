# Terminal reset through the native path

Date: 2026-09-27. Builds on local device-reply work after pushed checkpoint `c069a6b`; reply/reset changes remain uncommitted. [ADR 0019](../docs/adr/0019-soft-and-hard-terminal-reset.md) defines the exact reset profile.

Implemented soft reset (`CSI ! p`) and hard reset (`ESC c`) in original Rust using only std. Screen owns mode/saved-state initialization and row clearing; Terminal coordinates shared modes, buffer selection, decoding, tabs and style storage. No manifest, dependency, toolchain or ABI changed. Existing PTY libc and isolated research dependencies remain explicitly recorded gaps/exceptions in the current workspace inventory, not additions by this task or a claim of a fully dependency-free workspace.

## Evidence

`scripts/verify` passes **180 Rust tests** on macOS, Unicode data/hash/regeneration checks, formatting, warning-denied Clippy, four Python tests (one optional Ghostty test skipped), C/Swift boundary checks and AppKit compilation. The configured portable set is 159 Rust tests; hosted CI was not inspected. The unchanged owned corpus still matches all **19 fixtures / 244 replays**; reference expectations and known-gap fingerprints remain unchanged.

Ten new core tests cover active/hidden state, preserved content/history/styles, saved-state defaults, pending wrap, tabs and future growth, custom host limits/width policy, discarded resources and retained snapshots, output order/pressure, malformed/string-contained reset bytes, every split point, UTF-8/grapheme boundaries and repeated resets over widths 2–10 and heights 1–4. Hard reset returns to independently constructed fresh state with the same host options. Soft-reset metadata publication retains row versions when cells are unchanged.

The PTY package now has **27 tests**: 18 macOS-specific and nine portable. Its new peer waits for a native acknowledgement before soft reset, queries restored state, requires a normal-mode Up sequence, then dirties history/alternate/modes/tabs and sends RIS between queries. The worker observes unchanged rows after soft reset and a blank primary/default palette after hard reset. Exact old/new replies remain ordered; held frames retain original styles and visibility after worker destruction. The child exits cleanly with `RESET OK` and no denied effects.

The [boundary recording](../benchmarks/results/p0-18-reset-boundary/summary.json) retains **165 relevant Rust tests** plus C/Swift lifetime/layout checks. The [GUI recording](../benchmarks/results/p0-18-reset-native/window.json) first creates stale history and altered modes/tabs in both screens, validates soft-reset coordinates/saved-state/modes, validates hard-reset primary/default tabs, then displays `STARTUP OK / RESET OK`. It verifies default rendition and absence of stale text along with all previous seven native events, styles, tab positions, editing, no-wrap behavior, resize, cursor-only pixel changes and asynchronous cleanup. The [hidden](../benchmarks/results/p0-18-reset-native/hidden.png) and [visible](../benchmarks/results/p0-18-reset-native/window.png) captures were visually inspected. Artifact hashes and current source manifests were checked; earlier evidence remains unchanged.

## Limits and next work

Autowrap-on after soft reset, hidden-primary preservation under soft reset and history clearing under RIS are explicit Nebulax policies. This synthetic reset/startup probe is not a full TUI or hardware-conformance test. The shell preview still uses `TERM=dumb`. No native reset menu/public reset API, renderer performance result or complete SDK is implied.

Next replace the PTY libc crate with a small owned platform boundary verified against SDK C declarations, retaining lifecycle, signal, resize and cleanup coverage. This follows the owner's std/core-only direction before promoting session code into the reusable library. Broader input protocols, insert mode, native text/accessibility and Metal/ligatures remain subsequent work.
