// sqlite.js — SQLite for lib/db/sqlite in the browser (rust/DESIGN.md 3.11).
//
// The VM's `sqlite` host calls (see Reva.hostCall) are answered here, in
// the VM's worker, with the same operations, handles and result codes as
// the reva-sqlite crate the CLI and the desktop app use.
//
// SQLite itself is not redistributed: its official WebAssembly build is
// loaded at run time from jsDelivr, pinned to one version, the first time a
// program uses SQLite.  Third-party notice: SQLite is in the public domain;
// the @sqlite.org/sqlite-wasm package is Apache-2.0 (docs/licenses.md).
//
// Persistence: each database runs in memory; opening one loads its file
// from the browser's private file system (OPFS, directory reva-sqlite/),
// and after every committed change the whole database is written back
// (sqlite3_js_db_export, then a writable stream).  Nothing holds a file
// open, so a reload, a page kept in the back/forward cache or a second tab
// never blocks another (SQLite's own OPFS VFSes need exclusive handles or
// cross-origin-isolation headers that GitHub Pages cannot send).  Without
// OPFS, databases last for the session.

export const SQLITE_WASM =
  "https://cdn.jsdelivr.net/npm/@sqlite.org/sqlite-wasm@3.54.1-build1/dist/index.mjs";

const OK = 0, ERROR = 1, MISUSE = 21, ROW = 100, DONE = 101;
const dec = new TextDecoder(), enc = new TextEncoder();

/** Answers `sqlite` host calls; `load` imports SQLite (tests replace it). */
export class SqliteHost {
  constructor({ load = () => import(SQLITE_WASM) } = {}) {
    this.load = load;
    this.ready = null;
    this.handles = new Map();
    this.next = 1;
    this.rc = OK;
  }

  /** `{ int }`, `{ bytes }` or `{ error }` for `call` (see Reva.resume). */
  async answer(call) {
    try {
      await (this.ready ??= this.start());
    } catch (e) {
      this.ready = null;
      return { error: `SQLite could not be loaded (offline?): ${e?.message ?? e}` };
    }
    const [a = 0, b = 0, c = 0] = call.ints;
    const text = (k) => dec.decode(Uint8Array.from(call.bytes[k] ?? []));
    const op = OPS[call.op];
    if (!op) return { error: `SQLite in the browser has no operation ${call.op}` };
    return await op.call(this, { a, b, c, text, bytes: (k) => Uint8Array.from(call.bytes[k] ?? []) });
  }

  async start() {
    const init = (await this.load()).default;
    this.sqlite3 = await init({ print: () => {}, printErr: () => {} });
    try {
      const root = await navigator.storage.getDirectory();
      this.dir = await root.getDirectoryHandle("reva-sqlite", { create: true });
    } catch {
      this.dir = null; // no OPFS: databases last for the session
    }
  }

  /** The OPFS file name for a database path (one flat directory). */
  static fileName(path) {
    return encodeURIComponent(path.replace(/^\/+/, ""));
  }

  /** Load `db`'s saved image from OPFS, if there is one. */
  async restore(db, file) {
    let bytes;
    try {
      const f = await (await this.dir.getFileHandle(file)).getFile();
      bytes = new Uint8Array(await f.arrayBuffer());
    } catch {
      return; // a new database
    }
    if (bytes.length === 0) return;
    const { capi, wasm } = this.sqlite3;
    const p = wasm.allocFromTypedArray(bytes);
    const rc = capi.sqlite3_deserialize(db.pointer, "main", p, bytes.length, bytes.length,
      capi.SQLITE_DESERIALIZE_FREEONCLOSE | capi.SQLITE_DESERIALIZE_RESIZEABLE);
    db.checkRc(rc);
  }

  /** Write database handle `h` back to OPFS, unless a transaction is open. */
  async persist(h) {
    const d = this.handles.get(h);
    if (!d?.file || !this.dir) return;
    if (!this.sqlite3.capi.sqlite3_get_autocommit(d.value.pointer)) return;
    const bytes = this.sqlite3.capi.sqlite3_js_db_export(d.value);
    const w = await (await this.dir.getFileHandle(d.file, { create: true })).createWritable();
    await w.write(bytes);
    await w.close();
  }

  /** A database: in memory, loaded from its OPFS file if it has one. */
  async open(name) {
    const db = new this.sqlite3.oo1.DB(":memory:", "ct");
    if (name === ":memory:" || !this.dir) return { db, file: null };
    const path = "/" + name.replace(/^\/+/, "");
    if (path.split("/").includes("..")) {
      db.close();
      throw Object.assign(new Error("a database outside the store"), { resultCode: ERROR });
    }
    const file = SqliteHost.fileName(path);
    await this.restore(db, file);
    return { db, file };
  }

  /** Keep `rc` and return `v`; on an exception, the code and message. */
  run(db, f) {
    try {
      this.rc = OK;
      return f();
    } catch (e) {
      this.rc = e?.resultCode ? e.resultCode & 0xff : ERROR;
      const d = this.handles.get(db);
      if (d) d.errmsg = String(e?.message ?? e).replace(/^SQLITE_[A-Z_]+: (sqlite3 result code \d+: )?/, "");
      return null;
    }
  }

  add(kind, value, extra = {}) {
    const h = this.next++;
    this.handles.set(h, { kind, value, ...extra });
    return h;
  }

