// package.js — load a package from a static directory: `?package=DIR/`.
//
// DIR holds a `reva-package.json` manifest (the same one the desktop app's
// `--package DIR` reads; see docs/packages.md):
//
//   { "name": "CoSy", "title": "CoSy", "icon": "cosy.png", "id": "cosy",
//     "start": ["start-cosy.f"], "files": ["CoSy.f", "demo.f", ...],
//     "canvas": "split" }
//
// A static server cannot list a directory, so the manifest lists the files
// (`files`, plus the `start` files, which are fetched too).  Each lands at
// VM path `/<file>`, as in the desktop app, where the package folder is the
// VM's root.  All URLs are relative to DIR, which is relative to the page,
// so a package works unchanged from any host and under any subpath.
//
// Files the VM writes are kept in localStorage under `reva:<id>:fs:`, so
// two packages served from one origin keep their files apart; a stored file
// wins over the package's copy, so edits survive a reload.

/** At most this many fetches at once (browsers queue more anyway). */
const PARALLEL = 6;

/** A plain relative path: no `..`, no leading `/`, no backslashes or empty parts. */
export function plainPath(p) {
  return typeof p === "string" && p.length > 0 && p.length < 1024 && !p.startsWith("/")
    && !p.includes("\\") && p.split("/").every((s) => s !== "" && s !== "." && s !== "..");
}

/** A storage-key-safe package id: the manifest's `id`, else its `name`. */
export function packageId(m) {
  const id = String(m.id ?? m.name ?? "package").toLowerCase().replace(/[^a-z0-9._-]+/g, "-");
  return id.slice(0, 64) || "package";
}

/**
 * Fetch the package at `dir` (relative to `base`, default the page).
 * Resolves to `{ name, title, icon (URL) | undefined, canvas, start
 * (lines to feed), files ({ "/path": bytes }), prefix, notes }`; rejects
 * only when the manifest itself cannot be read.  A missing file is a note,
 * so one bad entry does not stop the rest.
 */
export async function loadPackage(dir, { base = globalThis.location?.href, fetch = globalThis.fetch } = {}) {
  const root = new URL(dir.endsWith("/") ? dir : `${dir}/`, base);
  const murl = new URL("reva-package.json", root);
  const r = await fetch(murl);
  if (!r.ok) throw new Error(`${murl}: ${r.status} ${r.statusText}`);
  let m;
  try {
    m = await r.json();
  } catch (e) {
    throw new Error(`${murl}: not JSON (${e.message})`);
  }
  const notes = [];
  const list = (key) => {
    const v = m[key] ?? [];
    if (!Array.isArray(v)) { notes.push(`manifest: "${key}" is not a list; ignored`); return []; }
    return v.filter((p) => plainPath(p) || (notes.push(`manifest: ${JSON.stringify(p)} is not a plain relative path; skipped`), false));
  };
  const start = list("start");
  const wanted = [...new Set([...list("files"), ...start])];
  const files = {};
  let next = 0;
  const worker = async () => {
    while (next < wanted.length) {
      const rel = wanted[next++];
      try {
        const res = await fetch(new URL(rel, root));
        if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
        files[`/${rel}`] = new Uint8Array(await res.arrayBuffer());
      } catch (e) {
        notes.push(`package file ${rel}: ${e.message ?? e}`);
      }
    }
  };
  await Promise.all(Array.from({ length: Math.min(PARALLEL, wanted.length) }, worker));
  const name = typeof m.name === "string" ? m.name : undefined;
  return {
    name,
    title: typeof m.title === "string" ? m.title : name && `${name} on Reva`,
    icon: plainPath(m.icon) ? new URL(m.icon, root).href : undefined,
    canvas: typeof m.canvas === "string" ? m.canvas : undefined,
    start: start.map((f) => `include ${f}`),
    files,
    prefix: `reva:${packageId(m)}:fs:`,
    notes,
  };
}
