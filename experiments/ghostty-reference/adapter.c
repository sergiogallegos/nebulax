/* Nebulax research adapter. Original code against the pinned public C API.
 * No PTY, native effects, callbacks, threads or retained grid references.
 * Internal line protocol: N cols rows history mode; W hex; R cols rows; S; F.
 */
#include <ghostty/vt.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static GhosttyTerminal terminal;

_Noreturn static void fail(const char *message) {
    fprintf(stderr, "adapter: %s\n", message);
    ghostty_terminal_free(terminal);
    exit(2);
}

#define CHECK(call) do { if ((call) != GHOSTTY_SUCCESS) fail(#call); } while (0)
#define GET(key, value) CHECK(ghostty_terminal_get(terminal, key, &(value)))

static void snapshot(void) {
    uint16_t cols, rows, x, y;
    size_t history;
    bool pending, processing_error;
    GhosttyTerminalScreen screen;
    GhosttyTerminalModeConfig mode = { .mode = GHOSTTY_MODE_GRAPHEME_CLUSTER };
    GET(GHOSTTY_TERMINAL_DATA_COLS, cols);
    GET(GHOSTTY_TERMINAL_DATA_ROWS, rows);
    GET(GHOSTTY_TERMINAL_DATA_CURSOR_X, x);
    GET(GHOSTTY_TERMINAL_DATA_CURSOR_Y, y);
    GET(GHOSTTY_TERMINAL_DATA_SCROLLBACK_ROWS, history);
    GET(GHOSTTY_TERMINAL_DATA_CURSOR_PENDING_WRAP, pending);
    GET(GHOSTTY_TERMINAL_DATA_ACTIVE_SCREEN, screen);
    GET(GHOSTTY_TERMINAL_DATA_VT_PROCESSING_ERROR, processing_error);
    GET(GHOSTTY_TERMINAL_DATA_MODE, mode);
    if (history > 1000 || cols > 160 || rows > 60) fail("snapshot bounds exceeded");
    printf("{\"columns\":%u,\"lines\":%u,\"history_lines\":%zu,"
           "\"cursor\":[%u,%u],\"wrap_pending\":%s,\"active_screen\":%d,"
           "\"grapheme_mode\":%s,\"processing_error\":%s,\"rows\":[",
           cols, rows, history, y, x, pending ? "true" : "false", screen,
           mode.value ? "true" : "false", processing_error ? "true" : "false");
    for (size_t row = 0; row < history + rows; row++) {
        if (row) putchar(',');
        GhosttyPoint point = { .tag = GHOSTTY_POINT_TAG_SCREEN,
            .value.coordinate = { .x = 0, .y = (uint32_t)row } };
        GhosttyGridRef ref = { .size = sizeof(ref) };
        GhosttyRow row_value;
        bool wrap;
        CHECK(ghostty_terminal_grid_ref(terminal, point, &ref));
        CHECK(ghostty_grid_ref_row(&ref, &row_value));
        CHECK(ghostty_row_get(row_value, GHOSTTY_ROW_DATA_WRAP, &wrap));
        printf("{\"wrap\":%s,\"cells\":[", wrap ? "true" : "false");
        for (uint16_t col = 0; col < cols; col++) {
            if (col) putchar(',');
            point.value.coordinate.x = col;
            CHECK(ghostty_terminal_grid_ref(terminal, point, &ref));
            GhosttyCell cell;
            GhosttyCellWide wide;
            uint32_t codepoints[4096];
            size_t count;
            CHECK(ghostty_grid_ref_cell(&ref, &cell));
            CHECK(ghostty_cell_get(cell, GHOSTTY_CELL_DATA_WIDE, &wide));
            CHECK(ghostty_grid_ref_graphemes(&ref, codepoints, 4096, &count));
            printf("{\"wide\":%d,\"codepoints\":[", wide);
            for (size_t i = 0; i < count; i++) {
                if (i) putchar(',');
                printf("%u", codepoints[i]);
            }
            printf("]}");
        }
        printf("]}");
    }
    printf("]}\n");
}

static unsigned nibble(char c) {
    if (c >= '0' && c <= '9') return (unsigned)(c - '0');
    if (c >= 'a' && c <= 'f') return (unsigned)(c - 'a' + 10);
    fail("invalid hex");
    return 0;
}

int main(void) {
    char line[8200];
    while (fgets(line, sizeof(line), stdin)) {
        size_t len = strlen(line);
        if (!len || line[len - 1] != '\n') fail("unterminated/oversized command");
        line[--len] = '\0';
        unsigned cols, rows, mode_value;
        size_t history;
        char extra;
        if (line[0] == 'N') {
            if (terminal || sscanf(line, "N %u %u %zu %u %c", &cols, &rows,
                                   &history, &mode_value, &extra) != 4 ||
                cols < 2 || cols > 160 || rows < 1 || rows > 60 ||
                history > 1000 || mode_value > 1) fail("invalid new command");
            CHECK(ghostty_terminal_new(NULL, &terminal, (uint16_t)cols, (uint16_t)rows));
            CHECK(ghostty_terminal_set(terminal, GHOSTTY_TERMINAL_OPT_SCROLLBACK_MAX_LINES, &history));
            GhosttyTerminalModeConfig mode = {
                .mode = GHOSTTY_MODE_GRAPHEME_CLUSTER, .value = mode_value != 0 };
            CHECK(ghostty_terminal_set(terminal, GHOSTTY_TERMINAL_OPT_MODE, &mode));
        } else if (!terminal) {
            fail("command without terminal");
        } else if (line[0] == 'W' && line[1] == ' ') {
            if ((len - 2) % 2 || len - 2 > 8192) fail("invalid feed length");
            uint8_t bytes[4096];
            size_t count = (len - 2) / 2;
            for (size_t i = 0; i < count; i++)
                bytes[i] = (uint8_t)((nibble(line[2 + i * 2]) << 4) | nibble(line[3 + i * 2]));
            ghostty_terminal_vt_write(terminal, bytes, count);
        } else if (line[0] == 'R') {
            if (sscanf(line, "R %u %u %c", &cols, &rows, &extra) != 2 ||
                cols < 2 || cols > 160 || rows < 1 || rows > 60) fail("invalid resize");
            CHECK(ghostty_terminal_resize(terminal, (uint16_t)cols, (uint16_t)rows, 0, 0));
        } else if (strcmp(line, "S") == 0) {
            snapshot();
        } else if (strcmp(line, "F") == 0) {
            ghostty_terminal_free(terminal);
            terminal = NULL;
        } else {
            fail("unknown command");
        }
    }
    if (ferror(stdin) || terminal) fail("read failure or missing free");
    return fflush(stdout) == 0 ? 0 : 2;
}
