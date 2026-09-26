# 0006 — Bounded syntax and ordered terminal output

Status: implemented Phase 0 foundation on 2026-09-26 under the approved owned-engine direction. This defines a Rust integration contract, not a stable public ABI or complete VT implementation.

## Decision

Separate byte syntax (`parser.rs`), terminal meaning (`semantic.rs`) and output retention (`output.rs`). The parser emits print/control, generic ESC/CSI dispatch, complete OSC, cancellation and diagnostic events. Headers preserve omitted parameters, colon subparameters, private prefixes and intermediates without interpreting terminal operations. Existing editing/screen behavior stays in the semantic layer; accepting syntax does not imply implementing its meaning.

Bounds are 32 parameter fields including colon fields, one private prefix, two intermediates, 64 header bytes and numeric values through 65,535. OSC retains at most 1,024 bytes including its command. Overflow never dispatches a truncated header or payload. CAN/SUB cancel; ESC restarts a header; C0 execution inside a header preserves it. Malformed headers discard through the final byte unless cancelled/restarted. Unsupported DCS/APC/PM/SOS strings discard payloads in constant space. An embedded non-terminating ESC does not release their content into terminal input; in OSC it invalidates the whole payload. This deliberate fail-closed string profile is not full DEC parser equivalence. Raw 8-bit C1 dispatch remains unsupported.

Use a terminal-owned FIFO of typed `Reply`, `Bell` and `Title` events. Retain at most 64 queued events / 8,192 queued payload bytes, plus one completed event waiting for space, individually capped at 1,024 bytes. Total retained output is therefore at most 65 events / 9,216 logical payload bytes; the parser separately has its bounded OSC buffer. These limits do not claim exact allocator capacity or RSS. No event is silently dropped, reordered or coalesced.

`feed` returns `consumed` and `output_blocked`. The byte completing a waiting event is consumed; later bytes are untouched. While that event cannot enter the FIFO, feeds consume zero. Popping the oldest event promotes the waiting event when space permits. Resume only the unconsumed input suffix. At EOF, drain output and retry `finish` if blocked; blocked finish does not discard partial input state. Aggregating outcomes sums consumed bytes, ORs diagnostics and takes the latest backpressure state.

The runtime owns popped events and partial PTY writes, limits its own buffers, and stops draining/feeding when downstream capacity is exhausted. A blocked popped event must remain owned until handled. Replies capture query-time state. Native title/bell requests remain untrusted data subject to runtime policy; there are no callbacks, I/O or control-channel actions in the core.

## Initial semantics and limits

Implement only status query CSI 5n (CSI 0n reply), cursor query CSI 6n (one-based row/column reply), BEL, and UTF-8 title requests OSC 0/1/2. These meanings follow the [xterm control-sequence reference](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html), checked at patch 411. Empty titles are allowed; malformed UTF-8, control characters, cancelled and oversized strings produce no title event. OSC 52 and other unsupported commands produce no effects. DA capability claims, mode queries, OSC 7/8, styling and broad VT coverage remain unimplemented.

No dependency is added; unsafe Rust remains forbidden. This slice does not migrate cell storage, implement row damage, or prove PTY/native resource behavior. Next connect one PTY and a deterministic child to test bounded transport, lifecycle and resize, then continue toward the snapshot/FFI/native-window path. Coordinate cursor/mode/region and input ownership as part of that integration before broader protocol expansion.

## Evidence

`scripts/verify` passes 62 Rust tests, including twelve new parser/output tests. They independently check exact syntax limits and overflow, cancellation/recovery, omitted/colon parameters, malformed/oversized strings, UTF-8 titles, output order under every split and fixed chunks, event-count/payload pressure, exact resume, partial headers across blocked finish, query-time cursor capture across resize, and repeated mixed output floods. Existing Unicode, history, resize and editing tests still pass.

The [retained grid regression](../../benchmarks/results/p0-06-owned-parser-output/replay.json) matches all 19 unchanged fixtures through 244 replays; its source manifest captures the new implementation/tests. This corpus does not exercise the new output protocols; its adapter now fails if input is partially consumed or any event is emitted. The separate tests above cover the output contract. Earlier retained artifacts remain unchanged. Neither suite proves complete emulator compatibility or product performance.
