# The desktop app

`reva-desktop` is the Reva REPL in its own window, on Linux and macOS
(Windows later). It shows the same page as the browser build and runs the
same WebAssembly VM, so a program behaves the same in both. What it adds:
files live in a folder on disk instead of browser storage, and it can open a
*package*, a folder with its own start files (CoSy is one).

## Build and install

From a checkout:

```text
desktop/build.sh              # builds desktop/target/release/reva-desktop
desktop/build.sh --install    # ... and installs reva-desktop into ~/.cargo/bin
desktop/smoke.sh              # checks it: evaluate, files, a package
```

Needs Rust (with the `wasm32-unknown-unknown` target, which the build adds)
and:

- **Linux:** WebKitGTK 4.1 headers. Debian/Ubuntu:
  `sudo apt install libwebkit2gtk-4.1-dev build-essential`. Fedora:
  `sudo dnf install webkit2gtk4.1-devel`.
- **macOS:** Xcode's command line tools (`xcode-select --install`).

The first build compiles Tauri and takes a few minutes; later builds take
seconds. `desktop/build.sh --bundle` makes installers (`.deb`/`.rpm`, or
`.app`/`.dmg`) and needs `cargo install tauri-cli` first.

## Run

```text
reva-desktop                          # the Reva REPL; files in ~/Reva
reva-desktop --package DIR            # open the package in DIR
reva-desktop --root DIR               # the REPL with files in DIR
reva-desktop -e ': sq dup * ; 7 sq .' --print-output   # batch: feed lines, copy output to stdout
```

`bye` closes the window. `restart` starts a fresh system in the same window
and includes the package's start files again. Stop (or Ctrl-C) interrupts a running word, as in
the browser.

## Files

The VM's file system is the folder: VM path `/notes.txt` is
`~/Reva/notes.txt` (or `DIR/notes.txt` for a package). Every file in the
folder is loaded at start, up to 16 MB per file and 256 MB in all (larger
ones are skipped with a note), and each file Forth writes or deletes is
written back at once. Hidden files and folders (`.git`) are not loaded. The
built-in library stays at `/reva/lib/` and is never written to disk. Paths
cannot leave the folder.

## Packages

A package is a folder with a `reva-package.json`:

```json
{
  "name": "CoSy",
  "start": ["start-cosy.f"]
}
```

`name` titles the window; `start` lists files to `include`, in order, after
Reva starts, and again after Forth's `restart` (which starts a fresh system,
as relaunching did; files on disk are kept). `--eval` lines are fed once. Relative paths in Forth resolve against the package folder, so
a package can `include` its own sources. A folder without a manifest opens
as a plain folder.

Coming next: a package may bring its own page (panes, keys, drawing) through
the UI service; until then a package runs in the REPL window.

## Design notes

- **One VM build.** The VM runs in the page, in a Web Worker, exactly as in
  the browser; the native side only supplies files and the window. That
  keeps one behaviour to test and one front end to maintain.
- **Own Cargo workspace.** `desktop/` is separate from `rust/`, so the
  portable core keeps zero dependencies and its tests stay fast.
- **Licenses.** Tauri and its dependencies are MIT/Apache-2.0, except five
  Servo CSS crates under MPL-2.0 (see [licenses.md](licenses.md)). On Linux
  the app uses the system's WebKitGTK and never bundles it.
