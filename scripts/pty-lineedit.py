#!/usr/bin/env python3
"""Drive `reva` at a pseudo-terminal: line editing, history, Ctrl-R.

    scripts/pty-lineedit.py [REVA]     (default target/release/reva)

Prints one line per check, "ok ..." or "FAIL ...", and exits non-zero on a
failure.  Runs reva with REVA_HISTORY empty (history in memory) and
TERM=xterm, so nothing outside the run matters.  Unix only (pty).
"""
import os, pty, select, sys, time

REVA = sys.argv[1] if len(sys.argv) > 1 else "target/release/reva"

def run(keys):
    """Start reva, wait for its prompt, send each chunk of keys, collect output."""
    pid, fd = pty.fork()
    if pid == 0:
        env = dict(os.environ, REVA_HISTORY="", TERM="xterm", COLUMNS="80")
        os.execve(REVA, [REVA], env)
    out = b""
    def read_for(sec):
        nonlocal out
        end = time.time() + sec
        while time.time() < end:
            r, _, _ = select.select([fd], [], [], 0.05)
            if r:
                try:
                    out += os.read(fd, 65536)
                except OSError:
                    return
    read_for(1.5)
    for k in keys:
        os.write(fd, k)
        read_for(0.4)
    os.waitpid(pid, 0)
    return out.decode("utf-8", "replace")

# (what, keys sent in turn, a result, how many times it must appear)
checks = [
    ("Ctrl-A moves to the start of the line", [b"20 2 - .", b"\x01", b"100 ", b"\r", b"bye\r"], "18 ", 1),
    ("Up recalls the previous line", [b"6 7 * .\r", b"\x1b[A", b"\r", b"bye\r"], "42 ", 2),
    ("Ctrl-R finds an older line", [b"11 1 + .\r", b"3 3 + .\r", b"\x12", b"11", b"\r", b"bye\r"], "12 ", 2),
    ("Backspace deletes before the cursor", [b"5 5 * .x", b"\x7f", b"\r", b"bye\r"], "25 ", 1),
]
failed = 0
for name, keys, want, times in checks:
    out = run(keys)
    plain = "".join(c for c in out if c.isprintable() or c == " ")
    results = plain.count(want + "ok>")
    good = results >= times
    print(("ok   " if good else "FAIL ") + name)
    if not good:
        failed += 1
        print("     got:", repr(plain[-200:]), file=sys.stderr)
sys.exit(1 if failed else 0)