  get(h, kind) {
    const v = this.handles.get(h);
    return v?.kind === kind ? v : null;
  }
}

/** A value as the bytes `column_text` gives (numbers as text, NULL as nothing). */
function toBytes(v) {
  if (v === null || v === undefined) return [];
  if (v instanceof Uint8Array) return Array.from(v);
  return Array.from(enc.encode(String(v)));
}

// The operations, named as lib/db/sqlite's rsql_ functions; `this` is the
// SqliteHost, the arguments come as in reva-sqlite.
const OPS = {
  rc() { return { int: this.rc }; },
  version() { return { bytes: Array.from(enc.encode(this.sqlite3.version.libVersion)) }; },
  async open({ text }) {
    try {
      const { db, file } = await this.open(text(0));
      this.rc = OK;
      return { int: this.add("db", db, { errmsg: "not an error", file }) };
    } catch (e) {
      this.rc = e?.resultCode ? e.resultCode & 0xff : ERROR;
      return { int: 0 };
    }
  },
  close({ a }) {
    const d = this.get(a, "db");
    if (!d) return (this.rc = MISUSE), { int: MISUSE };
    for (const [h, v] of this.handles) if (v.db === a) { v.value.finalize?.(); this.handles.delete(h); }
    d.value.close();
    this.handles.delete(a);
    return (this.rc = OK), { int: OK };
  },
  async exec({ a, text }) {
    const d = this.get(a, "db");
    if (!d) return (this.rc = MISUSE), { int: MISUSE };
    this.run(a, () => d.value.exec(text(0)));
    const rc = this.rc;
    await this.persist(a);
    return (this.rc = rc), { int: rc };
  },
  lastid({ a }) {
    const d = this.get(a, "db");
    return { int: d ? Number(this.sqlite3.capi.sqlite3_last_insert_rowid(d.value.pointer)) : 0 };
  },
  errmsg({ a }) {
    return { bytes: Array.from(enc.encode(this.get(a, "db")?.errmsg ?? "bad database handle")) };
  },
  query({ a, text }) {
    const d = this.get(a, "db");
    if (!d) return (this.rc = MISUSE), { int: 0 };
    const rows = this.run(a, () => d.value.exec({ sql: text(0), rowMode: "array", returnValue: "resultRows" }));
    return { int: rows ? this.add("query", null, { db: a, rows, row: [] }) : 0 };
  },
  next({ a }) {
    const q = this.get(a, "query");
    if (!q || q.rows.length === 0) return { int: -1 };
    q.row = q.rows.shift();
    return { int: q.row.length };
  },
  getcol({ a, b }) { return { bytes: toBytes(this.get(a, "query")?.row[b]) }; },
  endquery({ a }) { this.handles.delete(a); return { int: OK }; },
  prepare({ a, text }) {
    const d = this.get(a, "db");
    if (!d) return (this.rc = MISUSE), { int: 0 };
    const st = this.run(a, () => d.value.prepare(text(0)));
    return { int: st ? this.add("stmt", st, { db: a }) : 0 };
  },
  bind_null({ a, b }) { return bind.call(this, a, b, null); },
  bind_int({ a, b, c }) { return bind.call(this, a, b, c); },
  bind_text({ a, b, text }) { return bind.call(this, a, b, text(0)); },
  bind_blob({ a, b, bytes }) { return bind.call(this, a, b, bytes(0)); },
  bind_zeroblob({ a, b, c }) { return bind.call(this, a, b, new Uint8Array(Math.max(0, c))); },
  reset({ a }) { return stmtDo.call(this, a, (s) => s.reset()); },
  clear({ a }) { return stmtDo.call(this, a, (s) => s.clearBindings()); },
  finalize({ a }) {
    const s = this.get(a, "stmt");
    if (!s) return (this.rc = MISUSE), { int: MISUSE };
    s.value.finalize();
    this.handles.delete(a);
    return (this.rc = OK), { int: OK };
  },
  async step({ a }) {
    const s = this.get(a, "stmt");
    if (!s) return (this.rc = MISUSE), { int: MISUSE };
    const row = this.run(s.db, () => s.value.step());
    const rc = this.rc;
    // a statement that writes: save once it has finished
    if (row === false && !this.sqlite3.capi.sqlite3_stmt_readonly(s.value.pointer)) await this.persist(s.db);
    this.rc = rc;
    return { int: row === null ? rc : row ? ROW : DONE };
  },
  column_int({ a, b }) {
    const v = this.get(a, "stmt")?.value.get(b);
    return { int: typeof v === "number" ? Math.trunc(v) : typeof v === "bigint" ? Number(v) : parseInt(v, 10) || 0 };
  },
  column_text({ a, b }) { return { bytes: toBytes(this.get(a, "stmt")?.value.get(b)) }; },
  column_blob({ a, b }) { return { bytes: toBytes(this.get(a, "stmt")?.value.get(b)) }; },
};

function bind(st, index, value) {
  const s = this.get(st, "stmt");
  if (!s) return (this.rc = MISUSE), { int: MISUSE };
  this.run(s.db, () => s.value.bind(index, value));
  return { int: this.rc };
}

function stmtDo(st, f) {
  const s = this.get(st, "stmt");
  if (!s) return (this.rc = MISUSE), { int: MISUSE };
  this.run(s.db, () => f(s.value));
  return { int: this.rc };
}
