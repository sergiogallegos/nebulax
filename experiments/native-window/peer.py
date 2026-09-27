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
    write("\x1b[21;1Hstale text\x1b[2K\rTABS\tOK\x1b[1G\x1b[2I!")
    write("\x1b[16;19r\x1b[16;1HEDITxxOK\x1b[5G\x1b[2P\x1b[@")
    write("\x1b[16;1H\x1b[LINSERTED\x1b[16;1H\x1b[M\x1b[r")
    write("\x1b[19;78H\x1b[?7lABCD\x1b[?7h")
    write("\x1b[23;1H\x1b[1;3;4;38:2::12:34:56;48;5;230m界 RGB + indexed")
    write("\x1b[0;48;5;17m\x1b[K\x1b[0m\x1b[24;1HSTYLE OK\x1b[?25l")
    assert os.read(0, 1) == b"v"
    write("\x1b[?25h")
    expect(b"p")
    write("\x1b[?2004h\x1b[8;1HPASTE READY")
    expect("\x1b[200~界\ta\rb\rc\rd\x1b[201~".encode())
    write("\x1b[?2004l\x1b[11;1H\x1b[2KIM OK\x1b[11;2H\x1b[4hR\x1b[4$p\x1b[4l\x1b[4$p")
    expect(b"\x1b[4;1$y\x1b[4;2$y")
    write("\x1b[8;1H\x1b[2KPASTE OK")


def startup_probe():
    # Put both screens, history and controls into non-default states first.
    write("STALE\r\n" * 26)
    write("\x1b[31;44m\x1b[3g\x1b[?1049h\x1b[2;4r\x1b[?1;6h\x1b[?7;25lALT\x1b7")
    write("\x1b[!p\x1b[6n\x1b[?1$p\x1b[?6$p\x1b[?7$p\x1b[?25$p\x1b[?1049$p\x1b8\x1b[6n")
    expect(b"\x1b[2;4R\x1b[?1;2$y\x1b[?6;2$y\x1b[?7;1$y\x1b[?25;1$y\x1b[?1049;1$y\x1b[1;1R")
    write("\x1bc\x1b[?1049$p\t\x1b[6n\x1b[H")
    expect(b"\x1b[?1049;2$y\x1b[1;9R")
    # Do not draw the ready marker until the worker delivers every exact reply.
    write("\x1b[c\x1b[>c\x1b[5n\x1b[6n\x1b[?7$p\x1b[?25$p\x1b[?2004$p")
    expected = b"\x1b[?1;0c\x1b[>0;1;0c\x1b[0n\x1b[1;1R\x1b[?7;1$y\x1b[?25;1$y\x1b[?2004;2$y"
    expect(expected)


def expect(expected):
    actual = b""
    while len(actual) < len(expected):
        part = os.read(0, len(expected) - len(actual))
        assert part
        actual += part
    assert actual == expected, (actual, expected)


startup_probe()
write("stale display\x1b[2J\x1b[H")
write("NEBULAX\r\nSTARTUP OK / RESET OK\r\nWelcome to your terminal.\r\n")
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
