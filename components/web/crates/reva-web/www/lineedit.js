// lineedit.js — history and Emacs-style editing for the REPL's input line.
//
// The VM reads whole lines here (the page feeds a line on Enter), so
// editing is the page's job, as it is the terminal's on the command line:
// KEY and ACCEPT see only finished lines.  The CLI's editor (reva-cli
// src/lineedit.rs) has the same keys; docs/repl.md has the table.
//
//   Up/Down, Ctrl-P/N      previous/next history entry
//   Ctrl-R / Ctrl-S        search history back / forward (again: next match;
//                          Enter runs it, Esc or Ctrl-G puts the line back,
//                          other keys keep it for editing)
//   Ctrl-A/E, Ctrl-B/F     start/end of line, back/forward a character
//   Alt-B/F                back/forward a word
//   Ctrl-D, Ctrl-H         delete the character under/before the cursor
//   Ctrl-K/U               kill to end/start of line
//   Ctrl-W, Alt-Backspace  kill the word before the cursor
//   Alt-D                  kill the word after the cursor
//   Ctrl-Y                 yank (paste) the last kill
//   Ctrl-T                 transpose characters
//
// The browser's own keys win where it does not let a page have them:
// Ctrl-W (close tab), Ctrl-N (new window) and Ctrl-T (new tab) in a
// browser tab; in the desktop app they all reach the line.
//
// History is kept in localStorage under `key` (index.html uses
// `reva:history`, or `reva:<package id>:history` so each package has its
// own), at most MAX entries, re-read before each write so two tabs only
// add to each other's list.  Storage errors are ignored: history is a
// convenience and must never stop the REPL.

/** Entries kept. */
export const MAX = 1000;

// ---- pure editing operations on { text, pos } ----

const isWord = (c) => /[\p{L}\p{N}_]/u.test(c);

/** Index of the start of the word before `pos`. */
export function wordLeft(text, pos) {
  let i = pos;
  while (i > 0 && !isWord(text[i - 1])) i--;
  while (i > 0 && isWord(text[i - 1])) i--;
  return i;
}

/** Index just past the end of the word after `pos`. */
export function wordRight(text, pos) {
  let i = pos;
  while (i < text.length && !isWord(text[i])) i++;
  while (i < text.length && isWord(text[i])) i++;
  return i;
}

/**
 * Apply editing command `op` to `s = { text, pos, kill }`; returns the
 * new state (`kill` is the kill buffer, for Ctrl-Y), or null for an
 * unknown op.
 */
export function edit(op, { text, pos, kill = "" }) {
  const cut = (a, b) => ({ text: text.slice(0, a) + text.slice(b), pos: a, kill: text.slice(a, b) || kill });
  switch (op) {
    case "home": return { text, pos: 0, kill };
    case "end": return { text, pos: text.length, kill };
    case "left": return { text, pos: Math.max(0, pos - 1), kill };
    case "right": return { text, pos: Math.min(text.length, pos + 1), kill };
    case "word-left": return { text, pos: wordLeft(text, pos), kill };
    case "word-right": return { text, pos: wordRight(text, pos), kill };
    case "delete": return { text: text.slice(0, pos) + text.slice(pos + 1), pos, kill };
    case "backspace": return pos ? { text: text.slice(0, pos - 1) + text.slice(pos), pos: pos - 1, kill } : { text, pos, kill };
    case "kill-end": return cut(pos, text.length);
    case "kill-start": return cut(0, pos);
    case "kill-word-back": return cut(wordLeft(text, pos), pos);
    case "kill-word": return { ...cut(pos, wordRight(text, pos)), pos };
    case "yank": return { text: text.slice(0, pos) + kill + text.slice(pos), pos: pos + kill.length, kill };
    case "transpose": {
      if (text.length < 2 || pos === 0) return { text, pos, kill };
      const p = pos === text.length ? pos - 1 : pos; // at the end: swap the last two
      const t = text.slice(0, p - 1) + text[p] + text[p - 1] + text.slice(p + 1);
      return { text: t, pos: p + 1, kill };
    }
    default: return null;
  }
}

/** The command for a keydown event, or undefined. */
export function keyCommand(e) {
  if (e.ctrlKey && e.altKey) return undefined; // AltGr on Windows: a character
  if (e.metaKey) return undefined; // Cmd-A etc. stay the system's
  const letter = /^Key[A-Z]$/.test(e.code ?? "") ? e.code[3].toLowerCase() : e.key?.toLowerCase();
  if (e.ctrlKey) {
    return {
      a: "home", e: "end", b: "left", f: "right", d: "delete", h: "backspace",
      k: "kill-end", u: "kill-start", w: "kill-word-back", y: "yank", t: "transpose",
      p: "prev", n: "next", r: "search-back", s: "search-forward", g: "cancel",
    }[letter];
  }
  if (e.altKey) {
    if (e.key === "Backspace") return "kill-word-back";
    return { b: "word-left", f: "word-right", d: "kill-word" }[letter];
  }
  if (e.shiftKey) return undefined;
  return { ArrowUp: "prev", ArrowDown: "next", Escape: "cancel" }[e.key];
}

// ---- history ----

/** The history list in `storage` under `key` (oldest first). */
export class History {
  constructor(storage, key) {
    this.storage = storage;
    this.key = key;
    this.entries = this.load();
  }

  load() {
    try {
      const v = JSON.parse(this.storage?.getItem(this.key) ?? "[]");
      return Array.isArray(v) ? v.filter((x) => typeof x === "string").slice(-MAX) : [];
    } catch {
      return [];
    }
  }

