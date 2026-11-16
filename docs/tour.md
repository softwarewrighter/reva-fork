# A tour of reva-fork

Where things are and how to get around: starting and leaving each front
end, finding words and libraries, reading a library's source, files and
listing them (the browser's files too), the notebook, and the graphics.
[getting-started.md](getting-started.md) has the details for building and
the `reva` command; [docs/README.md](README.md) indexes every page.

## The four front ends

The same Reva runs in each; what differs is where the files are and what
can be drawn.

| Front end | Start it | Leave it | Files | Draws |
|---|---|---|---|---|
| the `reva` command (CLI) | `just cli` (or `target/release/reva`) | `bye` or Ctrl-D | the real files, from the current folder | turtle drawings as SVG/PNG files; IUP dialogs as text (TUI) at a terminal |
| the browser | the [live demo](https://softwarewrighter.github.io/reva-fork/), or `just web-dev` (<http://localhost:8000/>) | close the tab; `bye` halts the VM, reload to start again | kept in the browser (IndexedDB) | turtle, IUP dialogs and CD canvases |
| the desktop app | `just desktop` | `bye` or close the window | `~/Reva` (or `--root DIR`, `--package DIR`) | as the browser, the turtle in its own window |
| `reva-serve` | `just serve` (opens the browser) | Ctrl-C in its terminal | as the browser | as the browser |

In the browser, the desktop app and `reva-serve`, **Stop** or Ctrl-C
interrupts a running word and keeps your definitions; in the CLI, Ctrl-C
at the prompt drops the line, and while a word runs it ends `reva`, as
Reva always did ([repl.md](repl.md)). `restart` starts a fresh Reva in
place, keeping the files. `version` says which build is running, and
the page's footer says the same.

## Finding your way in the REPL

| Type | Shows |
|---|---|
| `words` | the words of the context on top of the search order |
| `words dup` | every word whose name contains `dup` |
| `words~ ~turtle` | the words of one context |
| `.contexts` | the contexts in the search order |
| `help WORD` | a word's stack picture, context, source and description |
| `see WORD` | the bytes of a word; after `needs debugger`, its x86 code ([x86-plan.md](x86-plan.md#seeing-the-x86-see)) |
| `.s` | the data stack, without changing it |
| `.needs` | the libraries loaded so far |

```forth
help dup
```
```output
dup  ( a -- a a )
  context: ~   source: src/reva.f
  Duplicate the top stack item
```

## Libraries: find, load, read

The libraries in `lib/` are built into every front end, at `/reva/lib/`:
`needs turtle/fractals` loads `lib/turtle/fractals` (and what it needs), once.
Most put their words in a context of their own; `with~ ~turtle` (or
`words~ ~turtle`) shows them. To see what there is, list the folders
(next section) or browse [`lib/`](../lib/) on GitHub:

| Folder | What |
|---|---|
| `turtle/` | turtle graphics and fractals ([turtle.md](turtle.md)) |
| `ui/` | IUP dialogs (`ui/gui`), CD drawing (`ui/cd`), the notebook, demos ([iup.md](iup.md)) |
| `highlight/` | colour listings: `pp` ([highlight.md](highlight.md)) |
| `db/` | SQLite ([sqlite.md](sqlite.md)) |
| `math/`, `string/`, `date/`, `random/`, `data/`, `alg/`, `crypt/`, `os/` | Reva's general libraries |
| `compat/` | Reva 7.0's behaviour for older programs ([compat-reva70.md](compat-reva70.md)) |

**Reading a library's source** is `pp`, built in (its first use loads
`highlight/core`): it lists a file in colour, and a name that is not a
file is looked up under `lib/`:

```text
pp ui/cd-demo          | the CD demo's source
pp turtle/fractals     | the fractals
pp notes.f             | a file of your own
```

Each library starts with a comment saying how to use it, so the first
lines of `pp` are usually the documentation.

## Files

Paths are `/`-separated everywhere. A relative name is in the current
folder: the CLI's own, the desktop app's `~/Reva`, the browser's `/`.

| Word | Does |
|---|---|
| `ls`, `ls DIR` | list the current folder, or DIR (built in) |
| `pp NAME` | show a file, in colour, without loading it (built in) |
| `" NAME" slurp type` | show a file, plain (`slurp` gives its text) |
| `include NAME` | load a Forth file |
| `" NAME" delete` | delete it; `" OLD" " NEW" rename` renames |
| `needs os/dir` | `mkdir`, `rmdir`, `isdir?`, and `dir`, which calls a word per file |

**Listing a folder** is `ls`: the names sorted, folders ending in `/`,
hidden names (`.git`) left out. The folder is the rest of the line, so
`ls` alone is the current folder:

```forth
ls /reva/lib/highlight
```
```output
core forth.tbl
```

In the browser `ls` lists what you have saved, for example
`notebook.txt reva/`; `/reva` is the built-in library, which is
read-only (`ls /reva/lib`). The CLI and the desktop app list the folder
on disk. A program uses `" DIR" (ls)`.

**Showing a file without loading it:** `pp notebook.txt` (in colour), or
`" notebook.txt" slurp type` (plain).

**Files in the browser.** Files that Forth writes are kept in the
browser's IndexedDB for this site, so they are still there after a
reload; a different browser or site has its own. The **Upload** button
copies files from your computer in (then `include NAME`); a file of the
same name is replaced. There is no Download button yet: show a file with
`slurp type` and copy it from the page. [web.md](web.md#files-storage-and-upload)
has the details, and how a package brings its own files.

## The notebook

```text
notebook
```

opens a notebook: a full-window text pane above a results pane. Type a
Forth line and press **Ctrl-J** (or F6, or Cmd-Enter in the page): the
line under the cursor is evaluated, and what it prints, any error and the
stack show in the lower pane. Enter starts a new line, Tab moves between
the panes and the Done button, and F1 lists every key.

**Leaving it:** the **Done** button, the `[×]` at the right of the top
border, or **Ctrl-C** (in the page with no text selected). You are back at the
`ok>` prompt with everything the notebook defined.

Its text is kept in `notebook.txt` in the current folder (in the browser,
with the saved files), saved at each evaluation and when it closes, so
it opens where you left off; the text it had when it opened is kept in
`notebook.bak`. It works in the CLI at a terminal, the browser, the
desktop app and `reva-serve`. A line that draws (`needs turtle/fractals
4 koch-demo`) draws beside it in the browser's split view.

A new notebook starts with lines to try: arithmetic and definitions,
finding words (`words dup`, `help dup`), files (`ls`, `ls /reva/lib`,
`pp notebook.txt`), reading and loading a library. A notebook that has a
`notebook.txt` already opens on that instead. To see the starting lines,
rename it first (`" notebook.txt" " notebook.old" rename`), then
`notebook`.

## Graphics

- **Turtle graphics**: `needs turtle/fractals 5 koch-demo`, and
  `dragon-demo`, `tree-demo`, ...; your own with `forward`, `right`,
  `penup`. In a pane or window beside the REPL, or SVG files from the
  CLI. [turtle.md](turtle.md).
- **IUP dialogs and CD canvases**: Reva's GUI library, its dialogs drawn
  as text, with CD drawings on canvases in the browser and the desktop
  app: `needs ui/cd-demo cd-demo`. [iup.md](iup.md).

## Where next

- [demo.md](demo.md): a scripted walk through every front end.
- [gotchas.md](gotchas.md): what surprises people.
- [diagnostics.md](diagnostics.md): finding where an error happened.
- [parity.md](parity.md): what from the original Reva runs, and what
  does not.
