"""IRM, Unicode width adjustment and reset/query handshake through a real PTY."""
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


write("ABCDEFGH\x1b[3G\x1b[4h❤\ufe0f\x1b[4$p\x1b[6n".encode())
expect(b"\x1b[4;1$y\x1b[1;5R")
write(b"\x1b[2;1HIRM ON")
expect(b"s")
write(b"\x1b[!p\x1b[4$p\x1b[1;3HZ")
expect(b"\x1b[4;2$y")
write(b"\x1b[2;1H\x1b[2KIRM OFF")
expect(b"r")
write(b"\x1b[4h\x1b[?1049h\x1bc\x1b[4$p\x1b[?1049$p")
expect(b"\x1b[4;2$y\x1b[?1049;2$y")
write(b"IRM OK")
