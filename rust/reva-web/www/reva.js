// reva.js — drive the Reva WebAssembly module from a browser or Node.
//
// Two layers:
//
// * `Reva` wraps one module instance: push input in, run a slice, pull
//   output out.  The module has no imports and never blocks; when Forth
//   wants a key that has not arrived, a slice ends with NEED_INPUT.
// * `start()` gives a page a session that keeps it responsive: the VM runs
//   in a Web Worker (reva-worker.js) in slices of about 8 ms, and the page
//   only receives output, status and changed files.  Where a worker cannot
//   start, the same loop runs on the page's thread instead.
//
// The VM boots reva.f, or starts from a saved image (`image`: bytes or a
// URL; make one with `reva --save-image` or `save` in Forth), which skips
// compiling reva.f and whatever the image was built with.  Forth's
// `restart` makes the VM again the same way, keeping files and input, and
// reads the *start input* again (`feedStart`: a page's start files, like
// the file arguments the CLI re-runs).
//
// Files live in memory inside the module (lib/ is built in, at /reva/lib/).
// Give `files` to bundle your own, and `storage` (e.g. localStorage) to keep
// what Forth code writes: each changed file is stored under `prefix + path`
// whenever the VM stops for input or halts, and restored on the next load.
//
// UI channels: the VM's turtle draws into a display list (see
// rust/reva-vm/src/canvas.rs); after each slice its new ops are handed to
// `onUi` as `{ type: "ui", channel: "canvas", ops: Float32Array }` (or
// `reset: true` after a `restart`), never one message per line.  See ui.js
// for the page's side.

export const YIELDED = 0, NEED_INPUT = 1, HALTED = 2, HOST_CALL = 3;
const BAD_IMAGE = -2;
export const RUNNING = "running";

// Files are bytes; Web Storage holds strings, so store one char per byte.
const toLatin1 = (bytes) => {
  let s = "";
  for (let i = 0; i < bytes.length; i += 0x8000)
    s += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
  return s;
};
const fromLatin1 = (s) => Uint8Array.from(s, (c) => c.charCodeAt(0));

/** Every file saved in `storage` under `prefix`, as `{ path: bytes }`. */
export function storedFiles(storage, prefix = "reva:fs:") {
  const files = {};
  for (let i = 0; i < storage.length; i++) {
    const key = storage.key(i);
    if (key && key.startsWith(prefix))
      files[key.slice(prefix.length)] = fromLatin1(storage.getItem(key));
  }
  return files;
}

/** Save a list of changes (`[{ path, data | null }]`) to `storage`. */
export function saveChanges(storage, list, prefix = "reva:fs:") {
  for (const { path, data } of list) {
    try {
      if (data) storage.setItem(prefix + path, toLatin1(data));
      else storage.removeItem(prefix + path);
    } catch (e) {
      console.warn(`reva: could not save ${path}:`, e);
    }
  }
}

/**
 * Give the event loop a turn.  A MessageChannel message runs as the next
 * task without the 4 ms minimum that nested `setTimeout(0)` calls get, but
 * some event loops (Node's, at least) run a port's messages ahead of other
 * work, so every `FAIR_MS` a timer is used instead: that is what lets a
 * worker see `input` and `stop` messages while it computes.  On a page,
 * `scheduler.yield()` (where it exists) lets the browser draw first.
 */
const FAIR_MS = 25;
const channel = typeof MessageChannel === "function" ? new MessageChannel() : null;
const waiting = [];
let lastTimer = 0;
if (channel) {
  channel.port1.onmessage = () => {
    waiting.shift()?.();
    if (!waiting.length) channel.port1.unref?.(); // Node: idle, let the process exit
  };
  channel.port1.unref?.();
}
export function yieldNow() {
  const now = performance.now();
  if (!channel || now - lastTimer >= FAIR_MS) {
    lastTimer = now;
    return new Promise((r) => setTimeout(r, 0));
  }
  // scheduler.yield() resumes ahead of other tasks, so it is only used on a
  // page (to let it draw), never in a worker that must hear `stop`.
  if (globalThis.document && globalThis.scheduler?.yield) return globalThis.scheduler.yield();
  return new Promise((r) => {
    waiting.push(r);
    channel.port1.ref?.();
    channel.port2.postMessage(0);
  });
}

