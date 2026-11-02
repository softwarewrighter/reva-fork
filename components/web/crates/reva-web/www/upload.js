// upload.js — the Upload button: files chosen on this computer go into
// the VM's file system (as /NAME) and into the page's storage, so they are
// there after a reload too.  Then `include NAME` loads a Forth file, as on
// the CLI; other files (data, a CoSy .csy) are read the same way.  An
// upload of the same name replaces the file; Forth's `delete` removes it
// (from storage too, as for any file the VM deletes).

import { size } from "./store.js";

const FORTH = /\.(f|fth|fs|4th|forth)$/i;

/**
 * Put `files` ({ name, bytes() }) into `session` (start()'s, which saves
 * them to storage too) as /NAME, the browser's name for the file (it gives
 * no folder); a file of that name is replaced.  `had(path)`: whether one
 * was stored.  The lines to show, `usage` ({ used, quota } or null) last.
 */
export async function uploadFiles(files, session, usage = null, had = () => false) {
  const lines = [];
  for (const f of files) {
    const data = new Uint8Array(await f.bytes());
    const path = `/${f.name}`;
    const old = had(path) ? ", replaced the old copy" : "";
    session.put(path, data);
    const how = / /.test(f.name) ? " (rename it without spaces to include it)"
      : FORTH.test(f.name) ? `: include ${f.name} loads it` : "";
    lines.push(`uploaded ${f.name} (${size(data.length)}${old})${how}`);
  }
  const u = await usage;
  if (u) lines.push(`storage: ${size(u.used)} used of ${size(u.quota)} for this site`);
  return lines;
}

/** Wire `button` to a file chooser that uploads into `session`. */
export function uploadButton(button, { session, storage, prefix = "reva:fs:", echo }) {
  const input = document.createElement("input");
  input.type = "file";
  input.multiple = true;
  input.hidden = true;
  button.after(input);
  button.hidden = false;
  button.addEventListener("click", () => input.click());
  input.addEventListener("change", async () => {
    const chosen = [...input.files].map((f) => ({ name: f.name, bytes: () => f.arrayBuffer() }));
    input.value = "";
    const had = (path) => storage?.getItem(prefix + path) != null;
    const lines = await uploadFiles(chosen, session, storage?.usage?.(), had);
    echo(lines.map((l) => l + "\n").join(""));
  });
}
