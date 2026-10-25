# The REPL: history and line editing

Every Reva REPL keeps a history of the lines typed at `ok>` and edits the
current line with Emacs keys (the readline/bash defaults).  This page is
the key table both front ends follow.

| Keys | What it does |
|------|--------------|
| Up / Down, Ctrl-P / Ctrl-N | previous / next history entry (Down past the newest brings back the line you were typing) |
| Ctrl-R / Ctrl-S | search history backward / forward; type to narrow, press again for the next match |
| Enter (while searching) | run the match |
| Esc, Ctrl-G, Ctrl-C (while searching) | give up, put the line back |
| any other key (while searching) | keep the match on the line, then do that key's usual thing |
| Ctrl-A / Ctrl-E | start / end of line |
| Ctrl-B / Ctrl-F | back / forward a character |
| Alt-B / Alt-F | back / forward a word |
| Ctrl-D, Ctrl-H | delete the character under / before the cursor |
| Ctrl-K / Ctrl-U | kill to end / start of line |
| Ctrl-W, Alt-Backspace | kill the word before the cursor |
| Alt-D | kill the word after the cursor |
| Ctrl-Y | yank (paste) the last kill |
| Ctrl-T | transpose characters |

## Web page, desktop app, zips, `reva-serve`

All of these run the same page (`rust/reva-web/www/index.html`), whose
input line gets the keys from `www/lineedit.js`.

* **History** is in `localStorage`, one list per package: `reva:history`
  for plain Reva, `reva:<package id>:history` for a package (CoSy:
  `reva:cosy:history`), at most 1000 entries.  Blank lines and a repeat of
  the last entry are not added.  The list is re-read before each add, so
  two tabs add to one list rather than overwrite each other.
* **Browser-reserved keys.**  In a browser tab, Chrome and Firefox keep
  Ctrl-W (close tab), Ctrl-N (new window) and Ctrl-T (new tab) for
  themselves; use Alt-Backspace and Down instead.  In the desktop app every
  key reaches the line.  Ctrl-R is taken from the browser's reload while
  the input has focus.
* **Ctrl-C** stops a running word (as before); while searching it ends the
  search instead.
* On macOS, Alt-B/F/D work whatever character Option would type, and
  Cmd-key shortcuts are left to the system.

## Design

* **Line level, in the host.**  The VM reads whole lines: the page feeds a
  line on Enter, as a terminal in canonical mode does.  So `KEY`, `ACCEPT`
  and `REFILL` are untouched, and editing needs no VM support or wasm
  rebuild.
* **Small and dependency-free.**  `lineedit.js` is one module: pure editing
  operations (`edit`), a key map (`keyCommand`), the `History` list, and
  `lineEditor(input, …)` which joins them to the element.  The pure parts
  are tested in Node: `node rust/reva-web/test/lineedit.mjs` (in `just
  test`).
* **Never in the way.**  Storage errors (full, blocked, private mode) leave
  history in memory for the session; a broken stored value reads as empty.
