# 0016 — Explicit experimental identification and state-derived mode replies

Status: implemented on 2026-09-27 after checkpoint `c069a6b`. No dependency, toolchain, public API or private ABI change.

## Profile

An original Rust query layer separates request recognition and reply construction from editing operations. The [xterm control-sequence reference](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) (patch 411, updated 2026-08-23) defines the request/reply syntax and mode-status values. This deliberately small profile supports startup negotiation without claiming advanced terminal options.

| Request (ESC shown symbolically) | Reply |
|---|---|
| `ESC [ c`, `ESC [ 0 c`, or legacy `ESC Z` | `ESC [ ? 1 ; 0 c` |
| `ESC [ > c` or `ESC [ > 0 c` | `ESC [ > 0 ; 1 ; 0 c` |
| `ESC [ > q` or `ESC [ > 0 q` | `ESC P > \| Nebulax <package-version> (experimental) ESC \` |
| `ESC [ 5 n` | `ESC [ 0 n` |
| `ESC [ 6 n` | `ESC [ <row> ; <column> R`, one-based and origin-relative |
| `ESC [ ? <mode> $ p` | `ESC [ ? <mode> ; <state> $ y` |
| `ESC [ <mode> $ p` | `ESC [ <mode> ; 0 $ y` |

Spaces in this table separate notation; only the version text contains actual spaces. DA1 uses the minimal VT100-family identifier that the reference labels VT101 with no options. DA2 type 0 uses that baseline family; revision 1 is Nebulax's experimental profile revision, not an xterm patch or copied firmware version. The ROM field is zero. XTVERSION identifies Nebulax with its actual Cargo package version. Neither legacy identifier certifies complete hardware emulation; unsupported VT behavior remains observable. Do not advertise advanced-video option bits, graphics, printer, clipboard or other unsupported facilities. The shell preview keeps `TERM=dumb`; a terminfo/product compatibility profile needs separate acceptance.

DECRQM reports state 1 (set) or 2 (reset) only for implemented DEC private modes: 1 application cursor keys, 6 origin, 7 autowrap, 25 cursor visibility and 1049 alternate screen. Values come directly from the authoritative terminal/screen state. All other private modes and all ANSI modes return state 0 (unrecognized). No permanently set/reset modes are advertised. An explicit numeric mode is required; zero is unrecognized. Multi-parameter or colon-subparameter requests, other private prefixes, unsupported intermediates and nonzero identification selectors do not reply. Parser numeric/header bounds apply unchanged; malformed input cannot produce a truncated query.

## State and transport

Queries do not change display, rendition, cursor, pending wrap, modes, history or saved state. Like existing non-SGR controls they close grapheme attachment. Replies capture state immediately at dispatch, so delayed draining or a subsequent resize/screen switch cannot rewrite them. Query-only output does not mark a frame changed; the worker drains output independently of snapshot publication.

Replies use the existing typed FIFO: at most 64 queued events and 8,192 payload bytes plus one bounded pending event. The exact consumed offset includes the completed waiting query; the caller drains and resumes at that offset. Partial writes remain ordered through the single PTY writer. No host effect, capability callback or control-channel operation is introduced. Echoed replies do not generate new replies.

## Validation and next scope

Eight core tests cover exact bytes, all split points and byte-at-a-time delivery, current/saved/alternate/resize mode state, origin coordinates, read-only display state, malformed/cancelled/oversized syntax, response-loop avoidance and 400 ordered replies under repeated queue pressure. A real PTY child blocks on exact startup replies, toggles modes and switches screens before drawing its success marker. The AppKit peer also gates its ready marker on an exact exchange.

See [findings and retained evidence](../../research/2026-09-27-device-replies.md). These synthetic peers validate bounded negotiation and delivery, not vim/tmux/fish compatibility. Next implement explicit soft/hard reset contracts, including saved state, modes, margins, tabs, style roots, history, pending output and immutable frames; then use reset during controlled startup/teardown. Broader keyboard modes, insert mode and native IME/accessibility remain separate work.
