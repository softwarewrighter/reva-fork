# The browser REPL as a static site

Reva runs in the browser as a WebAssembly VM in a Web Worker
(`rust/reva-web/www`). This page covers what a page can be told through its
URL, the split view, loading a package from a static directory, and
building the zip you upload to a web host.

```text
just web-zip                      # or: rust/scripts/make-web-zip.sh
#   → rust/target/web/reva-web-VERSION.zip   (and the same tree, unzipped, beside it)
# upload, unzip anywhere on the host, open https://your.host/any/path/reva-web-VERSION/
```

## URL options

| Option | Effect |
|---|---|
| `?canvas=split` | REPL on the left, turtle graphics on the right (below) |
| `?canvas=pane` | graphics in a pane below the REPL |
| `?canvas=window` | graphics in a popup window (the default; a pane where popups are blocked) |
| `?package=DIR/` | load the package in `DIR` (relative to the page) |
| `?include=FILE` | include FILE after start and after each `restart` (repeatable) |
| `?eval=TEXT` | feed TEXT once after start (repeatable) |
| `?image=FILE` | start from a saved image instead of booting `reva.f` |

When the URL does not say, the page falls back to the package's `canvas`,
then to its own default: the `reva-canvas` meta tag in `index.html`, which
the web zip sets to `split`. Likewise the `reva-package` meta tag names the
package to open when there is no `?package=`.

## The split view

`?canvas=split` (or `"canvas": "split"` in a package manifest) shows the
REPL and the graphics pane side by side, both full height, 50/50 to start.

- **Drag the divider** to resize; the REPL's share stays within 20–80%.
- **Keyboard:** Tab to the divider, then ←/→ move it 2% (Shift: 10%),
  Home/End go to the limits, Enter collapses or restores the pane.
- **Double-click** the divider to go back to 50/50.
- **Collapse:** the ⟩ button folds the graphics pane to a strip at the right
  edge; the strip's button brings it back.
- **Remembered:** position and collapsed state are kept in localStorage
  (`reva:ui:split`) for the site's origin.
- **Redraws on resize:** the drawing is redrawn at the pane's new
  resolution once a resize settles (120 ms).
- Under 640 px wide the panes stack, REPL above graphics.

The desktop app shows the same split with `--canvas split` or a package's
`"canvas": "split"` (see [desktop.md](desktop.md)).

## Packages from a static directory: `?package=DIR/`

`DIR/reva-package.json` is the manifest the [desktop app](desktop.md) reads
with `--package DIR`, plus a `files` list, since a static server cannot list
a folder:

```json
{
  "name": "CoSy",
  "title": "CoSy",
  "icon": "cosy.png",
  "id": "cosy",
  "canvas": "split",
  "start": ["start-cosy.f"],
  "files": ["CoSy.f", "util.f", "start-cosy.f"]
}
```

| Key | In the browser |
|---|---|
| `files` | fetched (six at a time) into the VM at `/<file>`, as the desktop app maps the package folder to `/`; a file that fails is a note, not a stop |
| `start` | fetched too, and included after start and after each `restart` |
| `title` | page title and header (default `<name> on Reva`) |
| `icon` | the page's icon (favicon) |
| `id` | names the localStorage prefix, `reva:<id>:fs:` (default from `name`), so two packages on one site keep their files apart |
| `canvas` | where graphics go, unless the URL says |

Paths must be plain relative paths (no `..`, no leading `/`); others are
skipped with a note. Files the VM writes are stored in localStorage and win
over the package's copy on the next load, so edits survive a reload. The
desktop app ignores `files` (it reads the folder itself).

## The uploadable zip: `just web-zip`

`rust/scripts/make-web-zip.sh` (POSIX sh; `just web-zip` runs it) builds
`reva.wasm` and writes `rust/target/web/NAME-web-VERSION.zip`, holding one
folder `NAME-web-VERSION/`:

```text
index.html  reva.js  reva-worker.js  reva.wasm  host.js  ui.js  canvas.html
split.js  package.js  README.txt  LICENSE.txt  LICENSE-MIT.txt  [pkg/ ...]
```

| Option | Effect |
|---|---|
| `--package DIR` | copy DIR into `pkg/` and open it by default; NAME defaults to its `id` |
| `--canvas MODE` | the default graphics placement (default `split`) |
| `--name NAME`, `--version V` | the zip's name (default `reva`, the workspace version) |
| `--out DIR` | where the zip goes (default `rust/target/web`) |
| `--no-build` | use the `reva.wasm` already built |

With `just`: `just web-zip --package path/to/pkg`. CoSy's zip is built in
its own repository (`just cosy-zip` in ba-cosy-zip-fork), which calls this
script.

To check a zip the way a web host serves it, unzip it into a subdirectory
of any static server:

```text
mkdir -p /tmp/site/a/b && cd /tmp/site/a/b && unzip ~/reva-web-VERSION.zip
python3 -m http.server -d /tmp/site 8000
#   open http://localhost:8000/a/b/reva-web-VERSION/
```

`file://` does not work: browsers do not start module workers or fetch
files from `file://` pages. To run the same tree with no server at all,
use [reva-serve](serve.md), which embeds it in one binary.

## Design notes

* **Static and relative, on purpose.** Every URL the page uses is relative to
  `index.html` (`reva.wasm`, `pkg/...`), there is no server logic, and no
  special headers: no SharedArrayBuffer, so no COOP/COEP, which ISP hosts
  rarely let you set. So the same tree works from an ISP's shared hosting,
  GitHub Pages, `python3 -m http.server`, or embedded in `reva-serve`.
* **The manifest is the one contract.** The desktop app, the browser and
  the zip read the same `reva-package.json`; `files` is the only key the
  browser needs that the desktop does not.
* **Defaults in meta tags, not code.** The zip is `www/` plus a package,
  with two `<meta>` values set (`reva-package`, `reva-canvas`). No build
  step rewrites JavaScript, and a URL option still overrides either.
* **Files are per origin.** localStorage belongs to scheme + host + port. A
  site moved to another host starts with no saved files, and the split
  position is remembered per origin too. Storage is optional: where a
  browser blocks it the page says so and works without it.
* **Reproducible zip.** Names are sorted and every entry carries the last
  commit's time, so one commit gives one zip (same SHA-256), which makes a
  zip on a web host easy to match with its source.
* **Resize redraw without a copy.** The graphics page keeps no copy of the
  drawing: on resize it sizes its backing store to the pane and asks the
  VM to send the drawing again (the same `resend` used when a window
  opens), so memory stays bounded and the lines stay sharp at any size.
* **Typing before boot.** Lines entered while a package is still loading
  are queued and fed after its start files.
