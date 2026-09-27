# 0019 — Soft and hard terminal reset

Status: implemented on 2026-09-27. Original std-only Rust; no dependency, toolchain, public API or private ABI change.

## Syntax and state contract

Support parameterless `CSI ! p` (DECSTR, soft reset) and `ESC c` (RIS, hard reset). Other prefixes, parameters or intermediates do not dispatch a reset. Malformed/cancelled headers retain the existing parser diagnostics; reset-looking bytes inside an OSC/DCS/APC/PM/SOS payload cannot escape string handling. These are in-stream operations, not process restarts or host-configuration changes.

The [DEC VT220 manual, section 4.18](https://www.zx.net.nz/computers/dec/vt220/doc/vt220-rm/chapter4.html#S4.18) defines the two sequences. Its [soft-reset table](https://www.zx.net.nz/computers/dec/vt220/doc/vt220-rm/table4-10.html) distinguishes saved-cursor initialization from moving the current cursor. Nebulax implements the applicable state subset with the explicit modern defaults and screen/history policies below. This does not claim complete hardware reset or VT220 emulation.

| State | Soft reset | Hard reset |
|---|---|---|
| Active screen selection | Keep current screen | Return to primary; discard alternate |
| Visible cells, styles on cells, soft-wrap boundaries | Preserve | Clear to default empty cells; remove wrap links |
| Primary scrollback | Preserve | Clear, including hidden primary history |
| Current physical cursor | Preserve row/column; cancel pending wrap | Home; no pending wrap |
| Active origin/margins | Absolute origin; full-height region | Same |
| Active autowrap | On | On |
| Current rendition/erase rendition | Default | Default |
| Active saved cursor | Home, default rendition, absolute origin, wrap on | Same |
| Hidden primary controls/saved state | Preserve | Reset with primary |
| Shared application cursor keys / visibility | Normal keys / visible cursor | Same |
| Shared tabs | Preserve custom/cleared stops and growth policy | Every eight columns, including future growth |
| Style table | Retain entries referenced by content/hidden state; ordinary bounded reclamation remains | Replace with default-only palette after dropping all old roots |
| Completed output events | Preserve FIFO order and captured values | Same |
| Geometry, configured limits, Unicode width policy | Preserve | Preserve |

Autowrap-on is Nebulax's startup policy, deliberately different from the VT220 table's soft-reset wrap-off default. Reset uses current viewport dimensions, never an 80×24 assumption. Soft reset on alternate does not switch buffers or reset hidden primary state; leaving alternate later restores that primary state, while terminal-wide key/visibility defaults remain shared. Clearing history on RIS is an explicit Nebulax policy. `FeedOutcome.history_evicted` reports when RIS discards nonempty primary history. Both commands conservatively set `changed`, including mode-only resets, so native consumers receive the new metadata.

## Ownership and stream order

Screen owns control/saved-state reset and visible-row clearing. Terminal coordinates active/hidden selection, shared modes, tabs, palette and decoding. Hard reset reuses primary visible row buffers, releases history and alternate storage plus cluster tails, and replaces the style table. Existing owned snapshots keep their independent text, palette and metadata even after style IDs are reused or the terminal is destroyed.

Reset closes grapheme attachment and leaves decoder/parser at ground. The ESC boundary first flushes a preceding incomplete UTF-8 prefix using the existing replacement policy; soft reset preserves that printed replacement and hard reset clears it with the rest of the display. Ordinary feed boundaries still do not flush incomplete input.

Completed replies, bell and title events survive both resets, including query-time values. They are part of the ordered output stream, not screen state. A reset behind an output-blocked event waits at the exact unconsumed byte offset; it cannot bypass pressure or discard that event. Reset itself emits no new event and does not disconnect/restart the PTY, alter native window geometry, read persistent configuration or grant any host authority. Earlier title events remain subject to runtime policy. Transport queues and immutable frame leases stay owned by their existing consumers.

## Validation and next task

Ten core tests cover preserved content/history/tabs, active versus hidden state, saved defaults, fresh-state equivalence with custom limits/width policy, resource reclamation, independent frames, tab shrink/grow, row-buffer reuse, rejected syntax/string payloads, byte partitions, decoder/grapheme boundaries, exact output resumption, effect order and repeated resets over 36 small geometries. The PTY handshake verifies mode-only frame publication, input encoding, ordered pre/post-reset replies and old-frame lifetime. The GUI startup probe verifies both resets before the existing native interaction/rendering test.

See [findings and evidence](../../research/2026-09-27-terminal-reset.md). The next bounded task is replacing the existing PTY libc crate with owned SDK-verified platform bindings under [ADR 0018](0018-standard-library-only-rust.md). Broader protocol coverage, public embedding acceptance and the native Metal/ligature implementation remain open.