const bytesOf = async (source) =>
  typeof source === "string" || source instanceof URL
    ? await (await fetch(source)).arrayBuffer()
    : source;

export class Reva {
  /**
   * @param {WebAssembly.Instance} instance
   * @param {{files?: Object<string, string|Uint8Array>, storage?: Storage,
   *          prefix?: string, image?: Uint8Array}} [options]
   */
  constructor(instance, { files = {}, storage = null, prefix = "reva:fs:", image = null } = {}) {
    this.x = instance.exports;
    this.enc = new TextEncoder();
    this.dec = new TextDecoder();
    this.storage = storage;
    this.prefix = prefix;
    for (const [path, data] of Object.entries(files)) this.put(path, data);
    if (storage) this.restore();
    this.status = image ? this.bootImage(image) : this.x.reva_boot();
  }

  bootImage(image) {
    const p = this.x.reva_fs_buffer(image.length);
    new Uint8Array(this.x.memory.buffer, p, image.length).set(image);
    const s = this.x.reva_boot_image(image.length);
    if (s === BAD_IMAGE) {
      const why = new Uint8Array(this.x.memory.buffer, this.x.reva_fs_data(), this.x.reva_fs_len());
      throw new Error(`image: ${this.dec.decode(why)}`);
    }
    return s;
  }

  /**
   * Load and start from a URL (browser) or bytes (Node); see the
   * constructor.  `options.image` may be a URL too.
   */
  static async load(source, options = {}) {
    const [bytes, image] = await Promise.all([bytesOf(source), options.image && bytesOf(options.image)]);
    const { instance } = await WebAssembly.instantiate(bytes, {});
    return new Reva(instance, { ...options, image: image ? new Uint8Array(image) : null });
  }

  /** Fetch files to bundle: `{ "/demo/x.f": "files/x.f", ... }` → `files`. */
  static async fetchFiles(urls) {
    const files = {};
    for (const [path, url] of Object.entries(urls))
      files[path] = new Uint8Array(await (await fetch(url)).arrayBuffer());
    return files;
  }

  /** Add or replace a file (not reported as a change). */
  put(path, data) {
    const p = this.enc.encode(path);
    const d = typeof data === "string" ? this.enc.encode(data) : data;
    const buf = this.x.reva_fs_buffer(p.length + d.length);
    const mem = new Uint8Array(this.x.memory.buffer, buf, p.length + d.length);
    mem.set(p);
    mem.set(d, p.length);
    this.x.reva_fs_put(p.length);
  }

  /** Files written (`data`: bytes) or removed (`data`: null) since the last call. */
  changes() {
    const n = this.x.reva_fs_changes();
    const view = new DataView(this.x.memory.buffer, this.x.reva_fs_data(), n);
    const bytes = new Uint8Array(view.buffer, view.byteOffset, n);
    const list = [];
    for (let i = 0; i < n;) {
      const kind = view.getUint8(i);
      const plen = view.getUint32(i + 1, true);
      const path = this.dec.decode(bytes.subarray(i + 5, i + 5 + plen));
      i += 5 + plen;
      const dlen = view.getUint32(i, true);
      const data = kind ? bytes.slice(i + 4, i + 4 + dlen) : null;
      i += 4 + dlen;
      list.push({ path, data });
    }
    return list;
  }

  /** Save changed files to `storage`; returns how many changed. */
  persist() {
    const list = this.changes();
    if (this.storage) saveChanges(this.storage, list, this.prefix);
    return list.length;
  }

  /** Load every file saved in `storage` into the file system. */
  restore() {
    for (const [path, data] of Object.entries(storedFiles(this.storage, this.prefix)))
      this.put(path, data);
  }

  feed(text, fn = "reva_feed") {
    const b = this.enc.encode(text);
    const p = this.x.reva_input_buffer(b.length);
    new Uint8Array(this.x.memory.buffer, p, b.length).set(b);
    this.x[fn](b.length);
  }

