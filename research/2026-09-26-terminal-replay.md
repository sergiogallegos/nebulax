# P0-06 initial terminal replay — 2026-09-26

Status: first bounded slice completed locally; full engine selection and Phase 0 remain open.

## Question, hypothesis and scope

Can the complete Rust engine provide a small deterministic state adapter and meet the first independent text/cursor/resize expectations without custom grid ownership? Does changing byte delivery boundaries change the observed state?

The approved [Phase 0 plan](../review/PHASE_0_PLAN.md) scoped this first task to a replay runner, fixed fixtures, normalized state and an initial gap matrix. This slice stops after those comparisons. It does not implement a PTY, UI, renderer, parser replacement, production engine wrapper or full protocol suite.

## Versions and method

Rust/Cargo 1.98.1, `alacritty_terminal` 0.26.0 and locked transitive dependencies; [bootstrap verification](../docs/development/TOOLCHAINS.md) records official sources. The engine exposes `Term`, grid/cell state and a reexported ANSI processor. Its registry source was read to integrate those APIs and inspect the scalar-width insertion path. No upstream implementation was copied into project source.

Nine synthetic fixtures specify final visible/history text, cursor and pending-wrap state; selected cells also specify text and exact wide/wrap flags. The emoji expectation is Nebulax's proposed two-cell grapheme policy, not a claim that all terminals agree. The resize expectation is a provisional viewport UX preference, not a normative VT requirement. Both expectations were written before the initial run and remain visible after the mismatch.

Each fixture runs whole-feed delivery, a repeat, fixed 1/2/3/7-byte chunks and every interior two-way split index of its feeds. Observed active-grid/history/cursor/mode/effect snapshots are compared after each operation. There are 125 replays in one suite. This is a deterministic correctness exercise with no timing, warmup or performance claims.

## Results

| Fixture | Replays | Observation |
|---|---:|---|
| ASCII cursor/erase | 19 | Expected text and cursor match |
| Split UTF-8 / CJK width | 11 | Expected text, wide/continuation cells and cursor match |
| Combining mark across feeds | 8 | Extends prior cell without extra cursor advance |
| Wide character at right edge | 13 | Wrap and continuation expectations match |
| Narrow soft-line reflow | 11 | Text survives, but `abcd` moves to history; `ef` is at visible row 0 with cursor `[0,2]`, rather than remaining at visible row 1 |
| Narrow hard-line resize | 11 | Explicit line boundaries and cursor expectations match |
| Scrollback order | 26 | Ordered history and visible text match |
| Alternate screen restore | 13 | Primary text and saved cursor return in this scenario |
| Woman-technologist ZWJ | 13 | Engine stores woman+ZWJ and laptop as separate wide cells; following `!` is at column 4 and cursor ends at 5, rather than column 2/cursor 3 |

All 125 replays agree at the observed operation checkpoints within each fixture. Seven fixtures match all final expectations. One required grapheme behavior gap and one viewport-policy question remain. Consistent chunk delivery does not make an incorrect product behavior acceptable.

Raw evidence: [replay JSON](../benchmarks/results/p0-06-initial/replay.json), [environment](../benchmarks/results/p0-06-initial/environment.json), [source hashes](../benchmarks/results/p0-06-initial/sources.json), [dependency inventory](../benchmarks/results/p0-06-initial/dependencies.json), [artifact hashes](../benchmarks/results/p0-06-initial/artifacts.json). The Git commit recorded by this initial run contains only the old README; dirty/untracked source hashes identify the actual experiment. This is not evidence of a clean committed release.

## Reproduction and regression treatment

Run `scripts/replay` for strict acceptance: exit 1 with the current gaps. `scripts/replay --allow-known-gaps` records exactly the same failing evidence and exits 0 only if the reviewed gap-state hashes and all ordinary expectations/checkpoint comparisons hold. Changed gap states, new mismatches and unexpected passes fail verification. Existing run directories are never overwritten.

`scripts/verify` covers formatting, warning-free Clippy, unit/integration checks and artifact recording. CLI tests verify strict/tolerated reports are identical, and that unknown options return an error. Unit tests exercise expectation mutation, input validation and gap-fingerprint regression handling. See the handoff for the final executed verification record; configuring hosted CI is not a claim it ran remotely.

## Confidence, limitations and decision consequence

Confidence is high about these specific observed states and limited for general protocol/Unicode behavior. Final state is not every internal engine state: inactive-screen data, saved modes/cursors, parser partial state, palette/damage and hyperlink identity need further transition fixtures. Split patterns are not exhaustive combinations. Native text shaping cannot establish terminal grapheme correctness and was not tested here.

No memory-byte cap, style/grapheme arena bound, input encoder, IME, accessibility adapter, modern-protocol coverage, full resize correctness or competitor performance is established. See [the gap matrix](ENGINE_GAPS.md). The headless run uses macOS 27.0; it proves nothing about the provisional macOS 14 deployment floor.

Retain reuse-first strategy but do not accept `alacritty_terminal` for production yet. Next, evaluate the maintenance path for required grapheme behavior against available complete-engine alternatives. Add modifiers, flags, variation selectors, erase/backspace and resize cases before deciding whether upstream changes, a bounded fork or another engine is justified. Review viewport anchoring separately from data preservation. Do not introduce a second authoritative grid or rewrite expected results to conceal gaps.
