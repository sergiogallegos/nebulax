#ifndef NEBULAX_PRIVATE_H
#define NEBULAX_PRIVATE_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Private ABI v3, macOS only. All functions return status, except version.
 * Caller storage: readable inputs, writable aligned outputs, no aliasing or
 * concurrent mutation during a call. Outputs are unchanged on nonzero status.
 * UTF-8 command strings: no NUL, <=4096 bytes each, <=16 args, <=16384 total.
 * Commands are trusted native caller input; PTY output never invokes this API.
 * No function waits for child cleanup. Short registry/mailbox locks are used.
 * Unwinding Rust panics become NB_PANIC; invalid pointers/OOM/abort are not caught.
 */
enum { NB_OK=0, NB_INVALID=1, NB_LIMIT=2, NB_BUSY=3, NB_NO_FRAME=4, NB_FAILED=5, NB_PANIC=6 };
enum { NB_STARTING=0, NB_RUNNING=1, NB_EXITED=2, NB_STOPPED=3, NB_SESSION_FAILED=4 };
typedef struct { const uint8_t *data; size_t len; } NbBytes;
typedef struct {
    uint32_t phase;
    int32_t exit_code;
    uint32_t failure, finished;
    uint64_t denied_effects;
} NbStatus;
typedef struct { uint32_t text_offset; uint16_t text_len; uint8_t width, kind; uint16_t style_id, reserved; } NbCell;
/* Color: 0 default; 0x01000000|index (0..255); 0x02000000|RRGGBB.
 * Attributes: bold=1, faint=2, italic=4, underline=8, inverse=16, hidden=32,
 * strike=64, double underline=128. Styles are immutable frame-owned values. */
typedef struct { uint32_t foreground, background, attributes; } NbStyle;
typedef struct {
    uint64_t generation;
    uint32_t columns, lines, cursor_row, cursor_column, wrap_pending, alternate;
    const NbCell *cells;
    size_t cell_count;
    const uint8_t *text;
    size_t text_len;
    const uint64_t *row_versions;
    const uint8_t *row_wraps;
    size_t row_count;
    const NbStyle *styles;
    size_t style_count;
    uint32_t cursor_visible, reserved;
} NbFrame;
uint32_t nb_abi_version(void);
int32_t nb_session_start(NbBytes executable, const NbBytes *args, size_t argc, uint32_t columns, uint32_t lines, uint64_t *out);
int32_t nb_session_status(uint64_t session, NbStatus *out);
/* Accepted resizes coalesce; failure to apply is reported as session failure. */
int32_t nb_session_resize(uint64_t session, uint32_t columns, uint32_t lines);
/* close requests cancellation. Poll status.finished; release is BUSY until true.
 * Max 4 live session slots, including closing workers. Never reused handle IDs. */
/* Native input only: valid non-control UTF-8 commits <=4096 bytes or basic keys.
 * Atomic acceptance into <=64-event/16-KiB mailbox; NB_LIMIT means not accepted.
 * Input may be abandoned by cancellation/exit; never interleaves partial replies.
 * Keys: 1 return, 2 backspace, 3 tab, 4 escape, 5 up, 6 down, 7 left,
 * 8 right, 9 home, 10 end, 11 delete; 32..63 encode C0 controls 0..31.
 */
int32_t nb_session_text(uint64_t session, NbBytes text);
/* Explicit paste: nonempty UTF-8 <=4096 bytes; TAB/CR/LF allowed, other
 * C0/C1/DEL controls rejected atomically. LF/CRLF normalize to CR. The mailbox
 * reserves payload+12 bytes; mode 2004 is sampled once at worker dispatch.
 * One complete frame is serialized with replies/keys, including short writes.
 * Clipboard access and user intent are native policy, never PTY output effects.
 * Additive private ABI v3 entry point; frame layouts/version are unchanged. */
int32_t nb_session_paste(uint64_t session, NbBytes text);
int32_t nb_session_key(uint64_t session, uint32_t key);
int32_t nb_session_close(uint64_t session);
int32_t nb_session_release(uint64_t session);
/* Max 2 acquired frames/session, 8 total. after_generation filters old frames.
 * Latest frame replaces older mailbox frames. Compare row_versions against your
 * last DISPLAYED frame, not against generation-1. Geometry changes redraw all.
 * Frames remain readable after session release. Read-only pointers are valid
 * until frame_release, which MUST NOT race their use. No engine lock is held.
 * cursor_visible is 0/1; frame.reserved is zero. Cursor changes may leave
 * row_versions unchanged; redraw the old/new cursor overlay independently.
 * Every cell.style_id is < style_count (<=1024); cell.reserved is zero.
 * kind: 0 empty, 1 lead, 2 wide continuation, 3 wrap padding. Text is UTF-8.
 * <=2 MiB cell/text/row/style payload per frame; no truncation on overflow.
 */
int32_t nb_frame_acquire(uint64_t session, uint64_t after_generation, uint64_t *out);
int32_t nb_frame_view(uint64_t frame, NbFrame *out);
int32_t nb_frame_release(uint64_t frame);
#ifdef __cplusplus
}
#endif
#endif
