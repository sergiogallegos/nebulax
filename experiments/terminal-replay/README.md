# P0-06: initial headless replay

This disposable package evaluates `alacritty_terminal`; it does not implement the Nebulax app or CLI.

`cargo run --locked -p nebulax-replay` emits deterministic JSON and exits 1 for unmet expectations. Use `scripts/replay` to include provenance and host metadata. `--allow-known-gaps` tolerates only explicitly reviewed failing-state hashes; it never removes failures from the report.

The current 19 synthetic fixtures cover selected cursor/erase, UTF-8/combining, wide-cell wrapping, resize, history, alternate-screen, emoji modifiers/flags/selectors, cluster erasure and reflow behavior. The original nine fixtures are preserved; the expanded suite performs 244 replays. Whole-feed delivery is compared with a repeat, fixed 1/2/3/7-byte chunks and every single interior split position of each feed. For multiple feeds, the same split index is tried on each; this is not all combinations of partitions. Compare observed state after each feed or resize; check independent expected text/cursor/cells at the final operation.

Snapshots include active-grid cells/history, text/zero-width scalars, colors, underline color, flags, hyperlink URI, mode bits, cursor/wrap state and captured synthetic events. They are not a serialization of the entire emulator: inactive-screen internals, saved modes/cursors, parser partial state, palette overrides, damage state and hyperlink identity are not exhaustively compared. Further fixtures must exercise those transitions. Text summaries strip trailing ASCII spaces and omit continuation placeholders; full cell snapshots retain them. Debug color/event strings are internal to this pinned experiment, not a stable product format.

No events are executed. No PTY, shell, clipboard, GUI, font stack or MCP service is created. There is no performance instrumentation. Resource caps on fixture size protect this harness; they do not prove production engine allocation bounds.

Findings and next work: [initial record](../../research/2026-09-26-terminal-replay.md), [grapheme follow-up](../../research/2026-09-26-grapheme-engine-assessment.md), [gap matrix](../../research/ENGINE_GAPS.md).
