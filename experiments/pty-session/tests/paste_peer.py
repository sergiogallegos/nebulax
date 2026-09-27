"""Exact paste bytes through a real PTY, including soft/hard reset negotiation."""
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


payload = "界\ta\rb\rc\rd".encode()
for stage, (setup, state) in enumerate([
    (b"\x1b[?2004h", 1),
    (b"\x1b[!p", 2),
    (b"\x1b[?2004h\x1bc", 2),
]):
    write(setup + b"\x1b[?2004$p")
    expect(f"\x1b[?2004;{state}$y".encode())
    write(f"\x1b[2J\x1b[HPASTE {stage}".encode())
    expected = b"\x1b[200~" + payload + b"\x1b[201~" if state == 1 else payload
    expect(expected)
write(b"\x1b[2J\x1b[HPASTE OK")
