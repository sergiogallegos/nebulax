#include "nebulax.h"
#include <assert.h>
#include <string.h>
#include <time.h>
#include <stdio.h>
_Static_assert(sizeof(NbCell) == 12, "cell ABI");
_Static_assert(sizeof(NbStyle) == 12, "style ABI");
_Static_assert(offsetof(NbFrame, styles) == 88, "style pointer offset");
_Static_assert(sizeof(NbStatus) == 24, "status ABI");
_Static_assert(sizeof(NbFrame) == 104, "frame ABI");
_Static_assert(offsetof(NbFrame, cells) == 32, "frame cell pointer offset");
_Static_assert(offsetof(NbFrame, row_count) == 80, "frame row count offset");

static NbBytes bytes(const char *s) { return (NbBytes){(const uint8_t *)s, strlen(s)}; }
int main(void) {
    assert(nb_abi_version() == 2);
    uint64_t handle = 12345;
    NbStatus status = {0};
    assert(nb_session_start((NbBytes){NULL, 1}, NULL, 0, 8, 3, &handle) == NB_INVALID);
    assert(nb_session_start((NbBytes){NULL, SIZE_MAX}, NULL, 0, 8, 3, &handle) == NB_LIMIT);
    assert(handle == 12345);
    assert(nb_session_status(UINT64_MAX, &status) == NB_INVALID);
    assert(nb_session_status(0, NULL) == NB_INVALID);
    assert(nb_frame_acquire(0, 0, NULL) == NB_INVALID);
    assert(nb_frame_release(0) == NB_INVALID);
    assert(nb_session_close(0) == NB_INVALID);
    assert(nb_session_start(bytes("/usr/bin/printf"), NULL, 0, 1, 3, &handle) == NB_INVALID);
    assert(handle == 12345);
    NbBytes expected = bytes("a\314\201\347\225\214"); /* combining accent and wide CJK */
    NbBytes arg = bytes("\033[1;38;2;12;34;56ma\314\201\347\225\214\033[0m");
    assert(nb_session_start(bytes("/usr/bin/printf"), &arg, 1, 8, 3, &handle) == NB_OK);
    const struct timespec pause = {0, 1000000};
    for (int i = 0; i < 5000; ++i) {
        assert(nb_session_status(handle, &status) == NB_OK);
        if (status.finished) break;
        nanosleep(&pause, NULL);
    }
    assert(status.finished && status.phase == NB_EXITED && status.exit_code == 0);
    uint64_t first, second, sentinel = 777;
    assert(nb_frame_acquire(handle, 0, &first) == NB_OK);
    assert(nb_frame_acquire(handle, 0, &second) == NB_OK);
    assert(nb_frame_acquire(handle, 0, &sentinel) == NB_LIMIT && sentinel == 777);
    NbFrame frame = {0};
    assert(nb_frame_view(first, &frame) == NB_OK);
    assert(frame.columns == 8 && frame.lines == 3 && frame.cell_count == 24 && frame.row_count == 3);
    assert(frame.cells[0].text_len == 3 && frame.cells[1].width == 2 && frame.cells[2].kind == 2);
    assert(frame.text_len == expected.len && memcmp(frame.text, expected.data, expected.len) == 0);
    assert(frame.style_count >= 2 && frame.style_count <= 1024);
    uint16_t style = frame.cells[0].style_id;
    assert(frame.styles[style].foreground == 0x020c2238 && frame.styles[style].attributes == 1);
    assert(frame.cells[1].style_id == style && frame.cells[2].style_id == style);
    assert(nb_session_release(handle) == NB_OK);
    assert(frame.styles[style].foreground == 0x020c2238);
    assert(nb_session_release(handle) == NB_INVALID);
    assert(memcmp(frame.text, expected.data, expected.len) == 0); /* Session gone, frame still live. */
    assert(nb_frame_release(first) == NB_OK);
    assert(nb_frame_view(first, &frame) == NB_INVALID); /* Do not dereference expired views. */
    assert(nb_frame_release(first) == NB_INVALID);
    assert(nb_frame_release(second) == NB_OK);
    puts("C ABI: layout, invalid arguments, limits, Unicode and lifetime passed");
    return 0;
}
