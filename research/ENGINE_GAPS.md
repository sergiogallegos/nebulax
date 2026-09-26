# Engine acceptance matrix

Candidate: `alacritty_terminal` 0.26.0. Status: experimental, not selected. Updated 2026-09-26.

The expanded corpus has 19 fixtures / 244 replays: eight expectations match, eleven differences remain. The [grapheme assessment](2026-09-26-grapheme-engine-assessment.md) distinguishes nine grapheme/emoji-policy acceptance gaps from two provisional policy questions.

“Observed” below means only the linked fixtures, not complete feature conformance. “Pending” means no executable evidence in this workspace yet. A documented library API is not proof that integration meets the requirement.

| Required area | Evidence / classification | Next evidence |
|---|---|---|
| xterm-compatible core | Selected cursor-left/erase scenario observed | Broader modes, margins, edits, save/restore, replies and conformance corpus |
| TERM / terminfo / fallback | Pending | Real advertised capability and remote fallback tests |
| 24-bit color | Pending | SGR forms, resets, palette changes and expected cell colors |
| Styled/colored underlines, undercurl | Pending | State fixtures and native rendering goldens |
| DEC 2026 synchronized output | Pending | Timed release, reset, disconnect and buffered-update behavior |
| Kitty keyboard | Pending | Negotiation/mode stack and actual input encoding integration |
| Bracketed paste / focus / SGR mouse | Pending | Mode-aware input encoder, cancellation, coordinates and lifecycle |
| Alternate screen | One restore sequence observed | Resize, selection, saved state, reset and history semantics |
| OSC 8 | Pending | Identity, bounds, reset and safe activation policy |
| OSC 52 | Pending | Denied reads, bounded writes, prompt/policy integration |
| OSC 7 / OSC 133 | Pending | Typed events, ordering, untrusted metadata and host/path handling |
| Unicode width / CJK | One multibyte and edge-wrap case observed | Ambiguous width, selectors, Unicode data policy and application disagreement |
| Combining marks | Extension across feeds and BS/ECH removal observed | Broader edits and pathological sequence limits |
| Emoji ZWJ / graphemes | Ownership/width/editing/wrapping gaps observed in the expanded corpus | Use reference comparisons to inform the owned Rust engine; see ADR 0002 |
| Resize / reflow | Hard-line case observed; soft-line viewport policy differs; ZWJ resize gap observed | Separate UX choice from preservation; history and selection anchors |
| Scrollback storage | Ordering observed | Enforce row and byte caps including side data; reclamation/entropy tests |
| Style/grapheme/hyperlink resources | Pending | Allocation bounds and continued parsing after caps/eviction |
| Selection / accessibility text | Pending | Logical ranges across history, graphemes, wraps and native AX boundary |
| Native IME / ligatures / fonts / Nerd symbols | Pending platform work | Xcode 27.0 verified; Core Text and native lifecycle experiments still pending |
| Typed output effects / control separation | Harness captures effects without executing them | Explicit OSC/reply fixtures and runtime policy integration |
| Latency / throughput / footprint / fairness | Unmeasured | Controlled baselines and dedicated scheduler/renderer/resource experiments |
| Licensing / adaptation maintenance | Direct package declares Apache-2.0; full release inventory pending | Inspect redistributed dependency notices and quantify necessary patches |

[Initial method and results](2026-09-26-terminal-replay.md) preserve both independent expectations and observed snapshots. [ADR 0002](../docs/adr/0002-owned-rust-engine-and-dependency-policy.md) selects an owned Rust engine; its first bounded implementation now has separate [findings](2026-09-26-owned-engine-slice.md). This matrix records the Alacritty baseline. Pending rows cannot be counted as passing merely because the underlying terminal project documents support.

The [executed Ghostty reference](2026-09-26-ghostty-reference-comparison.md) matches all 19 fixture expectations with mode 2027 enabled, while its mode-off control exposes ten differences. This does not close any implementation requirement for our owned Rust engine, whose first bounded slice is now implemented; see [owned-slice findings](2026-09-26-owned-engine-slice.md). The [first slice design](../docs/architecture/OWNED_ENGINE_SLICE.md) carries these findings forward.

The [owned history/resize follow-up](2026-09-26-history-screen-reflow.md) now passes all 19 existing fixtures / 244 replays with zero deferrals. Additional tests exercise exact history caps, separate buffers and primary/alternate resize. The Alacritty matrix above remains historical comparison evidence; broader protocol/native obligations remain open for the owned engine. That slice originally rejected primary resize when populated cells below its cursor would be lost. [ADR 0004](../docs/adr/0004-cursor-anchored-resize.md) now resolves rejection with bounded cursor-anchored retention and explicit crop counters; native integration remains open.
