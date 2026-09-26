"""Bounded interactive synthetic peer; never executes entered commands."""
import codecs
import os
import signal
import sys
import tty

assert all(os.isatty(fd) for fd in (0, 1, 2))
tty.setraw(0)
if "--test" in sys.argv:
    signal.alarm(20)


def write(text):
    data = text.encode("utf-8")
    while data:
        data = data[os.write(1, data):]


write("NEBULAX\r\n\r\nWelcome to your terminal.\r\n")
write("Unicode: cafe\u0301  界  👩‍💻\r\n\r\n")
write("Type a line and press Return. Arrow keys are recognized.\r\n")
write("This demo echoes text; it does not execute commands.\r\n\r\n> ")
decoder = codecs.getincrementaldecoder("utf-8")()
line = ""
escape = b""
while True:
    raw = os.read(0, 1)
    if not raw:
        break
    if escape or raw == b"\x1b":
        escape += raw
        known = {b"\x1b[A": "Up", b"\x1b[B": "Down", b"\x1b[C": "Right", b"\x1b[D": "Left", b"\x1b[H": "Home", b"\x1b[F": "End", b"\x1b[3~": "Delete"}
        if escape in known:
            write(f"\r\nKey: {known[escape]}\r\n> {line}")
            escape = b""
        elif len(escape) >= 4 or not any(key.startswith(escape) for key in known):
            escape = b""
        continue
    if raw == b"\r":
        write(f"\r\nYou typed: {line}\r\n> ")
        line = ""
        continue
    if raw == b"\x03":
        line = ""
        decoder.reset()
        write("^C\r\n> ")
        continue
    if raw == b"\x7f":
        # Clear/redraw prompt rather than guessing terminal width of deleted text.
        line = line[:-1]
        decoder.reset()
        write(f"\r\x1b[K> {line}")
        continue
    if raw == b"\t":
        continue
    char = decoder.decode(raw)
    if char and not any(ord(c) < 32 for c in char) and len(line.encode()) + len(char.encode()) <= 512:
        line += char
        write(char)
