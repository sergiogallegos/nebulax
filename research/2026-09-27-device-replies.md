# Device and mode replies through application startup

Date: 2026-09-27. Builds on pushed checkpoint `c069a6b`, which includes editing, autowrap/visibility and Files/LOC reporting. This reply slice is local and uncommitted. [ADR 0016](../docs/adr/0016-device-and-mode-replies.md) specifies exact bytes and compatibility limits.

Added original Rust recognition/reply construction for DA1/DECID, DA2, XTVERSION and DECRQM, alongside existing DSR status/cursor reports. Identification uses a minimal legacy family profile with no options and an explicit Nebulax experimental version string. Mode reports derive from implemented state; unsupported modes are unrecognized. No dependency, toolchain or ABI change; no native side effect or new output channel.

## Validation

`scripts/verify` passes **169 Rust tests** on macOS, Unicode hash/regeneration checks, formatting, warning-denied Clippy, four Python tests (one optional Ghostty test skipped), reference/owned replays, compiled C/Swift checks and AppKit compilation. The configured portable set is 149 Rust tests; hosted CI was not inspected. Original fixture expectations and pinned dependencies remain unchanged: all 19 owned fixtures / 244 replays match.

Eight new core tests cover exact query/reply bytes, every two-way split and byte-at-a-time delivery, state-derived modes, screen/save/resize transitions, origin coordinates, pending-wrap preservation, malformed/cancelled/over-limit requests, echoed replies and 400 replies through repeated bounded-queue pressure. The first blocked offset is checked exactly; repeated feed while blocked consumes zero and preserves state. Existing mixed reply/title/bell and payload-pressure tests still pass.

The PTY package now has **26 tests** (17 macOS-specific and nine portable). A controlled Python peer sets raw mode, waits for exact device/version/status/cursor replies, probes supported and unsupported modes, enters alternate state, changes modes, verifies origin-relative coordinates and restores primary state before printing `START OK`. The worker asserts clean child exit, no denied effects, exact visible text and restored cursor visibility. This tests query-only progress without needing display changes to trigger writes.

The [boundary recording](../benchmarks/results/p0-17-replies-boundary/summary.json) retains **154 relevant Rust tests**, C/Swift lifetime/layout checks and source/toolchain/binary hashes. The [native recording](../benchmarks/results/p0-17-replies-native/window.json) verifies a second exact startup exchange before the peer displays `STARTUP OK`. Existing seven native events, normal/application arrows, styles, tabs, editing, no-wrap output, resize, cursor-pixel comparison and child cleanup remain asserted. [Hidden](../benchmarks/results/p0-17-replies-native/hidden.png) and [visible](../benchmarks/results/p0-17-replies-native/window.png) captures were visually inspected. Sources remained unchanged during each recording; artifact hashes were checked.

## Limits

This is a synthetic startup protocol test, not an execution of a production TUI or a terminal-conformance claim. DA1/DA2's legacy identifiers are explicitly experimental; higher-level capability bits are not advertised. The clean Bash preview remains `TERM=dumb`. Terminal reset, insert mode, broader keyboard negotiation, production lifecycle/rendering performance and native IME/accessibility remain open. Reset semantics are the next bounded protocol slice.
