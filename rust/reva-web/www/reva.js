// reva.js — drive the Reva WebAssembly module from a browser or Node.
//
// The module has no imports.  Input is pushed in, the VM runs in small
// slices (so the page never freezes, even in an infinite loop), and output
// is pulled out after each slice.
//
// Files live in memory inside the module (lib/ is built in, at /reva/lib/).
// Give `files` to bundle your own, and `storage` (e.g. localStorage) to keep
// what Forth code writes: each changed file is stored under `prefix + path`
// after every run, and restored on the next load.

export const YIELDED = 0, NEED_INPUT = 1, HALTED = 2;

// Files are bytes; Web Storage holds strings, so store one char per byte.
const toLatin1 = (bytes) => {
  let s = "";
  for (let i = 0; i < bytes.length; i += 0x8000)
    s += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
  return s;
};
const fromLatin1 = (s) => Uint8Array.from(s, (c) => c.charCodeAt(0));

export class Reva {
  /**
   * @param {WebAssembly.Instance} instance
   * @param {{files?: Object<string, string|Uint8Array>, storage?: Storage,
   *          prefix?: string}} [options]
   */
  constructor(instance, { files = {}, storage = null, prefix = "reva:fs:" } = {}) {
    this.x = instance.exports;
    this.enc = new TextEncoder();
    this.dec = new TextDecoder();
    this.storage = storage;
    this.prefix = prefix;
    for (const [path, data] of Object.entries(files)) this.put(path, data);
    if (storage) this.restore();
    this.status = this.x.reva_boot();
  }

  /** Load and boot from a URL (browser) or bytes (Node); see the constructor. */
  static async load(source, options) {
    const bytes = typeof source === "string"
      ? await (await fetch(source)).arrayBuffer()
      : source;
    const { instance } = await WebAssembly.instantiate(bytes, {});
    return new Reva(instance, options);
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
    if (!this.storage) return list.length;
    for (const { path, data } of list) {
      try {
        if (data) this.storage.setItem(this.prefix + path, toLatin1(data));
        else this.storage.removeItem(this.prefix + path);
      } catch (e) {
        console.warn(`reva: could not save ${path}:`, e);
      }
    }
    return list.length;
  }

  /** Load every file saved in `storage` into the file system. */
  restore() {
    for (let i = 0; i < this.storage.length; i++) {
      const key = this.storage.key(i);
      if (key && key.startsWith(this.prefix))
        this.put(key.slice(this.prefix.length), fromLatin1(this.storage.getItem(key)));
    }
  }

  feed(text) {
    const b = this.enc.encode(text);
    const p = this.x.reva_input_buffer(b.length);
    new Uint8Array(this.x.memory.buffer, p, b.length).set(b);
    this.x.reva_feed(b.length);
  }

  closeInput() { this.x.reva_close_input(); }

  takeOutput() {
    const n = this.x.reva_output_len();
    if (n === 0) return "";
    const p = this.x.reva_output_ptr();
    const s = this.dec.decode(new Uint8Array(this.x.memory.buffer, p, n));
    this.x.reva_output_clear();
    return s;
  }

  get exitCode() { return this.x.reva_exit_code(); }

  /** One slice of work. */
  step(budget = 200000) {
    this.status = this.x.reva_run(budget);
    return this.status;
  }

  /**
   * Run until the VM needs input or halts, handing output to `onOutput`
   * after every slice and yielding to the event loop between slices.
   */
  async run(onOutput = () => {}, budget = 200000) {
    for (;;) {
      const s = this.step(budget);
      const out = this.takeOutput();
      if (out) onOutput(out);
      if (s !== YIELDED) {
        if (this.storage) this.persist();
        return s;
      }
      await new Promise((r) => setTimeout(r, 0));
    }
  }
}
