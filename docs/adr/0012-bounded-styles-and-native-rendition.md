# 0012 — Bounded styles and native rendition

Status: implemented on 2026-09-27. Extends ADRs 0010/0011 and replaces the private snapshot ABI v1 with v2. No dependency change.

## Decision

Keep the core cell at 16 bytes. Its existing `u16` style ID indexes a terminal-owned table of at most 1,024 entries, including permanent default zero. A style is a 12-byte value: encoded foreground, background and attribute bits. Colors distinguish default, indexed 0–255 and RGB; the core does not resolve theme colors. No cluster storage or second grid is introduced.

Intern complete rendition values and background-only erase values. Reuse matching values or vacant slots; on capacity pressure, mark cells in visible/hidden buffers and history, current/saved rendition on both screens, default zero and any pending first allocation. Reclaim unreferenced slots and retry. Entries otherwise remain a bounded cache. Table lookup is linear and collection scans bounded retained state; repeated hostile allocation pressure can be costly. This is a resource/lifetime implementation, not a throughput result.

An SGR operation changes rendition only if both values can be represented. Exhaustion consumes the sequence, retains the previous rendition and sets `FeedOutcome.style_limit`; it does not corrupt existing cells or approximate colors. An unused tentative entry can remain cached until collection. IDs may be recycled after all engine references disappear, so callers must resolve them within the owning terminal or use an owned snapshot. Deep terminal clones own independent tables.

## Protocol and editing policy

The supported SGR subset is reset/omitted zero; bold, faint, italic, single/double underline, inverse, hidden and strike; their resets; standard/bright indexed colors; default color resets; and extended indexed/RGB foreground/background. Extended colors accept semicolon forms and colon groups, including omitted or zero RGB color-space fields. Underline `4:0/1/2` is supported. Components must be explicit 0–255. Blink, curly underline, underline colors and nonzero color-space selectors remain unsupported.

Validate the complete SGR before applying it. Unknown/malformed entries reject the entire sequence with `unsupported`, rather than partially applying earlier parameters. Parser bounds remain 32 fields and 64 header bytes. This deliberate bounded profile is smaller and stricter than xterm; it is not complete SGR conformance. The [xterm control-sequence reference](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) informed parameter meanings; implementation is original Rust.

Successful SGR changes preserve pending wrap and grapheme attachment. Scalars extending an existing grapheme retain its original lead's style; a new grapheme uses current rendition. Wide continuations always share the owner's style. Other dispatched controls, invalid sequences, cancellation and ignored strings close attachment. An incomplete ESC alone defers that decision until dispatch; UTF-8 decoder boundaries remain explicit.

Current and saved rendition belong to each screen, alongside cursor/origin state. ESC 7/8 saves/restores them; a new alternate screen starts at default and leaving it restores the hidden primary's state. Erasure and newly scrolled-in rows retain only the current background. Foreground, inverse and decorations do not survive erasure. This is an explicit background-color-erase policy; see the [xterm FAQ](https://invisible-island.net/xterm/xterm.faq.html).

Reflow retains styles, including background-only empty cells, and keeps wide owners intact. Crop loss counts now include styled empty cells, excluding wrap padding as before. Newly added resize padding, cleared clipped owners and detached wrap-padding cells use default style; resize does not fill new geometry with current rendition. Existing cursor-anchored crop/history limits are unchanged.

## Snapshot and native boundary

Each snapshot copies the bounded palette into owned 12-byte style values; vacant slots become default values. The copy can include cached/invisible styles, up to 12 KiB. It does not retain engine borrows or table ownership. Cells become 12-byte transfer records with `style_id` and zero reserved bits; core cells remain 16 bytes. Row comparison resolves actual style values, so recycling the same numeric ID cannot hide color-only damage. Rendition changes without visible edits do not change row versions.

The two-MiB snapshot payload cap includes cells, UTF-8, row metadata and palette. Existing byte-limit failure behavior and native frame/session caps remain. Private ABI v2 adds `NbStyle`, changes `NbCell` from 8 to 12 bytes and `NbFrame` from 88 to 104 bytes on the verified 64-bit target. All bundled callers check version 2 and are rebuilt together; there is no v1 binary compatibility promise. Frames and their style values survive engine mutation and session teardown.

The AppKit prototype resolves a fixed 16-color palette, the 256-color cube/grayscale ramp and RGB with default theme colors. It paints backgrounds before text, including empty/continuation cells. Core Text draws lead clusters with four cached regular/bold/italic font combinations, faint blending, inverse colors, hidden text and line decorations. Bold does not implicitly select bright colors. Fixed cell metrics, clipping, fallback font quality and full redraw remain prototype choices; no Metal/performance or typography-completeness claim is made.

## Validation

Twelve new core tests cover independent SGR expectations, every two-way delivery split and byte-at-a-time feeds, malformed atomicity, grapheme style ownership, wide erasure, colored blanks/reflow, saved/hidden/history roots, bounded churn, two-slot rollback, exhaustion recovery, immutable older frames and style-only damage. The legacy unsupported-SGR negative test now uses unsupported `999m`; supported `31m` has explicit positive tests. Original replay fixtures and retained historical evidence are unchanged.

C checks verify v2 sizes/offsets and colored wide cells through a real PTY; C/Swift checks retain palettes after session teardown. The GUI probe validates a styled wide lead/continuation and background-only erasure after resize, then captures drawing and waits for worker cleanup. See [findings and evidence](../../research/2026-09-27-bounded-styles.md). Broader VT coverage, native IME/accessibility and production resource/performance acceptance remain open.
