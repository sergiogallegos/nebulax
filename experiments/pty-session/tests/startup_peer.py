"""Controlled startup handshake; this is not a general TUI compatibility test."""
import os
import signal
import sys
import tty

signal.alarm(10)
tty.setraw(0)


def exchange(request, expected):
    while request:
        request = request[os.write(1, request):]
    actual = b""
    while len(actual) < len(expected):
        part = os.read(0, len(expected) - len(actual))
        assert part, "PTY closed during startup"
        actual += part
    assert actual == expected, (actual, expected)


exchange(b"\x1b[c\x1b[>c\x1b[>q\x1b[5n\x1b[6n",
         b"\x1b[?1;0c\x1b[>0;1;0c\x1bP>|Nebulax " + sys.argv[1].encode("ascii")
         + b" (experimental)\x1b\\\x1b[0n\x1b[1;1R")
exchange(b"\x1b[?1$p\x1b[?7$p\x1b[?25$p\x1b[?2004$p\x1b[4$p",
         b"\x1b[?1;2$y\x1b[?7;1$y\x1b[?25;1$y\x1b[?2004;2$y\x1b[4;2$y")
exchange(b"\x1b[?1049h\x1b[?1;6h\x1b[?7;25l\x1b[2;3r\x1b[2;3H"
         b"\x1b[?1049$p\x1b[?1$p\x1b[?6$p\x1b[?7$p\x1b[?25$p\x1b[6n",
         b"\x1b[?1049;1$y\x1b[?1;1$y\x1b[?6;1$y\x1b[?7;2$y\x1b[?25;2$y\x1b[2;3R")
exchange(b"\x1b[?1049;1l\x1b[?25h\x1b[?1049$p\x1b[?6$p\x1b[?7$p\x1b[?25$p",
         b"\x1b[?1049;2$y\x1b[?6;2$y\x1b[?7;1$y\x1b[?25;1$y")
os.write(1, b"START OK")
