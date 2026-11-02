// store.js — where the page keeps files: IndexedDB, behind the Web Storage
// interface the page already uses (length, key, getItem, setItem,
// removeItem), plus getBytes/setBytes so files are kept as bytes.
//
// IndexedDB is asynchronous and the page reads files synchronously (at
// start and on a restart), so the store keeps a copy in memory: opened once,
// read from memory, written through to the database in order.  The desktop
// app's host.js gives the page another object with the same interface (a
// folder on disk); any host can (a server, for a CoSy host keeping notebooks
// on its own machine).
//
// Opening moves files saved by older pages from localStorage ("reva:fs:"
// and "reva:<package>:fs:" keys) into the database and frees them there:
// localStorage is about 5 MB for the whole site, shared by every
// softwarewrighter.github.io page, one character per byte.

const DB = "reva-files", STORE = "files";
const isFileKey = (k) => /^reva:([^:]+:)?fs:/.test(k);

// Strings and bytes, one char per byte, as reva.js stores in Web Storage.
const toLatin1 = (b) => {
  let s = "";
  for (let i = 0; i < b.length; i += 0x8000) s += String.fromCharCode.apply(null, b.subarray(i, i + 0x8000));
  return s;
};
const fromLatin1 = (s) => Uint8Array.from(s, (c) => c.charCodeAt(0));

/** A request or transaction as a promise. */
const done = (r) => new Promise((ok, fail) => {
  if ("oncomplete" in r) { r.oncomplete = () => ok(); r.onerror = r.onabort = () => fail(r.error); }
  else { r.onsuccess = () => ok(r.result); r.onerror = () => fail(r.error); }
});

/**
 * The database with its store.  A database that exists without the store
 * (made by something else at the same version) is opened again one version
 * up, which creates it.
 */
async function openDb(indexedDB, version) {
  const open = version ? indexedDB.open(DB, version) : indexedDB.open(DB);
  open.onupgradeneeded = () => {
    if (!open.result.objectStoreNames.contains(STORE)) open.result.createObjectStore(STORE);
  };
  const db = await done(open);
  if (db.objectStoreNames.contains(STORE)) return db;
  const next = db.version + 1;
  db.close();
  return openDb(indexedDB, next);
}

/** Every record of the store, as a Map of key to bytes. */
async function readAll(db) {
  const tx = db.transaction(STORE, "readonly");
  const st = tx.objectStore(STORE);
  const [keys, values] = await Promise.all([done(st.getAllKeys()), done(st.getAll())]);
  return new Map(keys.map((k, i) => [String(k), new Uint8Array(values[i])]));
}

/** Copy `legacy`'s file keys into the database; then remove them there. */
async function migrate(db, map, legacy) {
  const keys = [];
  for (let i = 0; i < (legacy?.length ?? 0); i++) {
    const k = legacy.key(i);
    if (k && isFileKey(k) && !map.has(k)) keys.push(k);
  }
  if (!keys.length) return 0;
  const tx = db.transaction(STORE, "readwrite");
  for (const k of keys) {
    const b = fromLatin1(legacy.getItem(k) ?? "");
    map.set(k, b);
    tx.objectStore(STORE).put(b, k);
  }
  await done(tx);
  for (const k of keys) legacy.removeItem(k);
  return keys.length;
}

/**
 * Open the file store: `{ storage, moved }`, `storage` the Web Storage
 * look-alike (plus getBytes, setBytes, usage(), flush()), `moved` how many
 * files came over from `legacy` (localStorage).  Rejects where IndexedDB
 * cannot open (then the page keeps using localStorage).
 */
export async function openIdbStorage({ indexedDB = globalThis.indexedDB, legacy = null, estimate } = {}) {
  if (!indexedDB) throw new Error("no IndexedDB");
  const db = await openDb(indexedDB);
  const map = await readAll(db);
  const moved = await migrate(db, map, legacy);
  return { storage: idbStorage(db, map, estimate), moved };
}

/** The Web Storage look-alike over `map`, writing through to `db`. */
function idbStorage(db, map, estimate = () => globalThis.navigator?.storage?.estimate?.()) {
  let writes = Promise.resolve();
  const write = (f) => {
    writes = writes.then(() => {
      const tx = db.transaction(STORE, "readwrite");
      f(tx.objectStore(STORE));
      return done(tx);
    }).catch((e) => console.warn("reva: could not save a file:", e));
  };
  return {
    get length() { return map.size; },
    key: (i) => [...map.keys()][i] ?? null,
    getItem: (k) => (map.has(k) ? toLatin1(map.get(k)) : null),
    setItem(k, v) { this.setBytes(k, fromLatin1(String(v))); },
    removeItem(k) { map.delete(k); write((st) => st.delete(k)); },
    getBytes: (k) => map.get(k) ?? null,
    setBytes(k, b) { const c = new Uint8Array(b); map.set(k, c); write((st) => st.put(c, k)); },
    /** Resolves when every write so far has reached the database. */
    flush: () => writes,
    /** `{ used, quota }` in bytes (the browser's estimate for the site), or null. */
    async usage() {
      const e = await estimate?.();
      return e ? { used: e.usage ?? 0, quota: e.quota ?? 0 } : null;
    },
  };
}

/** A byte count for people: 512 B, 1.5 KB, 3.2 MB, 10 GB. */
export function size(n) {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  while (n >= 1024 && i < units.length - 1) { n /= 1024; i++; }
  return `${i ? n.toFixed(n < 10 ? 1 : 0) : n} ${units[i]}`;
}
