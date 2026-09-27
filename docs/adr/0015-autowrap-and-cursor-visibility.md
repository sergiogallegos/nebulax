# 0015 — Saved autowrap and snapshot cursor visibility

Status: implemented on 2026-09-27. Extends screen/saved-state and snapshot contracts; private ABI v3 replaces v2. No dependency/toolchain change.

## State and printing policy

Support DEC private modes 7 (DECAWM) and 25 (DECTCEM), including validated parameter lists. Whole-list validation precedes mutation; unsupported entries cannot partially toggle modes or switch screens. The [xterm reference](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) defines these mode numbers and set/reset meanings. Implementation is original Rust.

Autowrap defaults on and belongs to each `Screen`. ESC 7/8 saves/restores it alongside cursor, pending wrap, origin and rendition. The default saved cursor also has autowrap on. A new alternate screen starts with wrapping on; leaving it restores the primary's mode. Resize retains active/saved modes, clamps saved positions and prevents pending wrap when wrapping is off. Primary resize still reflows existing logical content; disabling print-time wrapping does not disable resize policy.

An explicit mode-7 command cancels pending wrap, including repeated sets/resets. With autowrap on, existing delayed-wrap and wide-edge behavior remains unchanged. With wrapping off, printing never automatically advances rows: narrow text overwrites the rightmost column and the cursor stays there. A wide character at that edge is clamped left into the final two columns, replacing intersected owners while retaining the whole new cluster. A width-changing selector follows the same rule, retaining the original lead's style. A later narrow overwrite clears the complete intersected wide owner as usual. This is an explicit Nebulax wide-text policy, not a claim of identical behavior across other terminals.

No-wrap mode creates neither soft-wrap padding nor history through printing. Explicit LF/index/region operations continue to work. Supported mode commands close grapheme attachment as other non-SGR controls do; mode 25 alone preserves cursor coordinates and pending wrap.

Cursor visibility defaults on and belongs to `Terminal`, shared across screen switches. It is not saved/restored by ESC 7/8 and is unaffected by resize. It controls only presentation: hiding the cursor does not disable input, printing or cursor movement.

## Snapshot and native contract

`Snapshot::cursor_visible()` owns the visibility value. A visibility change sets `FeedOutcome.changed`, causing the worker to publish even when text, cursor coordinates and row versions are unchanged. Repeating the same visibility command needs no new frame. Old frames retain their original metadata after mutation or session destruction.

Private ABI v3 appends `cursor_visible` (0/1) and zero `reserved` to `NbFrame`: 112 bytes, visibility offset 104 on the verified 64-bit target. Cell/style records remain 12 bytes; core cells remain 16 bytes. All bundled callers check v3 and rebuild together; v2 binary compatibility is not promised. Existing two-MiB array/palette payload accounting and session/frame caps remain unchanged; fixed frame metadata is separate.

Consumers must handle cursor overlay changes independently of row versions. The AppKit preview already redraws each delivered frame and now omits the cursor when hidden. Cursor shape/blink and renderer performance remain separate work.

## Evidence

Ten new core tests cover disabled-wrap overwrites, explicit transition/pending-wrap policy, wide/selector changes across geometry and byte partitions, saved defaults, alternate isolation, resize, explicit region scrolling, atomic mode lists and immutable visibility-only snapshots. The real-PTY worker test waits for a hidden frame, sends a bounded native acknowledgement and receives a show-only frame with unchanged rows/text/position. C verifies hidden metadata and layout after session release; Swift verifies visible metadata and lifetime.

The GUI test checks right-edge output with wrapping disabled, captures a hidden cursor after resize, sends a seventh native event to request show-only output, then compares rendered pixels. Changes must be nonzero and confined to the cursor cell while text/row versions remain identical. Full verification passes 160 Rust tests, all 19 fixtures / 244 replays, C/Swift checks and AppKit compilation; the GUI test also passes. See [findings](../../research/2026-09-27-autowrap-visibility.md).
