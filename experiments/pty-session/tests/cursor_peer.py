"""Raw PTY protocol handshake; only fixed synthetic input is accepted."""
import os
import signal
import tty

signal.alarm(10)
tty.setraw(0)


def read_exact(expected):
    actual = b""
    while len(actual) < len(expected):
        part = os.read(0, len(expected) - len(actual))
        assert part
        actual += part
    assert actual == expected, (actual, expected)


os.write(1, b"TOP\x1b[2;3r\x1b[?1;6h\x1b[2;1Hbottom\x1b[HREADY\x1b[6n")
read_exact(b"\x1b[1;6R")
read_exact(b"\x1bOA")
os.write(1, b"\x1b[2;1H\nSCROLL\x1b[?1l\x1b[HNORMAL\x1b[6n")
read_exact(b"\x1b[1;7R")
read_exact(b"\x1b[D")
os.write(1, b"\x1b[H\x1b[KPASS\x1b[?6l")
