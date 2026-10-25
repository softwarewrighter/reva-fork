// host.js for the desktop app (served in place of rust/reva-web/www/host.js).
//
// Files live in a folder on disk (~/Reva, or the package).  The page keeps
// files through a Web Storage object, read once at start and written on
// every change, so this file gives it one: an in-memory copy of the folder,
// loaded through the `session` command, whose writes go back to disk
// through `write_file` / `remove_file`.  Keys are "reva:fs:" + VM path; the
// VM path `/a/b` is `<folder>/a/b`.  See reva-desktop/src/main.rs.

const PREFIX = "reva:fs:";

export async function hostSetup() {
  const invoke = globalThis.__TAURI__.core.invoke;
  const s = await invoke("session");
  const map = new Map(s.files.map(([path, data]) => [PREFIX + path, data]));
  const report = (e) => console.warn("reva-desktop:", e);
  const storage = {
    get length() { return map.size; },
    key: (i) => [...map.keys()][i] ?? null,
    getItem: (k) => map.get(k) ?? null,
    setItem(k, v) {
      map.set(k, v);
      if (k.startsWith(PREFIX)) invoke("write_file", { path: k.slice(PREFIX.length), data: v }).catch(report);
    },
    removeItem(k) {
      map.delete(k);
      if (k.startsWith(PREFIX)) invoke("remove_file", { path: k.slice(PREFIX.length) }).catch(report);
    },
  };
  return {
    storage,
    start: s.start,
    input: s.input,
    title: s.title,
    canvas: s.canvas ?? undefined,
    // files are the folder's: the page must not pick its own storage prefix
    keepsFiles: true,
    notes: [`files are kept in ${s.root}`, ...s.notes],
    output: s.print_output ? (text) => invoke("print", { text }) : undefined,
    // Let pending writes reach the disk before the window closes.
    halted: (code) => setTimeout(() => invoke("halt", { code }), 300),
    openWindow,
  };
}

// More windows (see www/ui.js): a second native window per name, opened by
// the `open_window` command (panes, as in the split view, are iframes in
// the page, handled by ui.js as in a browser); messages go down as `reva-ui-down` events and
// come up as `reva-ui-up` (the app adds `closed` when a window is closed).
// Events are JSON, so drawing ops travel as plain arrays.
const upListeners = new Map();
let listening = null;

function openWindow(name, { onMessage = () => {}, title = name, where = "window" } = {}) {
  if (where !== "window") return null;
  const { core, event } = globalThis.__TAURI__;
  listening ??= event.listen("reva-ui-up", (e) => upListeners.get(e.payload?.window)?.(e.payload));
  let closed = false;
  upListeners.set(name, (m) => {
    if (m.type === "closed") closed = true;
    onMessage(m);
  });
  core.invoke("open_window", { name, title }).catch((e) => console.warn("reva-desktop:", e));
  return {
    where: "window",
    post: (m) => event.emitTo(name, "reva-ui-down", m.ops ? { ...m, ops: Array.from(m.ops) } : m),
    closed: () => closed,
    focus: () => core.invoke("open_window", { name, title }),
  };
}
