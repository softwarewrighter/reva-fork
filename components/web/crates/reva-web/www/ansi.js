// ansi.js: ANSI SGR colour escapes -> styled runs, for the REPL page.
//
// Output from `pp`/`hl-type` (reva-vm highlight.rs) carries ESC[...m
// escapes; `new Ansi().feed(text)` returns runs [text, className] with the
// escapes removed, where className is made of
//   b (bold, 1)  d (dim, 2)  i (italic, 3)  u (underline, 4)
//   fgN          foreground: 30-37 -> fg0..fg7, 90-97 -> fg8..fg15
// and "" for plain text.  The page styles those classes for its light and
// dark themes (index.html), so a colour is a theme slot, not a fixed RGB,
// as in a terminal.  Other CSI sequences (cursor moves, ...) are dropped;
// an escape split across two chunks is held until the rest arrives.  No
// DOM here, so Node tests it (test/ansi.mjs).

export class Ansi {
  constructor() { this.carry = ""; this.reset(); }

  reset() { this.fg = -1; this.b = this.d = this.i = this.u = false; }

  cls() {
    const c = [];
    if (this.b) c.push("b");
    if (this.d) c.push("d");
    if (this.i) c.push("i");
    if (this.u) c.push("u");
    if (this.fg >= 0) c.push(`fg${this.fg}`);
    return c.join(" ");
  }

  sgr(params) {
    for (const p of params === "" ? [0] : params.split(";").map(Number)) {
      if (p === 0) this.reset();
      else if (p === 1) this.b = true;
      else if (p === 2) this.d = true;
      else if (p === 3) this.i = true;
      else if (p === 4) this.u = true;
      else if (p === 22) this.b = this.d = false;
      else if (p === 23) this.i = false;
      else if (p === 24) this.u = false;
      else if (p >= 30 && p <= 37) this.fg = p - 30;
      else if (p >= 90 && p <= 97) this.fg = p - 82;
      else if (p === 39) this.fg = -1;
    }
  }

  feed(text) {
    const s = this.carry + text;
    this.carry = "";
    const runs = [];
    const push = (t) => {
      if (!t) return;
      const c = this.cls();
      const last = runs[runs.length - 1];
      if (last && last[1] === c) last[0] += t; else runs.push([t, c]);
    };
    let i = 0;
    for (;;) {
      const e = s.indexOf("\x1b", i);
      if (e < 0) { push(s.slice(i)); break; }
      push(s.slice(i, e));
      const m = /^\x1b\[([0-9;?]*)([@-~])/.exec(s.slice(e, e + 64));
      if (m) {
        if (m[2] === "m") this.sgr(m[1]);
        i = e + m[0].length;
      } else if (/^\x1b(\[[0-9;?]*)?$/.test(s.slice(e)) && s.length - e < 64) {
        this.carry = s.slice(e); // the rest comes with the next chunk
        break;
      } else {
        i = e + 1; // a lone ESC: drop it
      }
    }
    return runs;
  }
}
