// host.js — what the page gets from where it runs.  This is the browser's:
// files persist in localStorage, `?include=FILE` (repeatable) includes FILE
// after start and again after each `restart`, and `?eval=TEXT` (repeatable)
// feeds a line once.  The desktop app serves its own
// host.js in this file's place (files on disk, packages); index.html is the
// same in both.
//
// hostSetup() resolves to { storage, start, input, title?, output?(text),
// halted?(code), notes }.

export async function hostSetup() {
  let storage = null;
  try {
    storage = globalThis.localStorage ?? null;
    storage?.length; // throws where site data is blocked
  } catch {
    storage = null;
  }
  const q = new URLSearchParams(globalThis.location?.search ?? "");
  const start = q.getAll("include").map((f) => `include ${f}`);
  const input = q.getAll("eval");
  return { storage, start, input, notes: storage ? [] : ["files are not kept: this browser blocks storage"] };
}
