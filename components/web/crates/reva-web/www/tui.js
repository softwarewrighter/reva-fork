// tui.js — the page's side of the TUI of IUP programs (docs/ui-model.md).
//
//   VM (worker) ──ui wait {changes}──▶ page ──changes──▶ TUI element (Yew)
//               ◀──hostreply {event}──       ◀──"key 65", "click 5"──
//               ──ui {channel:"tui"}──▶      (changes outside the loop)
//
// The VM's event loop asks with `ui` `wait` host calls carrying the
// changes since the last one; the TUI element (ui/reva_ui_web.js, built by
// scripts/build-web.sh) keeps its copy of the model, draws it as a grid of
// cells and reports keys and clicks as event lines, the next of which
// answers the wait (events that come while the VM is busy wait in turn).
// Changes made outside the loop arrive on the "tui" UI channel after each
// run slice.  While a dialog is shown the TUI takes the place of the REPL's
// output and input line in its column, so the graphics pane stays in sight.
// File dialogs are answered "cancelled" for now.

export class TuiPane {
  constructor(pane) {
    this.pane = pane;
    this.queue = [];
    this.waiter = null;
    this.mod = null;
    this.chain = Promise.resolve(); // keeps changes in order while loading
  }

  /** The TUI element, loaded and mounted on first use. */
  async load() {
    if (this.mod) return this.mod;
    const mod = await import("./ui/reva_ui_web.js");
    await mod.default();
    this.screen = this.pane.querySelector(".tui-root");
    mod.mount(this.screen, (line) => this.event(line));
    this.mod = mod;
    new ResizeObserver(() => this.fit()).observe(this.pane);
    // A paste the element did not take (WebKit may send it to the document).
    document.addEventListener("paste", (e) => {
      if (this.pane.hidden || e.defaultPrevented) return;
      e.preventDefault();
      mod.paste(e.clipboardData?.getData("text/plain") ?? "");
    });
    this.fit();
    return mod;
  }

  /** Tell the element how many cells fit the pane. */
  fit() {
    if (!this.mod || this.pane.hidden) return;
    const probe = (this.probe ??= Object.assign(document.createElement("span"), { className: "tui-probe", textContent: "M" }));
    if (!probe.isConnected) this.pane.append(probe);
    const cell = probe.getBoundingClientRect();
    const cols = Math.floor(this.pane.clientWidth / Math.max(cell.width, 1));
    const rows = Math.floor(this.pane.clientHeight / Math.max(cell.height, 1));
    this.mod.resize(Math.max(cols, 20), Math.max(rows, 5));
  }

  /**
   * IUP timers: the change lines say which elements are timers and their
   * TIME (ms) and RUN; a running one sends `timer H` every TIME ms (its
   * ACTION_CB), at most one waiting at a time, so a busy VM is not flooded.
   */
  timers(text) {
    this.timer ??= new Map();
    for (const line of text.split("\n")) {
      const [verb, h, name, value] = line.split(" ");
      const t = this.timer.get(h);
      if (verb === "created" && name === "timer") this.timer.set(h, { time: 1000, run: false, id: null });
      else if (verb === "destroyed" && t) { clearInterval(t.id); this.timer.delete(h); }
      else if (verb === "attr" && t && (name === "TIME" || name === "RUN")) {
        const v = (value ?? "").replace(/^=/, "");
        if (name === "TIME") t.time = Math.max(1, Number(v) || 1000);
        else t.run = v === "YES";
        clearInterval(t.id);
        t.id = t.run ? setInterval(() => {
          if (!this.queue.includes(`timer ${h}`)) this.event(`timer ${h}`);
        }, t.time) : null;
      }
    }
  }

  /** Apply change lines; show the pane while a dialog is shown. */
  apply(text) {
    this.timers(text);
    this.chain = this.chain.then(async () => {
      const mod = await this.load();
      this.show(mod.changes(text));
    });
    return this.chain;
  }

  /** The answer to a `ui` host call. */
  async answer(call) {
    if (call.op === "file") return { int: -1 };
    if (call.op !== "wait") return { error: `ui ${call.op} is not available on this page` };
    try {
      await this.apply(new TextDecoder().decode(Uint8Array.from(call.bytes[0] ?? [])));
    } catch (e) {
      return { error: `the TUI could not be loaded: ${e?.message ?? e}` };
    }
    if (this.queue.length) return this.reply(this.queue.shift());
    return new Promise((resolve) => (this.waiter = resolve));
  }

  /** A message on the "tui" UI channel: changes, or a reset after `restart`. */
  handle(m) {
    if (m.reset) {
      for (const t of this.timer?.values() ?? []) clearInterval(t.id);
      this.timer = new Map();
      this.chain = this.chain.then(() => { this.mod?.reset(); this.queue = []; this.show(false); });
    } else if (m.text) {
      this.apply(m.text).catch((e) => console.warn("reva: TUI:", e));
    }
  }

  /** An event line from the element: answers the wait, or waits its turn. */
  event(line) {
    const w = this.waiter;
    if (w) {
      this.waiter = null;
      w(this.reply(line));
    } else {
      this.queue.push(line);
    }
  }

  reply(line) {
    return { bytes: Array.from(new TextEncoder().encode(line)) };
  }

  /** Show the TUI in place of the REPL's output and input line, or not. */
  show(on) {
    const was = !this.pane.hidden;
    this.pane.hidden = !on;
    document.body.classList.toggle("tui-on", on);
    if (on && !was) {
      this.fit();
      this.pane.querySelector(".tui")?.focus();
    }
    if (!on && was) document.getElementById("line")?.focus();
  }
}
