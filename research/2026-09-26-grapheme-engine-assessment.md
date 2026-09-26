# Grapheme maintenance path and engine comparison — 2026-09-26

Status: expanded Alacritty experiment and source assessment completed; production engine remains undecided. Ghostty and WezTerm observations below are source/documentation findings, not executed comparative results.

**Subsequent owner decision:** [ADR 0002](../docs/adr/0002-owned-rust-engine-and-dependency-policy.md) selects an owned Rust engine with minimal dependencies and approves the pinned Ghostty experiment solely as a behavior/design reference. This resolves the pending exception below and supersedes this assessment's engine-adoption recommendations. The original findings and proposal are retained as the decision's historical context; the subsequent [executed comparison](2026-09-26-ghostty-reference-comparison.md) now records results.

## Outcome

Do not treat the Alacritty grapheme gap as a font-rendering fix or commit to a small adapter-only repair. The expanded suite demonstrates effects on cell ownership, cursor positions, erasure, wrapping and resize. A complete-engine comparison should precede a fork.

Ghostty's development VT library is the most promising next comparison target based on its source and exposed state API. It is **not a stable versioned dependency**, so adopting it even for an isolated executable experiment needs an explicit exception to the owner's newest-stable dependency instruction. A concrete proposed scope is recorded below. No such exception, adoption or production-engine decision has been inferred.

## Executed corpus and results

Preserved the original nine fixture objects and added ten expectations before observing their results: skin-tone modifier, regional-indicator flag, VS16 widening, VS15 text presentation, keycap, combining-mark BS/ECH, ZWJ suffix removal, ZWJ ECH(2), right-edge wrapping and narrow resize. Before running, corrected the representation-level expected WRAPLINE flag to the final continuation cell, following the engine's documented last-cell convention. No expected state was rewritten to match a failed result.

The suite performs **244 replays across 19 fixtures**. All observed operation checkpoints remain equivalent across repeated delivery and tested byte partitions. Eight fixtures meet their independent final expectations. Eleven differences remain: nine grapheme/emoji-policy acceptance gaps and two provisional policy questions (soft-line viewport anchoring and one-cell watch+VS15 text presentation). These counts describe this corpus, not a general conformance score.

| New case | Observed outcome |
|---|---|
| Skin-tone modifier | Base and modifier occupy two wide cells; cursor advances four columns instead of two |
| Regional-indicator flag | Total width is two, but each indicator occupies its own narrow cell instead of one cluster |
| Heart + VS16 | Selector attaches; prior cell remains narrow instead of widening |
| Watch + VS15 | Remains wide; differs from the explicitly proposed one-cell text policy |
| Keycap | Attached scalars remain in one narrow cell instead of two columns |
| Combining mark + BS/ECH | Pass: one-cell erase removes base and attached mark |
| ZWJ + suffix + BS/EL | Suffix disappears but the cursor and separate component cells retain the wider layout |
| ZWJ + CR/ECH(2) | Woman component is removed; laptop remains at column 2 |
| ZWJ at right edge | Laptop wraps independently from woman+ZWJ |
| ZWJ narrow resize | Incorrect four-column cluster layout interacts with viewport anchoring and moves text into history |

Unicode grapheme segmentation and terminal column width are distinct. The corpus makes its intended emoji/text widths explicit; in particular, VS15 narrowing is a proposed policy rather than a consequence guaranteed by UAX #29. Sources: [Unicode segmentation](https://www.unicode.org/reports/tr29/) and [emoji presentation](https://www.unicode.org/reports/tr51/). Native shaping and application width conventions still require separate compatibility testing.

Raw evidence: [replay](../benchmarks/results/p0-06-graphemes/replay.json), [environment](../benchmarks/results/p0-06-graphemes/environment.json), [source hashes](../benchmarks/results/p0-06-graphemes/sources.json), [artifact hashes](../benchmarks/results/p0-06-graphemes/artifacts.json). Original evidence remains in `p0-06-initial`. Known-gap tolerance records the exact reviewed failing-state hash; strict replay continues to exit 1. Seven harness unit/integration tests and the workspace verification pass with explicit gap tolerance.

## Why an Alacritty patch is more than a renderer adapter

