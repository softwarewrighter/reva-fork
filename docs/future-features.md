# Future features

Ideas discussed and deliberately **not** scheduled: no saga step does them.
Each says what it is, why it came up, and why it waits, so it can be picked
up later without rediscovering the reasoning. Scheduled work is in
`.agentrail/plan.md` and the saga's steps.

## Host paths in the desktop app: `include /any/host/path`

**What.** In the desktop app, let `include`, `slurp` and the other file
words reach any file on the computer by its host path (for example
`include /Users/me/src/tool.f` or `include ~/notes/today.f`), not only files
inside the app's folder.

**Why it came up.** Step 37 (`local-web-upload`, 2026-11-01) added an Upload
button to the browser page, which is the browser's way around having no
file system. The question was whether the desktop app needs Upload too.
It does not: its files are a real folder. But in the desktop the VM's `/`
*is* that folder (`~/Reva`, or the package's folder with `--package`), so
a file elsewhere cannot be included by its path; it has to be copied into
the folder, or its folder opened with `--package`.

**Why it waits.** The user does not need it now (2026-11-01). Copying a
file in, or `--package DIR`, covers the cases so far.

**If it is picked up.**

- How a host path is told apart from a VM path. Every VM path starts with
  `/` today, so a prefix or a separate word (`include-host`) may be
  clearer than guessing.
- The CLI already reaches host paths (its file system is the real one), so
  the desktop would match the CLI; the browser cannot, and keeps Upload.
- Writes outside the folder deserve a thought: today a desktop program can
  only change files in its own folder, which is a useful safety property
  for packages such as CoSy.
- The `write_file` / `remove_file` commands of the desktop app
  (`components/desktop`) would need the same rule.
