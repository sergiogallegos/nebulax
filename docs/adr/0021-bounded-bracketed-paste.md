# ADR 0021: Bounded explicit paste and mode 2004

Date: 2026-09-27. Status: accepted for the research integration slice.

## Decision

Implement xterm private mode 2004 through the existing validated mode-list semantics and state-derived DECRQM. The mode is terminal-wide, initially reset, independent of cursor saves, screen switches and resize. Both DECSTR and RIS reset it by explicit Nebulax policy. Changing it does not damage visible rows or require a frame publication.

The core exposes `bracketed_paste`, `valid_paste` and `encode_paste`. The runtime receives an explicit `Input::Paste` and the private C bridge adds `nb_session_paste`. Text/key input keeps its existing meaning. No clipboard access or native action is derived from PTY output.

A paste must contain 1–4,096 UTF-8 bytes. TAB, CR and LF are permitted; all other Unicode control scalars (C0, DEL and C1) cause whole-event rejection, including ESC/end-marker injection. Unicode continuation bytes are not mistaken for controls. LF and CRLF normalize to one CR; standalone CR is preserved. This intentionally normalizes line endings, without silently deleting disallowed controls. Hosts own clipboard acquisition, user intent and any multiline confirmation policy. Empty/oversized events are rejected; large-paste streaming and binary paste are not supported.

When mode 2004 is enabled at dispatch, encode `ESC [ 200 ~`, normalized text, and `ESC [ 201 ~`. Otherwise send normalized text only. This is framing, not a guarantee about what a receiving application executes. Framing is decided once, not during admission or after each short write.

## Ownership and bounds

The shared mailbox remains limited to 64 events and 16 KiB. Paste events reserve their raw length plus twelve framing bytes even when mode is currently reset. One staged raw event can additionally occupy 4 KiB, and one pending encoded write can occupy 4,108 bytes. Encoding briefly holds both the raw string and encoded vector. These are logical payload bounds; allocator/container overhead and kernel buffers are separate.

Existing generated replies are selected before staged input. One writer owns the complete selected frame and its offset; replies, keys and later paste events cannot split it. While a write is blocked, bounded PTY output is still consumed to prevent duplex echo deadlock. A mode change or reset during that drain affects later events only. There is no total arrival ordering between native input and unread PTY output. Acceptance is not delivery: cancellation abandons pending data, and write errors may leave a partial frame without its terminator.

`nb_session_paste` is additive to private ABI v3; no data layout or existing symbol changes. Trusted callers must supply readable buffers through return. The bridge checks length/UTF-8, copies the payload and admits it atomically through the existing registry/worker path. A future stable SDK will need explicit symbol/capability negotiation.

## Evidence and scope

Six core tests cover every two-way stream split, exact mode replies, control/UTF-8 byte bounds, newline normalization, global/reset ownership, invalid-list atomicity and sustained reply pressure. Three portable runtime tests cover framing reservations, every short-write offset and mode/reset/reply ordering. A fourth runtime test drives a real raw PTY peer through enabled/soft-reset/hard-reset exchanges. C checks invalid pointer/length/encoding arguments; Swift checks invalid payload rejection and exact live-PTY round trips. AppKit submits one synthetic explicit paste after the existing visibility-only pixel check and waits for `PASTE OK`.

The [findings](../../research/2026-09-27-bracketed-paste.md) retain results. Original fixtures and historical evidence remain unchanged. No third-party crate, toolchain update, clipboard implementation or public SDK release is introduced.

## Reference

[Xterm control sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Bracketed-Paste-Mode), accessed 2026-09-27, documents mode 2004 and its start/end sequences. Its readline-mode section describes the default LF-to-CR translation. The size limit, CRLF coalescing, reject-whole-control policy and soft-reset behavior above are explicit Nebulax decisions, not claims that every terminal uses the same policy. Modes 2005/2006 remain unsupported and query as unrecognized.