  /** Feed `text` now and again after every `restart`. */
  feedStart(text) { this.feed(text, "reva_feed_start"); }

  closeInput() { this.x.reva_close_input(); }

  /** Stop the running word at the next step (prints `*** interrupted`). */
  interrupt() { this.x.reva_interrupt(); }

  /** Give the VM the wall clock (it has no imports, so it cannot read one). */
  setClock(ms = Date.now()) {
    this.x.reva_set_time(Math.floor(ms / 1000), Math.floor((ms % 1000) * 1000));
  }

  takeOutput() {
    const n = this.x.reva_output_len();
    if (n === 0) return "";
    const p = this.x.reva_output_ptr();
    const s = this.dec.decode(new Uint8Array(this.x.memory.buffer, p, n));
    this.x.reva_output_clear();
    return s;
  }

  get exitCode() { return this.x.reva_exit_code(); }

  /**
   * The host call the VM waits on after `run` returned HOST_CALL
   * (rust/DESIGN.md 3.11): `{ service, op, ints, bytes }`, byte strings as
   * arrays of numbers.  Answer it with `resume`.
   */
  hostCall() {
    const p = this.x.reva_host_call_ptr(), n = this.x.reva_host_call_len();
    return JSON.parse(this.dec.decode(new Uint8Array(this.x.memory.buffer, p, n)));
  }

  /**
   * Answer the pending host call, then `run` again: `{ int }`, `{ bytes }`
   * (an array or Uint8Array), or `{ error }`, which stops the Forth line
   * with that message.
   */
  resume(reply) {
    const put = (bytes) => {
      const p = this.x.reva_fs_buffer(bytes.length);
      new Uint8Array(this.x.memory.buffer, p, bytes.length).set(bytes);
      return bytes.length;
    };
    if (reply?.error !== undefined) this.x.reva_refuse(put(new TextEncoder().encode(String(reply.error))));
    else if (reply?.bytes !== undefined) this.x.reva_resume_bytes(put(Uint8Array.from(reply.bytes)));
    else this.x.reva_resume_int(Number(reply?.int ?? 0));
  }

  /**
   * This build's facts (the page's footer): `{ name, version, copyright,
   * license, repository, build_host, build_commit, build_time }`, from
   * reva_vm::version through the module, so they match `reva -V`.
   */
  get build() {
    const p = this.x.reva_build_info_ptr(), n = this.x.reva_build_info_len();
    const text = this.dec.decode(new Uint8Array(this.x.memory.buffer, p, n));
    return Object.fromEntries(text.split("\n").filter(Boolean).map((l) => l.split("\t")));
  }

  /**
   * Hand UI messages produced since the last call to `onUi`: a reset after
   * `restart`, then the canvas ops in batches of bounded size.
   */
  flushUi(onUi) {
    if (!this.x.reva_canvas_take) return; // a module built before the canvas
    if (this.x.reva_canvas_reset()) onUi({ type: "ui", channel: "canvas", reset: true });
    for (let n; (n = this.x.reva_canvas_take());) {
      const ops = new Float32Array(this.x.memory.buffer, this.x.reva_canvas_ptr(), n).slice();
      onUi({ type: "ui", channel: "canvas", ops });
    }
  }

  /** A UI message from the page: `{ channel: "canvas", resend: true }`. */
  ui(m) {
    if (m.channel === "canvas" && m.resend) this.x.reva_canvas_resend?.();
  }

  /** One slice of at most `budget` VM steps. */
  step(budget = 200000) {
    this.status = this.x.reva_run(budget);
    return this.status;
  }

  /**
   * Run until the VM needs input or halts, in slices of about `sliceMs`
   * milliseconds (checked every `budget` steps), handing output to
   * `onOutput` after each slice and yielding to the event loop between
   * slices.  Saves changed files to `storage` when it stops.  UI messages
   * (the canvas) go to `onUi` after each slice.
   */
  async run(onOutput = () => {}, { sliceMs = 8, budget = 50000, clock = true, onUi = null } = {}) {
    for (;;) {
      if (clock) this.setClock();
      const t0 = performance.now();
      let s;
      do s = this.step(budget);
      while (s === YIELDED && performance.now() - t0 < sliceMs);
      const out = this.takeOutput();
      if (out) onOutput(out);
      if (onUi) this.flushUi(onUi);
      if (s !== YIELDED) {
        if (this.storage) this.persist();
        return s;
      }
      await yieldNow();
    }
  }
}

