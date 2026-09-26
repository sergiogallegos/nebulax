"""Synthetic PTY peer. Never launches a shell or reads user files/configuration."""
import os
import signal
import sys
import termios
import time
import tty

signal.alarm(10)  # Bound a failed test even if its parent is interrupted.
mode = sys.argv[1]
assert all(os.isatty(fd) for fd in (0, 1, 2))
assert os.getsid(0) == os.getpid()
assert os.tcgetpgrp(0) == os.getpgrp()
for fd in range(3, 256):
    try:
        os.fstat(fd)
    except OSError:
        continue
    raise AssertionError(f"inherited unexpected descriptor {fd}")
tty.setraw(0)


def write(data, fd=1):
    while data:
        data = data[os.write(fd, data):]


def expect(data):
    received = b""
    while len(received) < len(data):
        part = os.read(0, len(data) - len(received))
        assert part, "unexpected EOF"
        received += part
    assert received == data, (received, data)


if mode == "roundtrip":
    resized = False

    def on_resize(signum, frame):
        global resized
        resized = True

    signal.signal(signal.SIGWINCH, on_resize)
    assert os.get_terminal_size(0) == (8, 3)
    write(b"ab\x1b[6n\x1b[5n\x1b]2;ready\x07\x07")
    expect(b"\x1b[1;3R\x1b[0n")
    while not resized:
        time.sleep(0.001)
    assert os.get_terminal_size(0) == (12, 4)
    write(b"R\x1b[6n")
    expect(b"\x1b[1;4R")
    write(b"\r\nDONE")
    write(b"\r\nERR", 2)
    write(b"\xf0\x9f")  # EOF must explicitly finish the decoder.
elif mode == "effects":
    for _ in range(300):
        write(b"\x1b]2;synthetic\x07\x07\x1b[5n")
    expect(b"\x1b[0n" * 300)
    write(b"DONE")
elif mode == "eof_first":
    write(b"EOF")
    for fd in (0, 1, 2):
        os.close(fd)
    time.sleep(0.1)
    sys.exit(7)
elif mode == "idle":
    write(b"\x1b]2;idle\x07")
    while True:
        time.sleep(1)
elif mode == "nonzero":
    write(b"LAST")
    sys.exit(9)
else:
    raise AssertionError(mode)
