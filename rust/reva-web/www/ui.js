// ui.js — the page's side of the VM's UI channels.  Today there is one,
// "canvas" (turtle graphics); the planned UI service adds more on the same
// envelope: `{ type: "ui", channel, ... }` from the VM, and windows opened
// by name through `openWindow(host, name, options)`.
//
//   VM (worker) ──ui {channel:"canvas", ops}──▶ page ──{type:"ops"}──▶ window
//                ◀──ui {resend}──────────────       ◀──{type:"ready"}──
//
// The page keeps no copy of the drawing.  A window that (re)loads says
// "ready", and the page asks the VM to send the whole drawing again; ops
// that arrive while no window is ready are dropped, since the resend
// covers them.  So a closed, reloaded or late window always catches up,
// and the page never queues without bound.
//
// Where a window goes is the host's business (host.js): the desktop app
// opens a second native window; in a browser a popup, or, where popups
// are blocked (or `where: "pane"` asks for it), a pane below the REPL.

/**
 * Open (or find) the window `name` (`<name>.html` beside this page).
 * Options: `where` ("window", the default, or "pane"), `title`,
 * `onMessage(m)` for messages the window sends.  Returns `{ where,
 * post(m), closed(), focus() }`.
 */
export function openWindow(host, name, options = {}) {
  return host.openWindow?.(name, options) ?? browserWindow(name, options);
}

const panes = new Map();

function browserWindow(name, { where = "window", onMessage = () => {}, title = name } = {}) {
  let w = null, frame = null;
  if (where === "window") {
    try {
      w = window.open(`${name}.html`, `reva-${name}`, "popup,width=840,height=900");
    } catch {
      w = null;
    }
  }
  if (!w) {
    frame = panes.get(name);
    if (!frame?.isConnected) {
      frame = document.createElement("iframe");
      frame.className = "pane";
      frame.title = title;
      frame.src = `${name}.html?pane`;
      document.getElementById("panes").append(frame);
      panes.set(name, frame);
    }
    w = frame.contentWindow;
  }
  const listener = (e) => {
    if (e.source === w && e.data?.window === name) onMessage(e.data);
  };
  // One listener per window name: a reopened window replaces the last one.
  if (browserWindow.listeners?.[name]) removeEventListener("message", browserWindow.listeners[name]);
  (browserWindow.listeners ??= {})[name] = listener;
  addEventListener("message", listener);
  return {
    where: frame ? "pane" : "window",
    post: (m, transfer = []) => {
      if (frame) w = frame.contentWindow;
      w?.postMessage(m, "*", transfer);
    },
    closed: () => (frame ? !frame.isConnected : !w || w.closed),
    focus: () => w?.focus(),
  };
}

/**
 * The canvas channel: routes the VM's drawing ops to the graphics window,
 * opening it when the first drawing starts.  `onState(state)` reports
 * "open", "pane" or "closed" (the page shows a button to reopen); `where`
 * is where to open it ("window" or "pane").
 */
export class CanvasChannel {
  constructor(host, session, { onState = () => {}, where = "window" } = {}) {
    this.where = where;
    this.host = host;
    this.session = session;
    this.onState = onState;
    this.win = null;
    this.ready = false;
    this.used = false;
    this.lines = 0; // drawn, as the window last reported
  }

  /** A `{ channel: "canvas", ops | reset }` message from the VM. */
  handle(m) {
    if (m.reset) {
      this.lines = 0;
      if (this.ready && !this.win.closed()) this.win.post({ type: "reset" });
      return;
    }
    const newDrawing = m.ops.length > 0 && m.ops[0] === 1; // starts with CLEAR
    this.used = true;
    if (!this.win || this.win.closed()) {
      if (this.win && !newDrawing) return this.state("closed");
      this.open();
    }
    if (this.ready) this.win.post({ type: "ops", ops: m.ops }, [m.ops.buffer]);
  }

  /** Open the window (again), e.g. from a button. */
  open(where = this.where) {
    this.ready = false;
    this.win = openWindow(this.host, "canvas", {
      where,
      title: "Reva graphics",
      onMessage: (m) => this.fromWindow(m),
    });
    this.state(this.win.where === "pane" ? "pane" : "open");
  }

  fromWindow(m) {
    switch (m.type) {
      case "ready":
        this.ready = true;
        this.session.ui({ channel: "canvas", resend: true });
        break;
      case "drawn":
        this.lines = m.lines;
        break;
      case "closed":
        this.ready = false;
        this.state("closed");
        break;
    }
  }

  state(s) {
    this.onState(s);
  }
}
