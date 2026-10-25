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
    notes: [`files are kept in ${s.root}`, ...s.notes],
    output: s.print_output ? (text) => invoke("print", { text }) : undefined,
    // Let pending writes reach the disk before the window closes.
    halted: (code) => setTimeout(() => invoke("halt", { code }), 300),
  };
}
