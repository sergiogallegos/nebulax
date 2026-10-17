"""Fixed synthetic output for shaping; no config, shell or native effects."""
import os
import signal
import tty

signal.alarm(10)
tty.setraw(0)
assert os.get_terminal_size(0) == (80, 12)
rows = {
    1: "-> => != === <= >=",
    2: "office ffi fi fl",
    3: "e\u0301 A\u0308 界 👩🏽‍💻 🇺🇸 1️⃣",
    4: "-\x1b[31m>\x1b[0m fi",
    5: "f\x1b[3mi\x1b[0m",
    6: "a\x1b[6;5Hb",
    7: "x" * 79 + "界",
    9: "abc אבג",
    10: "e" + "\u0301" * 20,
    11: "DONE",
}
data = "\x1bc" + "".join(f"\x1b[{row};1H{text}" for row, text in rows.items())
pending = data.encode()
while pending:
    pending = pending[os.write(1, pending):]
