# 0003 — Bounded history, alternate screen and reflow

Status: implemented and tested within the approved Phase 0 scope on 2026-09-26. These are research implementation policies, not a declaration of complete VT compatibility or final native-window behavior.

## Decision

Use one owned `Screen` for each live terminal buffer. A screen owns physical rows, its cursor and its history deque. The active primary screen is moved into saved state on DEC mode 1049 entry; a fresh alternate grid becomes active. Leaving restores the primary object, including its cursor and history. Repeated enable/disable is idempotent; it does not overwrite the saved primary state. Alternate screen scrolling retains no history. Only mode 1049 is supported in this slice; 47/1047/1048 and multi-parameter mode lists remain outside its scope.

The primary history limit is the smaller of configured physical rows and configured cells divided by current columns. Defaults are 1,000 rows / 65,536 cells; each limit can be lowered to zero, and both have hard maxima of 65,536. Evict oldest physical rows first. A retained suffix may start partway through a former logical line after eviction; its remaining soft-wrap links still join forward. Report eviction separately from scrolling with history disabled. Each visible grid is bounded by the existing 65,536-cell geometry limit. This bounds retained storage; it is not a measured RSS or allocation-failure guarantee.

Primary resize joins only soft-wrapped physical rows, keeps whole grapheme owners, removes structural wide padding, and repacks owners at the new width. Preserve printed spaces and hard boundaries. Track the cursor as a logical cell offset, including a pending-wrap position and positions inside wide tails. Reuse unused trailing screen rows, pull history into view on growth when it fits, and keep the cursor in the active area. Reapply history limits at the new width. A geometry-changing resize closes the current grapheme extension target but preserves incomplete UTF-8 and parser input; an identical-size resize is a complete no-op.

Alternate resize preserves physical coordinates using crop/pad. Remove a clipped wide owner completely rather than retaining half. Clamp its cursor, report crop counts, and resize/reflow the saved primary buffer at the same time. No primary history or content is imported into the alternate screen. Each buffer remains the sole owner of its own state; this is not a shadow grid.

Build sparse rows during reflow and materialize padding only for the bounded retained history/viewport. Otherwise a legal narrow/tall → wide/short resize could allocate a full-width row for every old hard line. All logic is safe Rust with no new dependencies.

## Explicit unresolved edge

A sufficiently small primary viewport cannot always keep the cursor and all later text visible simultaneously. The current slice returns `PrimaryContentWouldBeCropped` and leaves **both screens and partial input unchanged** if reflow would discard populated primary cells below the cursor. This also protects the saved primary while alternate is active. History eviction under configured limits remains allowed and reported. Alternate crop remains allowed and reported.

The error is an intentional exposed limitation. A final policy for native resize requires further design and application compatibility evidence before PTY/window integration; callers must handle the `Result`. Do not claim every valid geometry resize succeeds, silently drop primary text or hide this case in fixture expectations.

## Evidence and scope

All 19 existing fixtures now pass under 244 replay variations. Fourteen additional tests cover exact history limits, screen separation, resize round trips, cursor edges, pending wrap, wide-tail editing after reflow, malformed private modes, incomplete input, inactive-primary protection and extreme aspect ratios. See [executed findings](../../research/2026-09-26-history-screen-reflow.md).

DEC 1049 behavior is informed by the primary [xterm control-sequence reference](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html). The viewport/reflow/retention choices above are explicit Nebulax policies; they are not claimed to be mandated by that reference. No Ghostty implementation was imported or translated.
