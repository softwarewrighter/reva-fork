// Node test for www/store.js and www/upload.js, with a small in-memory
// IndexedDB.  Run: node components/web/crates/reva-web/test/store.mjs
import { openIdbStorage, size } from "../www/store.js";
import { uploadFiles } from "../www/upload.js";
import { storedFiles, saveChanges } from "../www/reva.js";

let failed = 0;
const check = (what, got, want) => {
  const ok = JSON.stringify(got) === JSON.stringify(want);
  if (!ok) failed++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}: ${JSON.stringify(got)}`);
};

// ---- a fake IndexedDB: one database, one store, async callbacks ----
const later = (f) => setTimeout(f, 0);
function fakeIndexedDB() {
  const dbs = new Map();
  const request = (run) => {
    const r = {};
    later(() => { r.result = run(); r.onsuccess?.(); });
    return r;
  };
  const versions = new Map();
  return {
    open(name, version) {
      const r = {};
      later(() => {
        const fresh = !dbs.has(name);
        if (fresh) dbs.set(name, new Map());
        const upgrade = fresh || (version ?? 0) > versions.get(name);
        versions.set(name, Math.max(versions.get(name) ?? 1, version ?? 1));
        const stores = dbs.get(name);
        r.result = {
          version: versions.get(name),
          objectStoreNames: { contains: (s) => stores.has(s) },
          close() {},
          createObjectStore: (s) => stores.set(s, new Map()),
          transaction(s) {
            const data = stores.get(s);
            const tx = { oncomplete: null, objectStore: () => ({
              put: (v, k) => request(() => data.set(k, v)),
              delete: (k) => request(() => data.delete(k)),
              getAll: () => request(() => [...data.values()]),
              getAllKeys: () => request(() => [...data.keys()]),
            }) };
            later(() => later(() => tx.oncomplete?.()));
            return tx;
          },
        };
        if (upgrade) r.onupgradeneeded?.();
        r.onsuccess?.();
      });
      return r;
    },
  };
}
function fakeLocalStorage(entries) {
  const m = new Map(Object.entries(entries));
  return {
    get length() { return m.size; },
    key: (i) => [...m.keys()][i] ?? null,
    getItem: (k) => m.get(k) ?? null,
    setItem: (k, v) => m.set(k, String(v)),
    removeItem: (k) => m.delete(k),
    keys: () => [...m.keys()],
  };
}

const idb = fakeIndexedDB();
const legacy = fakeLocalStorage({
  "reva:fs:/notebook.txt": "2 3 +",
  "reva:cosy:fs:/x.csy": "\x00\xff\x01",
  "reva:history": '["1 ."]',
});
const estimate = async () => ({ usage: 2048, quota: 10 * 1024 ** 3 });
const { storage, moved } = await openIdbStorage({ indexedDB: idb, legacy, estimate });
check("files moved from localStorage", moved, 2);
check("localStorage keeps only non-file keys", legacy.keys(), ["reva:history"]);
check("a moved file, as bytes", [...storage.getBytes("reva:cosy:fs:/x.csy")], [0, 255, 1]);
check("reva.js reads bytes", Object.keys(storedFiles(storage)), ["/notebook.txt"]);

saveChanges(storage, [{ path: "/a.f", data: new TextEncoder().encode(": a 1 ;") }, { path: "/notebook.txt", data: null }]);
await storage.flush();
const again = await openIdbStorage({ indexedDB: idb, legacy, estimate });
check("written through: reopened", Object.keys(storedFiles(again.storage)), ["/a.f"]);
check("nothing moved twice", again.moved, 0);
check("usage", await again.storage.usage(), { used: 2048, quota: 10737418240 });

const puts = [];
const session = { put: (path, data) => puts.push([path, data.length]) };
const file = (name, text) => ({ name, bytes: async () => new TextEncoder().encode(text).buffer });
const lines = await uploadFiles([file("pets.f", ": cat ;"), file("R.csy", "xyz"), file("my notes.f", "")], session, estimate().then((e) => ({ used: e.usage, quota: e.quota })));
check("uploads go to /NAME", puts, [["/pets.f", 7], ["/R.csy", 3], ["/my notes.f", 0]]);
check("upload lines", lines, [
  "uploaded pets.f (7 B): include pets.f loads it",
  "uploaded R.csy (3 B)",
  "uploaded my notes.f (0 B) (rename it without spaces to include it)",
  "storage: 2.0 KB used of 10 GB for this site",
]);
const replaced = await uploadFiles([file("pets.f", ": dog ;")], session, null, (p) => p === "/pets.f");
check("same name: replaced", replaced, ["uploaded pets.f (7 B, replaced the old copy): include pets.f loads it"]);
check("sizes", [size(512), size(1536), size(3.2 * 1024 ** 2), size(10 * 1024 ** 3)], ["512 B", "1.5 KB", "3.2 MB", "10 GB"]);

// A database made elsewhere without the store: opened one version up.
const bare = fakeIndexedDB();
await new Promise((ok) => { const o = bare.open("reva-files"); o.onsuccess = ok; });
const fixed = await openIdbStorage({ indexedDB: bare });
fixed.storage.setBytes("reva:fs:/b", new Uint8Array([7]));
await fixed.storage.flush();
check("a database without the store gets one", [...(await openIdbStorage({ indexedDB: bare })).storage.getBytes("reva:fs:/b")], [7]);

check("no IndexedDB rejects", await openIdbStorage({ indexedDB: null }).then(() => "opened", (e) => e.message), "no IndexedDB");
process.exit(failed ? 1 : 0);