  /** Record `line`, unless blank or a repeat of the last entry. */
  add(line) {
    if (!line.trim()) return;
    this.entries = this.load();
    if (this.entries.at(-1) === line) return;
    this.entries.push(line);
    this.entries = this.entries.slice(-MAX);
    try {
      this.storage?.setItem(this.key, JSON.stringify(this.entries));
    } catch {
      // full or blocked storage: keep this session's list in memory
    }
  }

  /** Newest entry before `from` (exclusive) containing `q`, or -1. */
  searchBack(q, from) {
    for (let i = Math.min(from, this.entries.length) - 1; i >= 0; i--) if (this.entries[i].includes(q)) return i;
    return -1;
  }

  /** Oldest entry after `from` (exclusive) containing `q`, or -1. */
  searchForward(q, from) {
    for (let i = from + 1; i < this.entries.length; i++) if (this.entries[i].includes(q)) return i;
    return -1;
  }
}

// ---- the input element ----

/**
 * Give `input` (an <input>) history and Emacs keys.  `prompt` is the
 * element showing "ok>", which shows the search while Ctrl-R is active.
 * Call `.accept(line)` from the form's submit handler.
 */
export function lineEditor(input, { prompt, storage = safeLocalStorage(), key = "reva:history" } = {}) {
  const history = new History(storage, key);
  const promptText = prompt?.textContent ?? "";
  let index = history.entries.length; // history.entries.length: the draft
  let draft = "";
  let kill = "";
  let search = null; // { dir, query, at, original }

  const set = (text, pos = text.length) => {
    input.value = text;
    input.setSelectionRange?.(pos, pos);
  };
  const show = () => {
    if (!prompt) return;
    prompt.textContent = search
      ? `(${search.dir < 0 ? "reverse" : "forward"}-i-search${search.failed ? ", failed" : ""})\`${search.query}':`
      : promptText;
  };
  const go = (i) => {
    if (i < 0 || i > history.entries.length || i === index) return;
    if (index === history.entries.length) draft = input.value;
    index = i;
    set(i === history.entries.length ? draft : history.entries[i]);
  };
  const find = (dir, from) => {
    const q = search.query;
    const i = dir < 0 ? history.searchBack(q, from) : history.searchForward(q, from);
    search.failed = i < 0;
    if (i >= 0) {
      search.at = i;
      const text = history.entries[i];
      const p = text.indexOf(q);
      input.value = text;
      input.setSelectionRange?.(p, p + q.length);
    }
    show();
  };
  const startSearch = (dir) => {
    if (search) {
      search.dir = dir;
      if (search.query) return find(dir, search.at);
      search.query = search.last ?? "";
      if (search.query) return find(dir, search.at);
      return show();
    }
    search = { dir, query: "", at: dir < 0 ? history.entries.length : -1, original: input.value, last: startSearch.last };
    show();
  };
  const endSearch = (restore) => {
    if (!search) return;
    startSearch.last = search.query || startSearch.last;
    if (restore) set(search.original);
    else set(input.value, input.value.length);
    search = null;
    show();
  };

  input.addEventListener("keydown", (e) => {
    const cmd = keyCommand(e);
    if (search) {
      if (cmd === "search-back" || cmd === "search-forward") {
        e.preventDefault();
        return startSearch(cmd === "search-back" ? -1 : 1);
      }
      if (cmd === "cancel" || (e.ctrlKey && e.key === "c")) {
        e.preventDefault();
        e.stopPropagation(); // Ctrl-C here ends the search, not the program
        return endSearch(true);
      }
      if (e.key === "Backspace" && !e.ctrlKey && !e.altKey) {
        e.preventDefault();
        search.query = search.query.slice(0, -1);
        if (!search.query) return show();
        return find(search.dir, search.dir < 0 ? history.entries.length : -1);
      }
      if (e.key.length === 1 && !e.ctrlKey && !e.altKey && !e.metaKey) {
        e.preventDefault();
        search.query += e.key;
        // the current match still matches? stay on it; else look on
        const cur = history.entries[search.at];
        if (cur?.includes(search.query)) return find(search.dir, search.dir < 0 ? search.at + 1 : search.at - 1);
        return find(search.dir, search.at);
      }
      if (e.key === "Shift" || e.key === "Control" || e.key === "Alt" || e.key === "Meta") return;
      // Enter runs the match (the form submits); anything else keeps it
      // on the line and then does its usual thing.
      endSearch(false);
      if (e.key === "Enter") return;
    }
    if (!cmd) return;
    e.preventDefault();
    switch (cmd) {
      case "prev": return go(index - 1);
      case "next": return go(index + 1);
      case "search-back": return startSearch(-1);
      case "search-forward": return startSearch(1);
      case "cancel": return;
    }
    const pos = input.selectionStart ?? input.value.length;
    const end = input.selectionEnd ?? pos;
    // With a selection, deleting commands take the selection, as usual.
    if (end !== pos && ["delete", "backspace"].includes(cmd)) {
      return set(input.value.slice(0, pos) + input.value.slice(end), pos);
    }
    const s = edit(cmd, { text: input.value, pos, kill });
    if (!s) return;
    kill = s.kill;
    set(s.text, s.pos);
  });
  // Typing in a recalled line edits the line; the draft comes back with Down.
  input.addEventListener("blur", () => endSearch(false));

  return {
    history,
    /** Record a submitted line and go back to an empty draft. */
    accept(line) {
      endSearch(false);
      history.add(line);
      index = history.entries.length;
      draft = "";
    },
    /** True while Ctrl-R search is active. */
    searching: () => !!search,
  };
}

function safeLocalStorage() {
  try {
    return globalThis.localStorage ?? null;
  } catch {
    return null; // blocked (sandboxed frame, privacy settings)
  }
}
