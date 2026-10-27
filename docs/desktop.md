# The desktop app

`reva-desktop` is the Reva REPL in its own window, on Linux and macOS
(Windows builds, but is untested; see [packaging.md](packaging.md#5-windows-10-and-later)). It shows the same page as the browser build and runs the
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
seconds. `desktop/build.sh --bundle` makes installers (`.deb`/`.rpm`,
`.app`/`.dmg`, or NSIS/MSI on Windows) and needs `cargo install tauri-cli`
first; [packaging.md](packaging.md) covers signing and what to ship.

## Run

```text
reva-desktop                          # the Reva REPL; files in ~/Reva
reva-desktop --package DIR            # open the package in DIR
reva-desktop --root DIR               # the REPL with files in DIR
reva-desktop -e ': sq dup * ; 7 sq .' --print-output   # batch: feed lines, copy output to stdout
```

Turtle graphics (`needs turtle/fractals 5 koch-demo`, see
[turtle.md](turtle.md)) open in a second window beside the REPL; closing it
is fine, the REPL's Graphics button or the next drawing brings it back.

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


SQLite databases (`needs db/sqlite`, see [sqlite.md](sqlite.md)) are
files of the same folder, opened by the app's native side: the VM's
calls pause it while the page asks the `host_call` command, which answers
from the `reva-sqlite` crate.

## Packages

A package is a folder with a `reva-package.json`:

```json
{
  "name": "CoSy",
  "start": ["start-cosy.f"]
}
```

`name` titles the window ("CoSy on Reva"); `start` lists files to `include`, in order, after
Reva starts, and again after Forth's `restart` (which starts a fresh system,
as relaunching did; files on disk are kept). `--eval` lines are fed once. Relative paths in Forth resolve against the package folder, so
a package can `include` its own sources. A folder without a manifest opens
as a plain folder.

### Branding

A package can make the window its own, so one installed `reva-desktop`
looks like each package it opens:

```json
{
  "name": "CoSy",
  "title": "CoSy",
  "icon": "cosy.png",
  "id": "cosy",
  "start": ["start-cosy.f"]
}
```

| Key | Effect | Default |
|---|---|---|
| `title` | window title and page header | `<name> on Reva`, or `Reva Forth` |
| `icon` | PNG in the package (256×256 suits every desktop): the window icon, which is also the taskbar icon on Windows and X11, and on macOS the Dock and app-switcher icon while the app runs | Reva's icon |
| `id` | Linux only: the Wayland `app_id` and X11 `WM_CLASS` (letters, digits, `.` `-` `_`). Docks use it to find the package's `.desktop` entry, and with it the dock icon and name | `reva-desktop` |

A bad `icon` or `id` is reported in the window's notes and the app opens with
Reva's. The window is created with these settings, so Reva's title and icon
never flash first.

What run time cannot change, so a package needs its own entry or bundle:

- **Linux app launcher and dock.** A `.desktop` file with `Icon=` and
  `StartupWMClass=<id>`, and the icon under `~/.local/share/icons/hicolor/`.
  With it, GNOME, KDE and Xfce show the package's icon in the dock and task
  switcher (Wayland included), not just on the window. CoSy's
  `install-desktop.sh` is an example.
- **macOS Finder and Launchpad.** The Dock icon is set at run time
  (NSApplication's `applicationIconImage`, once the app has finished
  launching), but Finder, Launchpad and the Dock before launch show the
  `.app` bundle's own `.icns`, and the menu bar shows the executable's name.
  A package that wants those makes a small bundle of its own that runs
  `reva-desktop --package DIR`; CoSy's `make-cosy-app.sh` is an example.
- **Windows Explorer, Start menu and pinned taskbar.** These show the `.exe`
  resource icon or a shortcut's icon. A shortcut to `reva-desktop.exe
  --package DIR` with the package's `.ico` covers it.

### About

The window's **About** button shows the package's `about` entry (see
[web.md](web.md#the-about-dialog)) above reva-fork's version, build,
credits, licenses and links. Links open in the system browser through the
`open_url` command (http and https only).

### Where graphics go: `canvas`

`"canvas": "split"` in the manifest, or `--canvas split` on the command
line (which wins), shows the REPL and the turtle graphics side by side in
the main window, with a draggable divider, as in the browser (see
[web.md](web.md#the-split-view)). `pane` puts them below the REPL; `window`
(the default) opens a second native window. The split and pane are the
browser's iframe pane, so they behave the same in both.

Coming next: a package may bring its own page (panes, keys, drawing) through
the UI service; until then a package runs in the REPL window.

## Design notes

- **One VM build.** The VM runs in the page, in a Web Worker, exactly as in
  the browser; the native side only supplies files and the window. That
  keeps one behaviour to test and one front end to maintain.
- **Own Cargo workspace.** `desktop/` is separate from `rust/`, so the
  portable core keeps zero dependencies and its tests stay fast.
- **Branding at run time, not build time.** Packages are folders, not
  builds: one app binary serves Reva, CoSy and the next package, and a
  package changes its look by editing JSON. The cost is the list above of
  icons only a bundle sets. The icon is decoded with the `png` crate Tauri
  already uses, and the app id is set through `glib`, also already in the
  tree, so branding adds no dependencies.
- **Licenses.** Tauri and its dependencies are MIT/Apache-2.0, except five
  Servo CSS crates under MPL-2.0 (see [licenses.md](licenses.md)). On Linux
  the app uses the system's WebKitGTK and never bundles it.
