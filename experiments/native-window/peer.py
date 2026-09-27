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


def protocol_probe():
    # Exercise the same native key path after the child changes the engine mode.
    # Fixed rows bracket a scrolling region; the reply verifies saved origin.
    write("\x1b[13;1HFIXED TOP\x1b[20;1HFIXED BOTTOM")
    write("\x1b[14;18r\x1b[?1;6h\x1b[5;1Hfirst\r\nREGION OK")
    write("\x1b7\x1b[HAPP READY\x1b8\x1b[6n")
    expected = b"\x1b[5;10R\x1bOA"
    actual = b""
    while len(actual) < len(expected):
        part = os.read(0, len(expected) - len(actual))
        assert part
        actual += part
    assert actual == expected, (actual, expected)
    write("\x1b[2;1HAPP KEY Up\x1b[?1;6l\x1b[r\x1b[22;1HPROTOCOL OK")
    write("\x1b[23;1H\x1b[1;3;4;38:2::12:34:56;48;5;230m界 RGB + indexed")
    write("\x1b[0;48;5;17m\x1b[K\x1b[0m\x1b[24;1HSTYLE OK")


write("NEBULAX\r\n\r\nWelcome to your terminal.\r\n")
write("Unicode: cafe\u0301  界  👩‍💻\r\n")
write("\x1b[1;36mBold cyan\x1b[0m  \x1b[3;38;5;214mItalic amber\x1b[0m  \x1b[4mUnderline\x1b[0m  \x1b[7mInverse\x1b[0m\r\n")
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
            if "--test" in sys.argv and escape == b"\x1b[A":
                protocol_probe()
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
