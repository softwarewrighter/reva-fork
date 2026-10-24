// reva.js — drive the Reva WebAssembly module from a browser or Node.
//
// The module has no imports.  Input is pushed in, the VM runs in small
// slices (so the page never freezes, even in an infinite loop), and output
// is pulled out after each slice.

export const YIELDED = 0, NEED_INPUT = 1, HALTED = 2;

export class Reva {
  /** @param {WebAssembly.Instance} instance */
  constructor(instance) {
    this.x = instance.exports;
    this.enc = new TextEncoder();
    this.dec = new TextDecoder();
    this.status = this.x.reva_boot();
  }

  /** Load and boot from a URL (browser) or bytes (Node). */
  static async load(source) {
    const bytes = typeof source === "string"
      ? await (await fetch(source)).arrayBuffer()
      : source;
    const { instance } = await WebAssembly.instantiate(bytes, {});
    return new Reva(instance);
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
      if (s !== YIELDED) return s;
      await new Promise((r) => setTimeout(r, 0));
    }
  }
}
