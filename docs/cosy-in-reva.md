# CoSy in reva-fork

[CoSy](https://cosy.com/CoSy/) is Bob Armstrong's APL-inspired language
built on Reva Forth, and reva-fork's main user. This page lists the ways to
run it on reva-fork, and which of them work today and which are planned.

**This repository holds no CoSy code, and never will.** Not its sources, not
a package, not test fixtures or goldens, not the live demo. Whether and
where CoSy is published is up to Bob. The CoSy package is built in the
private repository
[ba-cosy-zip-fork](https://github.com/softwarewrighter/ba-cosy-zip-fork)
(the "CoSy lane"), which patches Bob's sources and adds launchers. Its
`reva-fork-subset/README.md` and `DEMO.md` are the detailed instructions.
Everything below runs a package you build there, on your own machine.

Status checked on 2026-11-01 against reva-fork `dev` (2026.1101.0, after
steps 33 to 35), from a scratch copy of ba-cosy-zip-fork's `dev`.

## The package

```text
cd ba-cosy-zip-fork
reva-fork-subset/make-subset.sh      # → reva-fork-subset/build/, a reva-fork package
```

`build/` is a [package](web.md): a folder with `reva-package.json`
(`start`: `start-cosy.f`). Every host below runs it. When CoSy starts it
loads its sources, restores `CoSy.csy` into its dictionary `R` and runs the
script stored there.

## At a glance

| Way | Status | How |
|---|---|---|
| CLI REPL | **works** | `cd build && reva start-cosy.f` (or `build/run-cosy`) |
| CLI, piped or scripted | **works** | `reva start-cosy.f -e '...'`, or input on stdin |
| CLI drawings as files | **works** | `reva --draw png start-cosy.f`, then `include cosy-fractals.f` and `12 cdragon` |
| reva-fork's notebook, in the terminal | **works** | `needs ui/notebook notebook` at CoSy's `ok>` |
| Browser, local (`cosy-serve`) | **works** | `scripts/build-serve.sh --package build --name cosy` (see [below](#browser-and-desktop)) |
| reva-fork's notebook, in the browser | **works** | `needs ui/notebook notebook` at CoSy's `ok>` |
| Browser, on a web host (zip) | works, not published | `scripts/make-web-zip.sh --package build --name cosy`; publish only where Bob agrees |
| Desktop app | works (ba-cosy) | `cosy-desktop` or `CoSy.app` from ba-cosy-zip-fork; not re-run on 2026-11-01 |
| ba-cosy's `just cosy-zip`, `just cosy-serve` | **broken** | they call reva-fork's old paths; workaround below |
| Bob's `Tui.f` windows (IUP) | planned | step 40 `local-tui-cosy` |
| Upload a file or package into the page | planned | step 37 `local-web-upload` |
| One saved image (`--image`) | **not yet** | step 38 `local-image-everywhere` |
| A published site (e.g. cosy.softwarewrighter.com) | planned, Bob's call | step 39 `local-publish-package` |

## The CLI

```text
cd ba-cosy-zip-fork/reva-fork-subset/build
reva start-cosy.f                    # the REPL; `bye` quits, `restart` starts over
reva start-cosy.f -e 'demo paste-tidy.txt' -e bye     # a scripted run
```

Checked: CoSy boots in about half a second, and its demo script
(`demo paste-tidy.txt`: vectors, reduce, reshape, strings, the dictionary)
prints the expected result for every line.

**Drawings.** The CLI has no graphics window, so each turtle drawing goes to
a new file ([turtle.md](turtle.md#the-cli-drawings-as-files)). With
`--draw png`, CoSy's `include cosy-fractals.f` and `12 cdragon` write
`turtle-001.png` (4,096 segments), never over an existing file. In iTerm2
or WezTerm the default `--draw auto` also shows the PNG inline.

**The notebook.** `needs ui/notebook notebook` opens reva-fork's TUI
notebook full screen in the terminal, over CoSy: a CoSy line evaluated with
Ctrl-J shows its result in the lower pane. Checked in tmux. The notebook
keeps its text in `notebook.txt` in the package folder.

## Browser and desktop

The browser page, `reva-serve` and the desktop app run the same web page,
with CoSy's REPL on the left and the turtle graphics on the right
(`"canvas": "split"` in the manifest).

**cosy-serve: one local binary.** reva-fork's
[reva-serve](serve.md) built with the package:

```text
reva-fork/scripts/build-serve.sh --package ba-cosy-zip-fork/reva-fork-subset/build \
    --name cosy --port 8481 --out /some/private/folder
/some/private/folder/cosy-serve-VERSION-macos-universal     # opens http://127.0.0.1:8481/
```

Checked: CoSy boots in the page; `?eval=include%20cosy-fractals.f&eval=12%20cdragon`
draws the dragon beside the REPL; `needs ui/notebook notebook` opens the
notebook, and a CoSy line evaluated there shows its result. Pass `--out`:
by default the scripts write into reva-fork's `target/`, and CoSy builds
should not sit in this checkout.

**A web host: the zip.** `scripts/make-web-zip.sh --package ... --name cosy
--out ...` makes a static site to unzip anywhere under a web root
([web.md](web.md)). Files CoSy writes stay in the visitor's browser
storage. The binary and the zip both contain Bob's sources: share them only
where he agrees.

**The desktop app.** ba-cosy-zip-fork's `cosy-desktop` runs
`reva-desktop --package build` in a window with CoSy's title and icon;
`make-cosy-app.sh` makes a self-contained macOS `CoSy.app`. Both are the
CoSy lane's and were not re-run here; the desktop app shows the same page
as the browser, which was.

**Broken today: ba-cosy's wrappers.** `just cosy-zip` and `just cosy-serve`
in ba-cosy-zip-fork call `$REVA_FORK/rust/scripts/make-web-zip.sh` and
`rust/scripts/build-serve.sh`, and expect output in `rust/target/`. Since
reva-fork's components restructuring these are `scripts/` and `target/` at
the root, with the same options. Until the CoSy lane updates its paths, run
reva-fork's scripts directly as above.

## Planned

- **Bob's windows, `Tui.f` and `Job.f`** (step 40 `local-tui-cosy`): the
  IUP dialogs CoSy's TUI is made of, through reva-fork's virtual IUP
  library ([iup-plan.md](iup-plan.md), [ui-model.md](ui-model.md)), in the
  terminal, the browser and the desktop app. ba-cosy's `tui-text.f` covers
  the text side today.
- **Upload into the page** (step 37 `local-web-upload`): an Upload button
  for a Forth file or a whole package, kept in browser storage (IndexedDB or
  OPFS for large files), with storage pluggable so that, for example, a
  CoSy host can keep notebooks on its own server.
- **One file to ship** (step 38 `local-image-everywhere`): a saved image
  that starts in the CLI, the page and the desktop app. Not yet for CoSy:
  `--save-image` after loading CoSy writes an image, but `--image` with it
  shows CoSy's depth display and then neither results nor a prompt.
- **A published site** (step 39 `local-publish-package`): a package or
  image behind its own domain, such as cosy.softwarewrighter.com, only if
  Bob wants CoSy published that way. Otherwise reva-fork is hosted alone,
  and CoSy is hosted by Bob.

What CoSy needs from reva-fork, by priority, is tracked in
ba-cosy-zip-fork's `docs/reva-fork-asks.md`.