/**
 * Start a session for a page: the VM in a Web Worker where possible.
 *
 * Options: `image` (start from a saved image: URL or bytes), `files`
 * (bundled, `{ path: string | bytes }`), `storage` and
 * `prefix` (persistence), `onOutput(text)`, `onUi(message)` (UI channels:
 * the canvas), `onHostCall(call)` (answers the VM's host calls, see
 * `Reva.hostCall`; may return a promise), `localServices` (host-call
 * services the VM's own worker answers: ["sqlite"] loads SQLite's wasm
 * build in the worker, sqlite.js), `onStatus(status)` (RUNNING,
 * NEED_INPUT or HALTED), `worker` (false: run on this thread), `workerUrl`,
 * `createWorker(url)` (for tests), `stopTimeoutMs` (default 2000).
 *
 * Returns `{ kind, ready, build, feed(text), feedStart(text), closeInput(), stop(),
 * ui(message) (page → VM, e.g. `{ channel: "canvas", resend: true }`),
 * idle(), status,
 * exitCode, terminate() }`; `ready` resolves after boot with its output
 * already sent to `onOutput`, and `build` (see `Reva.build`) set.
 */
export function start(wasmUrl, options = {}) {
  const {
    worker = typeof Worker === "function" || !!options.createWorker,
    storage = null, prefix = "reva:fs:",
  } = options;
  const files = { ...options.files, ...(storage ? storedFiles(storage, prefix) : {}) };
  const opts = { ...options, files, storage, prefix, wasmUrl: String(wasmUrl) };
  if (worker) {
    try {
      return new WorkerSession(opts);
    } catch (e) {
      console.warn("reva: no worker, running on this thread:", e);
    }
  }
  return new LocalSession(opts);
}

class Session {
  constructor({ onOutput = () => {}, onStatus = () => {}, onUi = () => {}, onHostCall = null, localServices = [] }) {
    this.local = localServices;
    this.onOutput = onOutput;
    this.onUi = onUi;
    this.onStatus = onStatus;
    this.onHostCall = onHostCall;
    this.status = RUNNING;
    this.exitCode = 0;
    this.waiters = [];
  }
  setStatus(s, exitCode = 0) {
    this.status = s;
    this.exitCode = exitCode;
    this.onStatus(s);
    if (s !== RUNNING) for (const w of this.waiters.splice(0)) w(s);
  }
  /** The answer to a host call, from `onHostCall`, or a refusal. */
  async answer(call) {
    if (this.local?.includes(call.service)) {
      this.sqlite ??= new (await import("./sqlite.js")).SqliteHost();
      return this.sqlite.answer(call);
    }
    if (!this.onHostCall) return { error: `${call.service} is not available on this page` };
    try {
      return await this.onHostCall(call);
    } catch (e) {
      return { error: String(e?.message ?? e) };
    }
  }

  /** Resolves once the VM is waiting for input or has halted. */
  idle() {
    return this.status === RUNNING ? new Promise((r) => this.waiters.push(r)) : Promise.resolve(this.status);
  }
}

