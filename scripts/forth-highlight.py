#!/usr/bin/env python3
# Copyright (c) 2026 Michael A. Wright. MIT License (see components/LICENSE).
"""forth-highlight.py: color-highlight Reva Forth (and CoSy) source as HTML or PNG.

Usage:
  forth-highlight.py FILE [--lines A-B] [--title TEXT] [--no-numbers] [-o OUT.html]
  forth-highlight.py FILE [--lines A-B] [--title TEXT] --png OUT.png [--scale 2]
  ... [--table FORTH.tbl --table COSY.tbl]

Writes a self-contained HTML page (stdout by default). With --png it renders
that page with headless Chrome or Chromium (CHROME=/path overrides the search)
into a PNG sized to the code, for slides, docs and blog figures.

Highlighting is lexical, one line at a time: `|` comments to the end of the
line, `( ... )` stack comments, strings (`"`, `s"`, `."`), character literals
('F), numbers, `:` definitions and their names; other words get their class
and colour from token tables (--table, repeatable; default
lib/highlight/forth.tbl, the table the REPLs' `pp` uses, see
docs/highlight.md). Python 3 standard library only.
"""
import argparse, glob, html, os, re, shutil, subprocess, sys, tempfile

LEXICAL = ("com", "stk", "str", "num", "colon", "def")
HERE = os.path.dirname(os.path.abspath(__file__))

def default_tables():
    """reva-fork's lib/highlight/forth.tbl, then any *.tbl beside this script.

    The same script runs from reva-fork (scripts/) and from a package
    such as ba-cosy-zip-fork (reva-fork-subset/, with reva-fork beside the
    checkout or named by $REVA_FORK); a package's own table sits beside it.
    """
    forth = [os.path.join(HERE, "..", "lib", "highlight", "forth.tbl"),
             os.path.join(HERE, "..", "..", "reva-fork", "lib", "highlight", "forth.tbl")]
    if os.environ.get("REVA_FORK"):
        forth.insert(0, os.path.join(os.environ["REVA_FORK"], "lib", "highlight", "forth.tbl"))
    found = [p for p in forth if os.path.isfile(p)][:1]
    if not found:
        sys.exit("forth-highlight: reva-fork's lib/highlight/forth.tbl not found; "
                 "set REVA_FORK=/path/to/reva-fork or pass --table")
    return found + sorted(glob.glob(os.path.join(HERE, "*.tbl")))
STYLES, WORDS = {}, {}

def load_table(path):
    """Add a token table (lib/highlight/forth.tbl's format; later lines win)."""
    for line in open(path, encoding="utf-8").read().splitlines():
        f = line.split()
        if len(f) >= 3 and f[0] == "style":
            STYLES[f[1]] = " ".join(f[3:])
        elif len(f) >= 2 and f[0] == "words":
            for w in f[2:]:
                WORDS[w] = f[1]

def tokens(line):
    out, i, n, naming = [], 0, len(line), False
    while i < n:
        m = re.match(r"[ \t]+", line[i:])
        if m:
            out.append(("ws", m.group())); i += len(m.group()); continue
        w = re.match(r"\S+", line[i:]).group()
        if w == "|":
            out.append(("com", line[i:])); break
        if w == "(":
            j = line.find(")", i); j = n - 1 if j < 0 else j
            out.append(("stk", line[i:j + 1])); i = j + 1; continue
        if w in ('"', 's"', '."'):
            j = line.find('"', i + len(w)); j = n - 1 if j < 0 else j
            out.append(("str", line[i:j + 1])); i = j + 1; continue
        if naming:
            kind, naming = "def", False
        elif w == ":":
            kind, naming = "colon", True
        elif w in (";", ";then"):
            kind = "colon"
        elif re.fullmatch(r"'\S", w):
            kind = "str"
        elif re.fullmatch(r"-?\d+|\$[0-9a-fA-F]+", w):
            kind = "num"
        else:
            kind = WORDS.get(w) or WORDS.get(w.lower()) or "w"
        out.append((kind, w)); i += len(w)
    return out

CSS = """body{margin:0;background:#16181d}
#code{display:inline-block;background:#16181d;color:#d7dae0;padding:16px 22px 18px;
 font-family:"JetBrains Mono",Menlo,"DejaVu Sans Mono",Consolas,monospace;font-size:15px;line-height:1.45}
.title{font-family:"Helvetica Neue",Arial,sans-serif;font-size:14px;color:#f5c518;margin-bottom:10px}
.ln{white-space:pre}.no{display:inline-block;width:2.2em;color:#4b5160;text-align:right;margin-right:1em;user-select:none}
"""

def page(lines, first, title, numbers):
    rows = []
    for k, line in enumerate(lines, first):
        no = '<span class="no">%d</span>' % k if numbers else ""
        body = "".join('<span class="%s">%s</span>' % (t, html.escape(s)) for t, s in tokens(line))
        rows.append('<div class="ln">%s%s</div>' % (no, body))
    head = '<div class="title">%s</div>' % html.escape(title) if title else ""
    return ('<!doctype html><meta charset="utf-8"><style>%s</style><div id="code">%s%s</div>\n'
            % (CSS + "".join(".%s{%s}" % kv for kv in STYLES.items()), head, "".join(rows)))

def chrome():
    for c in [os.environ.get("CHROME"), "google-chrome", "chromium", "chromium-browser", "chrome",
              "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
              "/Applications/Chromium.app/Contents/MacOS/Chromium"]:
        if c and (shutil.which(c) or os.path.exists(c)):
            return shutil.which(c) or c
    sys.exit("forth-highlight: no Chrome or Chromium found for --png (set CHROME=/path)")

def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("file")
    ap.add_argument("--lines", help="A-B, 1-based and inclusive")
    ap.add_argument("--title", default="")
    ap.add_argument("--no-numbers", action="store_true")
    ap.add_argument("-o", "--output")
    ap.add_argument("--png")
    ap.add_argument("--scale", type=float, default=2.0)
    ap.add_argument("--table", action="append", default=[],
                    help="token table, repeatable (default: reva-fork's lib/highlight/forth.tbl, "
                         "then *.tbl beside this script)")
    a = ap.parse_args()
    for t in a.table or default_tables():
        load_table(t)
    text = open(a.file, encoding="utf-8", errors="replace").read().split("\n")
    first, last = 1, len(text)
    if a.lines:
        first, last = (int(x) for x in a.lines.split("-"))
    lines = [l.rstrip() for l in text[first - 1:last]]
    doc = page(lines, first, a.title, not a.no_numbers)
    if not a.png:
        (open(a.output, "w", encoding="utf-8") if a.output else sys.stdout).write(doc)
        return
    # The window is sized from the text: 15px monospace is about 9.05px a column, 21.75px a line.
    cols = max([len(l) for l in lines] + [len(a.title) * 0.8]) + (0 if a.no_numbers else 4)
    w = int(cols * 9.05 + 50); h = int(len(lines) * 21.75 + (60 if a.title else 36))
    with tempfile.TemporaryDirectory() as d:
        src = os.path.join(d, "code.html")
        open(src, "w", encoding="utf-8").write(doc)
        subprocess.run([chrome(), "--headless=new", "--disable-gpu", "--hide-scrollbars",
                        "--force-device-scale-factor=%g" % a.scale, "--window-size=%d,%d" % (w, h),
                        "--screenshot=" + os.path.abspath(a.png), "file://" + src],
                       check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    print(a.png)

if __name__ == "__main__":
    main()
