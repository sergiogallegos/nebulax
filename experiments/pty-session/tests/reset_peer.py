"""Reset/reply/input handshake with a real PTY; no user shell or startup files."""
import os
import signal
import tty

signal.alarm(10)
tty.setraw(0)


def write(data):
    while data:
        data = data[os.write(1, data):]


def expect(expected):
    actual = b""
    while len(actual) < len(expected):
        part = os.read(0, len(expected) - len(actual))
        assert part
        actual += part
    assert actual == expected, (actual, expected)


# The first frame must survive a mode-only soft reset with identical row damage.
write(b"\x1b[31;44mKEEP\x1b[?1h\x1b[?7;25l\x1b7")
expect(b"s")
write(b"\x1b[!p\x1b[6n\x1b[?1$p\x1b[?7$p\x1b[?25$p")
expect(b"\x1b[1;5R\x1b[?1;2$y\x1b[?7;1$y\x1b[?25;1$y")
expect(b"\x1b[A")  # Worker must encode this native key in reset normal mode.
# Ensure hidden primary/history, custom tabs and alternate state all existed.
write(b"\x1b[3g\x1b[6G\x1bH\r\na\r\nb\r\nc\x1b[?1049h\x1b[32mALT\x1b[?1;6h\x1b[?7;25l")
# Earlier replies remain ordered before post-RIS replies. Then wait for native ack.
write(b"\x1b[?7$p\x1bc\x1b[?1049$p\x1b[?7$p\x1b[6n")
expect(b"\x1b[?7;2$y\x1b[?1049;2$y\x1b[?7;1$y\x1b[1;1R")
expect(b"r")
write(b"\x1b8\t\x1b[6n")
expect(b"\x1b[1;8R")  # Eight-column viewport clamps restored default stop.
write(b"\rRESET OK")
