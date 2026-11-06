// host.js — what the page gets from where it runs.  This is the browser's:
// files persist in IndexedDB (store.js; localStorage where IndexedDB cannot
// open), `?include=FILE` (repeatable) includes FILE after start and again
// after each `restart`, `?diagnose=NAME` (repeatable) runs `diagnose NAME`
// before those, and `?eval=TEXT` (repeatable) feeds a line once.
// The desktop app serves its own host.js in this file's place (files on
// disk, packages); index.html is the same in both.
//
// hostSetup() resolves to { storage, start, input, title?, output?(text),
// halted?(code), notes }.

import { openIdbStorage } from "./store.js";

/** localStorage, or null where the browser blocks site data. */
function webStorage() {
  try {
    const s = globalThis.localStorage ?? null;
    s?.length; // throws where site data is blocked
    return s;
  } catch {
    return null;
  }
}

export async function hostSetup() {
  const legacy = webStorage();
  const notes = [];
  let storage = legacy;
  try {
    const opened = await openIdbStorage({ legacy });
    storage = opened.storage;
    if (opened.moved) notes.push(`${opened.moved} saved file(s) moved from localStorage to IndexedDB`);
  } catch (e) {
    console.warn("reva: IndexedDB not available, files go to localStorage:", e);
  }
  if (!storage) notes.push("files are not kept: this browser blocks storage");
  const q = new URLSearchParams(globalThis.location?.search ?? "");
  // `?diagnose=NAME` (repeatable) turns on a check first, so includes run with it
  const checks = q.getAll("diagnose").map((d) => `diagnose ${d}`);
  const start = checks.concat(q.getAll("include").map((f) => `include ${f}`));
  const input = q.getAll("eval");
  return { storage, start, input, notes };
}
