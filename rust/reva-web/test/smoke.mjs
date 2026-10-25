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

process.exit(failed ? 1 : 0);
