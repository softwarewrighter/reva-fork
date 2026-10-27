// Node test for www/ansi.js.  Run: node rust/reva-web/test/ansi.mjs
import { Ansi } from "../www/ansi.js";

let failed = 0;
const check = (what, got, want) => {
  const ok = JSON.stringify(got) === JSON.stringify(want);
  if (!ok) failed++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}: ${JSON.stringify(got)}`);
};

check("plain", new Ansi().feed("1 2 + . 3 ok"), [["1 2 + . 3 ok", ""]]);
check("pp line", new Ansi().feed("\x1b[1;33m:\x1b[0m \x1b[1;33msq\x1b[0m \x1b[3;90m| c\x1b[0m\n"),
  [[":", "b fg3"], [" ", ""], ["sq", "b fg3"], [" ", ""], ["| c", "i fg8"], ["\n", ""]]);
const a = new Ansi();
check("split escape, part 1", a.feed("x \x1b[3"), [["x ", ""]]);
check("split escape, part 2", a.feed("5mif\x1b[0m"), [["if", "fg5"]]);
check("other CSI dropped", new Ansi().feed("a\x1b[2Kb"), [["ab", ""]]);
check("lone ESC dropped", new Ansi().feed("a\x1bxb"), [["axb", ""]]);
check("ESC[m resets", new Ansi().feed("\x1b[31mr\x1b[mn"), [["r", "fg1"], ["n", ""]]);
process.exit(failed ? 1 : 0);
