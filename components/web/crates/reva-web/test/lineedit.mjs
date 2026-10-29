// Node test for www/lineedit.js: the editing commands, the key map,
// history (storage, per-key lists, limits) and Ctrl-R search, driven
// through a fake <input>.  Prints one line per check; exits non-zero on
// any mismatch.  Run: node components/web/crates/reva-web/test/lineedit.mjs
import { edit, keyCommand, History, lineEditor, MAX } from "../www/lineedit.js";

let failed = 0;
const check = (what, got, want) => {
  const ok = JSON.stringify(got) === JSON.stringify(want);
  if (!ok) failed++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}: ${JSON.stringify(got)}`);
};

// ---- pure edits ----
const s0 = { text: "dup swap drop", pos: 8, kill: "" };
const run = (ops, s = s0) => ops.reduce((st, op) => edit(op, st), s);
check("home/end", [edit("home", s0).pos, edit("end", s0).pos], [0, 13]);
check("word left/right", [edit("word-left", s0).pos, edit("word-right", s0).pos], [4, 13]);
check("kill-end + yank", run(["kill-end", "home", "yank"]), { text: " dropdup swap", pos: 5, kill: " drop" });
check("kill-start", edit("kill-start", s0), { text: " drop", pos: 0, kill: "dup swap" });
check("kill-word-back", edit("kill-word-back", s0), { text: "dup  drop", pos: 4, kill: "swap" });
check("kill-word", edit("kill-word", { ...s0, pos: 3 }), { text: "dup drop", pos: 3, kill: " swap" });
check("delete/backspace", [edit("delete", s0).text, edit("backspace", s0).text], ["dup swapdrop", "dup swa drop"]);
check("backspace at 0", edit("backspace", { text: "x", pos: 0 }).text, "x");
check("transpose mid", edit("transpose", { text: "abc", pos: 1 }), { text: "bac", pos: 2, kill: "" });
check("transpose end", edit("transpose", { text: "abc", pos: 3 }), { text: "acb", pos: 3, kill: "" });

// ---- keys ----
const k = (key, mods = {}) => keyCommand({ key, code: /^[a-z]$/.test(key) ? `Key${key.toUpperCase()}` : key, ...mods });
check("ctrl keys", ["a", "e", "b", "f", "k", "u", "w", "r", "p"].map((c) => k(c, { ctrlKey: true })),
  ["home", "end", "left", "right", "kill-end", "kill-start", "kill-word-back", "search-back", "prev"]);
check("alt keys (code, as on a Mac)", [keyCommand({ key: "∫", code: "KeyB", altKey: true }), k("f", { altKey: true }), k("Backspace", { altKey: true })],
  ["word-left", "word-right", "kill-word-back"]);
check("AltGr and Cmd pass", [k("e", { ctrlKey: true, altKey: true }), k("a", { metaKey: true })], [undefined, undefined]);
check("arrows", [k("ArrowUp"), k("ArrowDown"), k("ArrowLeft")], ["prev", "next", undefined]);

// ---- history ----
const store = () => {
  const m = new Map();
  return { getItem: (x) => m.get(x) ?? null, setItem: (x, v) => m.set(x, String(v)), m };
};
const st = store();
const h = new History(st, "reva:history");
for (const l of ["1 .", "", "1 .", "2 ."]) h.add(l);
check("history skips blanks/repeats", new History(st, "reva:history").entries, ["1 .", "2 ."]);
check("per-package keys", new History(st, "reva:cosy:history").entries, []);
for (let i = 0; i < MAX + 5; i++) h.add(`${i}`);
check("history bounded", [h.entries.length, h.entries[0]], [MAX, "5"]);
check("bad storage is empty", new History({ getItem: () => "{oops" }, "k").entries, []);
check("no storage works", (() => { const x = new History(null, "k"); x.add("a"); return x.entries; })(), ["a"]);

// ---- the element ----
class FakeInput {
  constructor() { this.value = ""; this.selectionStart = this.selectionEnd = 0; this.l = {}; }
  setSelectionRange(a, b) { this.selectionStart = a; this.selectionEnd = b; }
  addEventListener(t, f) { this.l[t] = f; }
  type(s) { for (const c of s) this.key(c); }
  key(key, mods = {}) {
    const e = { key, code: /^[a-z]$/i.test(key) ? `Key${key.toUpperCase()}` : key, prevented: false,
      preventDefault() { this.prevented = true; }, stopPropagation() {}, ...mods };
    this.l.keydown(e);
    if (!e.prevented && key.length === 1 && !mods.ctrlKey && !mods.altKey) {
      // what the browser does with a plain character
      const p = this.selectionStart;
      this.value = this.value.slice(0, p) + key + this.value.slice(this.selectionEnd);
      this.setSelectionRange(p + 1, p + 1);
    }
    return e;
  }
}
const input = new FakeInput();
const prompt = { textContent: "ok>" };
const ed = lineEditor(input, { prompt, storage: store(), key: "t" });
const submit = () => { const t = input.value; input.value = ""; ed.accept(t); return t; };
for (const l of ["1 2 + .", ": sq dup * ;", "7 sq ."]) { input.type(l); submit(); }
input.type("dra");
input.key("ArrowUp");
check("up recalls last", input.value, "7 sq .");
input.key("ArrowUp"); input.key("ArrowUp"); input.key("ArrowUp");
check("up stops at oldest", input.value, "1 2 + .");
input.key("ArrowDown"); input.key("ArrowDown"); input.key("ArrowDown");
check("down returns the draft", input.value, "dra");
input.key("a", { ctrlKey: true }); input.key("k", { ctrlKey: true });
check("ctrl-a ctrl-k", input.value, "");
input.key("r", { ctrlKey: true });
check("ctrl-r prompt", prompt.textContent, "(reverse-i-search)`':");
input.type("sq");
check("search finds newest", [input.value, prompt.textContent], ["7 sq .", "(reverse-i-search)`sq':"]);
input.key("r", { ctrlKey: true });
check("ctrl-r again: older", input.value, ": sq dup * ;");
input.key("r", { ctrlKey: true });
check("no older: failed", prompt.textContent, "(reverse-i-search, failed)`sq':");
input.key("e", { ctrlKey: true });
check("other key keeps the match", [input.value, input.selectionStart, prompt.textContent], [": sq dup * ;", 12, "ok>"]);
input.key("u", { ctrlKey: true });
input.type("xyz"); input.key("r", { ctrlKey: true }); input.type("+"); input.key("g", { ctrlKey: true });
check("ctrl-g restores", [input.value, prompt.textContent], ["xyz", "ok>"]);
input.key("r", { ctrlKey: true }); input.type("1 2");
const enter = input.key("Enter");
check("enter runs the match", [enter.prevented, input.value, prompt.textContent], [false, "1 2 + .", "ok>"]);

console.log(failed ? `${failed} FAILED` : "all ok");
process.exit(failed ? 1 : 0);