The pinned `alacritty_terminal` 0.26.0 source inserts one `char` at a time using `UnicodeWidthChar::width`. Zero-width scalars are appended to prior-cell extras. Positive-width components get separate cells. The locked `unicode-width` 0.2.2 library has sequence-aware string-width logic, but the terminal insertion path does not use it. Updating a width table or glyph shaping alone does not establish correct engine state. [Pinned insertion source](https://docs.rs/alacritty_terminal/0.26.0/src/alacritty_terminal/term/mod.rs.html)

Inference from the source and fixtures: a robust change must address incremental grapheme extension across reads, width expansion/contraction after a selector, wide-cell repair at a row edge, wrapping/reflow, edits and selection/text extraction, reset/control boundaries, and bounded cluster storage. `Cell` already has extra scalar storage, but that does not prove all consumers tolerate positive-width components there. This needs an engine-level change and a maintained regression corpus. No patch-size or calendar estimate is supported yet.

The upstream [emoji-modifier issue](https://github.com/alacritty/alacritty/issues/3975) remains open on the check date. That corroborates an unresolved feature area, but neither promises upstream acceptance nor proves that every related issue remains unfixed. Keep Alacritty as a reproducible baseline; do not start an unbounded fork or a second authoritative grid.

## Alternative source assessment

| Candidate | Verified entry point and constraints | Disposition |
|---|---|---|
| Ghostty stable 1.3.1, commit `332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28` | Zig terminal code handles cluster extension and variation selectors. The standalone VT C exports at this tag cover parsers/input helpers, without a full terminal-state C interface. Its build guard requires the 0.15.x Zig minor series, at least 0.15.2; installed/current stable Zig is 0.16.0. | Plausible core, but requires a Zig-facing adapter and a compiler-policy exception or port. A stable app tag does not establish a stable embedding ABI. Not built here. |
| Ghostty development commit `6301810a48aaa3426887a4316668f18833a40138` | Version `1.3.2-dev`, minimum Zig 0.16.0; exposes terminal creation/feed/state APIs in `terminal.h`. Header and README still describe an unstable, untagged VT library. | Best next **isolated comparison** candidate if the owner permits the explicit dependency exception. No dependency added or development code executed. |
| WezTerm latest stable application tag `20240203-110809-5046fc22`, commit `5046fc225992db6ba2ef8812743fadfdfe4b184a` | `wezterm-term` is an embeddable Rust terminal with `advance_bytes` and caller-provided output. Its manifest includes sibling path crates, image support and a broader dependency set. The direct crates.io `wezterm-term` query returned 404 on this date. | Source-pinned integration is possible to investigate, not a verified drop-in registry dependency. The stable source has a feed-boundary segmentation concern described below. Not built here. |
| `vte` plus owned state | Parser reuse leaves terminal storage, modes, Unicode, editing and reflow under our ownership. | Retain as fallback after complete-engine options; not a shortcut around the observed maintenance work. |

Ghostty source evidence: [stable C header](https://github.com/ghostty-org/ghostty/blob/332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28/include/ghostty/vt.h), [stable terminal](https://github.com/ghostty-org/ghostty/blob/332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28/src/terminal/Terminal.zig), [compiler guard](https://github.com/ghostty-org/ghostty/blob/332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28/src/build/zig.zig), [development API](https://github.com/ghostty-org/ghostty/blob/6301810a48aaa3426887a4316668f18833a40138/include/ghostty/vt/terminal.h), [development manifest](https://github.com/ghostty-org/ghostty/blob/6301810a48aaa3426887a4316668f18833a40138/build.zig.zon). Ghostty documents `grapheme-width-method` and mode 2027; a comparison must explicitly enable the intended mode rather than assume the GUI default carries into library initialization. [Configuration reference](https://ghostty.org/docs/config/reference#grapheme-width-method)

In WezTerm's inspected stable source, each `advance_bytes` creates a performer; dropping it flushes its printable buffer. The flush segments that buffer into graphemes and skips zero-width graphemes. **Inference, not an executed result:** a cluster spanning successive feed calls may be split or lose a separately delivered combining suffix, so a favorable whole-string demonstration is insufficient. The same chunk-splitting corpus must test this before selecting it. [Terminal entry point](https://github.com/wezterm/wezterm/blob/5046fc225992db6ba2ef8812743fadfdfe4b184a/term/src/terminal.rs), [performer](https://github.com/wezterm/wezterm/blob/5046fc225992db6ba2ef8812743fadfdfe4b184a/term/src/terminalstate/performer.rs), [manifest](https://github.com/wezterm/wezterm/blob/5046fc225992db6ba2ef8812743fadfdfe4b184a/term/Cargo.toml).

URLs, checked revisions and downloaded-source SHA-256 values are in [the source manifest](evidence/2026-09-26-engine-source-manifest.json). Third-party source was inspected in temporary storage, not copied into project source. Neither competitor's documentation substitutes for local execution or a release license audit.

## Proposed next experiment — explicit exception pending

If authorized, use **only commit `6301810a48aaa3426887a4316668f18833a40138` of Ghostty for a disposable headless comparison**, with current stable Zig 0.16.0. Keep it outside the production Rust workspace/dependency graph. Verify downloaded source and dependency hashes and record toolchain/build configuration. Implement only create/feed/resize/read/free and normalized text/cursor/width observations; execute no PTY, clipboard, shell or UI effects.

Run the same 19 fixtures and byte-delivery variants, with grapheme mode/config explicit. Separate policy differences from unmet requirements; preserve both expected and observed state. Stop after comparable results, lifecycle/ownership observations and a dependency/license inventory. Do not silently follow branch updates, patch around every failure, fork the core or turn a passing small corpus into production selection. Production adoption remains a later ADR with protocol/resource/text-access and release-stability evidence.

This exception would not waive stable Rust/Swift/Zig toolchains, change the approved Rust/Swift application boundary, replace global tools or approve a release. If it is declined, remain on stable dependencies and investigate WezTerm's latest stable source or additional stable candidates with the same acceptance corpus. No policy change is assumed from permission to continue Phase 0 generally.
