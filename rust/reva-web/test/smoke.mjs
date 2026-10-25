// Node smoke test for the WebAssembly build: boot reva.f, run some Forth,
// exercise pausing for input and budget yields.  Prints a transcript (the
// reg-rs baseline) and exits non-zero on any mismatch.
import { readFile } from "node:fs/promises";
import { Reva, YIELDED, NEED_INPUT, HALTED } from "../www/reva.js";

const wasm = await readFile(new URL("../www/reva.wasm", import.meta.url));
const reva = await Reva.load(wasm);
let failed = 0;
const check = (what, got, want) => {
  const ok = got === want;
  if (!ok) failed++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}: ${JSON.stringify(got)}`);
};

// The boot ends as Reva's saved executable started: banner, then a prompt.
const BANNER = "Reva 2011.2 Linux\n\nok> ";
check("boot status", reva.status, NEED_INPUT);
reva.feed("2 3 + . : sq dup * ; 7 sq .\n");
check("arith", await (async () => { let o = ""; await reva.run((s) => (o += s)); return o; })(), BANNER + "5 49 \nok> ");

reva.feed(": forever forever ; forever\n");
check("infinite loop yields", reva.step(5000), YIELDED);
check("still yielding", reva.step(5000), YIELDED);

const reva2 = await Reva.load(wasm);
reva2.feed(": k2 key key + . ; k2\n");
check("key waits", reva2.step(100000), NEED_INPUT);
reva2.feed("\x01\x02");
let out2 = "";
await reva2.run((s) => (out2 += s));
check("key resumes", out2, BANNER + "3 \nok> ");
reva2.feed("9 ~sys.(bye)\n");
check("halts", await reva2.run(), HALTED);
check("exit code", reva2.exitCode, 9);

// Files: bundled before boot, written by Forth, kept in a Storage, and
// restored by the next instance.
class MemStorage {
  constructor() { this.m = new Map(); }
  get length() { return this.m.size; }
  key(i) { return [...this.m.keys()][i] ?? null; }
  getItem(k) { return this.m.has(k) ? this.m.get(k) : null; }
  setItem(k, v) { this.m.set(k, String(v)); }
  removeItem(k) { this.m.delete(k); }
}
const storage = new MemStorage();
const reva3 = await Reva.load(wasm, { files: { "/demo/hi.f": ": hi .\" bundled\" ;" }, storage });
reva3.feed("include /demo/hi.f hi\n\" /demo/note\" creat value h \" saved\u00e9\" h write h close\n");
let out3 = "";
await reva3.run((s) => (out3 += s));
check("bundled file", out3, BANNER + "bundled\nok> \nok> ");
check("persisted", storage.getItem("reva:fs:/demo/note"), "saved\u00c3\u00a9"); // UTF-8 bytes, one char each
const reva4 = await Reva.load(wasm, { storage });
reva4.feed("\" /demo/note\" slurp type  \" /demo/note\" delete ioerr @ .\n");
let out4 = "";
await reva4.run((s) => (out4 += s));
check("restored", out4, BANNER + "saved\u00e90 \nok> ");
check("removal persisted", storage.getItem("reva:fs:/demo/note"), null);
reva4.feed("needs util/locals .\" loaded\"\n");
let out5 = "";
await reva4.run((s) => (out5 += s));
check("needs from the built-in lib/", out5, "loaded\nok> ");

process.exit(failed ? 1 : 0);
