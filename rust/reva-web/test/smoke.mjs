// Node smoke test for the WebAssembly build: boot reva.f, run some Forth,
// exercise pausing for input and budget yields, save an image and start
// from it, restart.  Prints a transcript (the
// reg-rs baseline) and exits non-zero on any mismatch.
import { readFile } from "node:fs/promises";
import { Worker } from "node:worker_threads";
import { Reva, start, YIELDED, NEED_INPUT, HALTED } from "../www/reva.js";

const wasm = await readFile(new URL("../www/reva.wasm", import.meta.url));
const reva = await Reva.load(wasm);
let failed = 0;
const check = (what, got, want) => {
  const ok = JSON.stringify(got) === JSON.stringify(want);
  if (!ok) failed++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}: ${JSON.stringify(got)}`);
};

// The boot ends as Reva's saved executable started: banner, then a prompt.
const BANNER = "reva-fork 2026.1025.0 portable\n\nok> ";
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

// Interrupting a runaway loop keeps the dictionary.
reva4.feed(": kept 42 ; : spin repeat again ; spin\n");
check("spinning", reva4.step(10000), YIELDED);
reva4.interrupt();
let out6 = "";
await reva4.run((s) => (out6 += s));
check("interrupted", out6.startsWith("\n*** interrupted (in spin"), true);
reva4.feed("kept .\n");
let out7 = "";
await reva4.run((s) => (out7 += s));
check("dictionary kept", out7, "42 \nok> ");

// Slicing changes when output appears, never what it is.
const transcript = async (budget) => {
  const r = await Reva.load(wasm);
  r.feed(": tri 0 swap 0 do i + loop ; 1000 0 do i tri drop loop 200000 tri . 10 0 do i . loop cr bye\n");
  let o = "";
  await r.run((s) => (o += s), { budget, sliceMs: 1, clock: false });
  return o;
};
check("same output for 1k and 1M step slices", await transcript(1000) === await transcript(1000000), true);

// An image saved by Forth (`save`) starts a new instance without reva.f's
// compile, and `restart` makes the VM again from the same image.
const maker = await Reva.load(wasm);
maker.feed(": hi .\" from the image\" ; save /app.img\n");
await maker.run();
const image = maker.changes().find((c) => c.path === "/app.img")?.data;
check("save writes an image", new TextDecoder().decode(image?.subarray(0, 8)), "REVAVX86");
const fromImage = await Reva.load(wasm, { image });
check("starts from the image", fromImage.takeOutput(), BANNER);
fromImage.feed(": gone ; hi cr restart\nhi cr gone\n");
let out8 = "";
await fromImage.run((s) => (out8 += s));
check("restart", out8.replace(/\.\.\.[^]*$/, "..."), "from the image\n" + BANNER + "from the image\n\nERROR: unknown word:  gone\n...");
let badImage = "";
try { await Reva.load(wasm, { image: new Uint8Array(4) }); } catch (e) { badImage = e.message; }
check("not an image", badImage, "image: image is truncated");

// A session in a worker thread, as a page runs it in a Web Worker.
const nodeWorker = (url) => {
  const w = new Worker(new URL("./node-worker.mjs", import.meta.url));
  const adapter = {
    postMessage: (m, t) => w.postMessage(m, t),
    terminate: () => w.terminate(),
    set onmessage(f) { w.on("message", (data) => f({ data })); },
    set onerror(f) { w.on("error", f); },
  };
  return adapter;
};
const storage2 = new MemStorage();
let wout = "";
const session = start("reva.wasm", {
  createWorker: nodeWorker, storage: storage2, wasmBytes: wasm,
  files: { "/demo/w.f": ": w .\" from the worker\" ;" },
  onOutput: (s) => (wout += s),
});
await session.ready;
check("worker boots", [session.kind, session.status, wout], ["worker", NEED_INPUT, BANNER]);
wout = "";
session.feed("include /demo/w.f w\n\" /demo/out\" creat value h \" ok\" h write h close\n");
check("worker runs", await session.idle(), NEED_INPUT);
check("worker output", wout, "from the worker\nok> \nok> ");
check("worker files saved by the page", storage2.getItem("reva:fs:/demo/out"), "ok");
wout = "";
session.feed(": spin repeat again ; spin\n");
await new Promise((r) => setTimeout(r, 50));
session.stop();
check("worker stop", await session.idle(), NEED_INPUT);
check("worker interrupted", wout.startsWith("\n*** interrupted (in spin"), true);
session.feed("7 ~sys.(bye)\n");
check("worker halts", [await session.idle(), session.exitCode], [HALTED, 7]);
session.terminate();

let iout = "";
const imageSession = start("reva.wasm", {
  createWorker: nodeWorker, wasmBytes: wasm, image, onOutput: (s) => (iout += s),
});
await imageSession.ready;
imageSession.feed("hi cr\n");
await imageSession.idle();
check("worker starts from an image", iout, BANNER + "from the image\n\nok> ");
imageSession.terminate();

process.exit(failed ? 1 : 0);