/** The VM on this thread (fallback, and for pages that ask for it). */
class LocalSession extends Session {
  constructor(o) {
    super(o);
    this.kind = "main";
    this.ready = Reva.load(o.wasmUrl, { files: o.files, image: o.image }).then(async (reva) => {
      this.reva = reva;
      this.build = reva.build;
      reva.setClock();
      const out = reva.takeOutput();
      if (out) this.onOutput(out);
      this.storage = o.storage;
      this.prefix = o.prefix;
      this.setStatus(reva.status);
    });
  }
  async pump() {
    if (this.pumping || !this.reva) return;
    this.pumping = true;
    this.setStatus(RUNNING);
    let s = await this.reva.run(this.onOutput, { onUi: this.onUi });
    while (s === HOST_CALL) {
      this.reva.resume(await this.answer(this.reva.hostCall()));
      s = await this.reva.run(this.onOutput, { onUi: this.onUi });
    }
    this.pumping = false;
    if (this.storage) saveChanges(this.storage, this.reva.changes(), this.prefix);
    this.setStatus(s, s === HALTED ? this.reva.exitCode : 0);
  }
  feed(text) { this.reva.feed(text); this.pump(); }
  feedStart(text) { this.reva.feedStart(text); this.pump(); }
  closeInput() { this.reva.closeInput(); this.pump(); }
  stop() { if (this.status === RUNNING) this.reva.interrupt(); }
  ui(m) {
    this.reva?.ui(m);
    if (!this.pumping) this.reva?.flushUi(this.onUi);
  }
  terminate() {}
}

/** The VM in a Web Worker (reva-worker.js); see the message table there. */
class WorkerSession extends Session {
  constructor(o) {
    super(o);
    this.kind = "worker";
    this.o = o;
    this.startText = "";
    this.spawn();
  }
  spawn() {
    const o = this.o;
    const url = o.workerUrl ?? new URL("./reva-worker.js", import.meta.url);
    this.w = o.createWorker ? o.createWorker(url) : new Worker(url, { type: "module" });
    this.status = RUNNING;
    let booted;
    this.ready = new Promise((r) => (booted = r));
    this.w.onmessage = ({ data: m }) => {
      switch (m.type) {
        case "output": this.onOutput(m.text); break;
        case "files": if (o.storage) saveChanges(o.storage, m.list, o.prefix); break;
        case "status":
          clearTimeout(this.stopTimer);
          if (m.build) this.build = m.build;
          this.setStatus(m.status, m.exitCode);
          if (m.boot) booted();
          break;
        case "ui": this.onUi(m); break;
        case "hostcall":
          this.answer(m.call).then((reply) => this.w.postMessage({ type: "hostreply", reply }));
          break;
        case "error": this.onOutput(`\n*** ${m.message}\n`); break;
      }
    };
    this.w.onerror = (e) => this.onOutput(`\n*** worker: ${e.message ?? e}\n`);
    // A fresh worker has a fresh, empty drawing.
    if (this.spawned) this.onUi({ type: "ui", channel: "canvas", reset: true });
    this.spawned = true;
    // Files restored from storage go in again on a restart.
    const files = o.storage ? { ...o.files, ...storedFiles(o.storage, o.prefix) } : o.files;
    const abs = (u) => (typeof u === "string" || u instanceof URL ? new URL(u, globalThis.location?.href).href : u);
    this.w.postMessage({ type: "init", wasm: o.wasmBytes ?? abs(o.wasmUrl), image: abs(o.image), files, local: o.localServices ?? [] });
    // A replacement worker (see stop) runs the start input again too.
    if (this.startText) this.w.postMessage({ type: "start", text: this.startText });
  }
  feed(text) { this.status = RUNNING; this.w.postMessage({ type: "input", text }); }
  feedStart(text) {
    this.startText += text;
    this.status = RUNNING;
    this.w.postMessage({ type: "start", text });
  }
  closeInput() { this.status = RUNNING; this.w.postMessage({ type: "close" }); }
  ui(m) { this.w.postMessage({ ...m, type: "ui" }); }
  /**
   * Interrupt the running word.  If the worker does not answer in
   * `stopTimeoutMs` (a loop inside one VM step), replace it with a fresh
   * one: files already saved survive, the session's definitions do not.
   */
  stop() {
    if (this.status !== RUNNING) return;
    this.w.postMessage({ type: "stop" });
    clearTimeout(this.stopTimer);
    this.stopTimer = setTimeout(() => {
      this.w.terminate();
      this.onOutput("\n*** not responding: restarted\n");
      this.spawn();
    }, this.o.stopTimeoutMs ?? 2000);
  }
  terminate() { clearTimeout(this.stopTimer); this.w.terminate(); }
}
