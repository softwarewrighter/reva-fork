# Parity: old documentation and examples vs. the portable core

The old wiki ([`www/`](../www/)) documents the 32-bit x86 distribution and is
**deprecated**. This page tracks what of it, and of the example programs,
works on the portable core and has been rebuilt here. The goal is parity for
everything still supported, not a copy of the old pages: the new pages are
written from the implementation and their samples are checked by tests.

Status legend: **done** — rebuilt and checked · **partial** — some of it ·
**ready** — what it covers works on the portable core; the page is not
rebuilt yet · **blocked** — needs a roadmap item first (see
[DESIGN.md §7](../rust/DESIGN.md#7-roadmap)) · **superseded** — no longer
applies · **decide** — needs a decision on whether it is still supported.

## Wiki pages

| Old page | Topic | Status | Replacement / blocker |
|---|---|---|---|
| `index.wiki` | manual front page, "what is Reva" | partial | [README.md](README.md), [../README.md](../README.md); the manual's concept/internals chapters were links to pages not in this repository |
| `linux.wiki` | install on Linux (PATH, LD_LIBRARY_PATH, /proc) | superseded | [getting-started.md](getting-started.md): one binary, no library paths, no /proc |
| `mac.wiki` | install on macOS, IUP/GTK setup | superseded / decide | install: [getting-started.md](getting-started.md); IUP: see GUI decision below |
| `LICENSE.wiki` | license | superseded | [../LICENSE](../LICENSE), [../LICENSING.md](../LICENSING.md) |
| `gotchas.wiki` | pitfalls | done (1 item pending FFI) | [gotchas.md](gotchas.md) |
| `libraries.wiki` | `needs`, library layout | ready | `needs` works; libraries resolve under the virtual prefix `/reva/lib/` (CLI: the checkout or `$REVA_HOME`). Libraries that call native C libraries stop with a note naming the library until a virtual library serves them ([portability.md](portability.md)) |
| `tutorial.wiki` | basic tutorial | ready | see sections below |
| `tutorial-intermediate.wiki` | `words`, `dump`, dictionary, internals | ready / rewrite | `words` and the dictionary work; internals chapters must be rewritten for the vx86 core |

### Manual chapters linked from `index.wiki`

The front page links to chapters that are not in this repository (they lived
on the old online wiki). They are tracked here so the new documentation
covers the same ground.

| Chapter | Status | Replacement / plan |
|---|---|---|
| Installation, Command-line parameters | done | [getting-started.md](getting-started.md); a walk-through in [demo.md](demo.md) |
| Distribution (the old `bin/` per OS, `tools/build.f`) | superseded | [packaging.md](packaging.md): one self-contained binary per target, a static web zip, desktop installers; Windows status and test plan |
| Glossary, Built-in help system | blocked | `src/help.txt` (6,000 lines, in the repository) is the source: generate a glossary page from it, with a note per word on whether it works yet; `help` will read a text index built from it instead of the SQLite `help.db` (see Decisions) |
| Turnkey (standalone applications) | ready | `save NAME` (and `(save)`) write a vx86 image instead of an executable; start it with `reva --image NAME` or in the browser (`start(..., { image })`). `restart` starts over from the same image or boot |
| Cells and Stacks, Words and the dictionary, Numbers, Strings | partial | material exists across [gotchas.md](gotchas.md) and [DESIGN.md](../rust/DESIGN.md); `reva.f` now boots fully, so topic pages can be written |
| Deferred words, Contexts, Libraries | ready | `defer`/`is`, `context:`/`with~`/`~ctx.word`, `needs` all work |
| Design goals, Tips and Tricks, Comparisons (RetroForth/HelFORTH, ANS), Benchmarks | decide | historical; benchmarks to be redone with `bench/` on the new core |
| Development | superseded | [../rust/README.md](../rust/README.md) (tests, reg-rs, doc-check) |

### Basic tutorial, by section

`reva.f` now boots fully and the file words work, so everything the
tutorial uses works except floats (roadmap 3). Sections are rebuilt as
pages are written.

| Section | Subject works today? | Status |
|---|---|---|
| Forth Syntax | yes | ready |
| Stack Manipulation | yes | ready |
| Arithmetic, More Arithmetic, Arithmetic Overflow | yes | ready |
| Defining a New Word | yes | ready |
| Convert Algebraic Expressions | yes | ready |
| Character Input and Output | yes | ready |
| Compiling from Files | yes (`include`) | ready |
| Variables, Constants | yes | ready |
| Logical Operators, Conditionals | yes | ready |
| Loops | yes | ready |
| Text Input and Output | yes (`."`; `accept` reads the input queue) | ready |
| Changing Numeric Base | yes (`hex`, `decimal`, `binary`, prefixes) | ready |
| Libraries | yes (`needs`) | ready |
| Files & File Handling | yes (`open/r`, `open/rw`, `creat`, `read`, `write`, `seek`, `stat`, `rename`, `delete`; CLI: real files, browser: in memory + `localStorage`) | ready |

## Example programs

[parity-examples.md](parity-examples.md) is generated by running every
program in `examples/` (132 of them) on the portable core. With all of
`reva.f` booting, 64 run cleanly (none did before; 10 of them since `save` writes images). The rest, by cause:
(the file words are in; the one example that needed them, `sha1.f`, now
computes correct digests but loops when given no file names: `argc 2 do`
with `argc` = 2, as on the original)

| Cause | Programs | Roadmap |
|---|---|---|
| a native C library (`iconv`, IUP, SQLite, GTK, pthreads, ...) | 35 | 4 (virtual libraries, [portability.md](portability.md)), or the GUI decision below |
| x87 floating point (`fldz` ...) | 6 | 3 |
| interactive or endless (time limit) | 12 | none: expected with stdin closed |
| other faults and missing words (one is `lib/asm`, which ran the external FASM assembler) | 15 | investigate per program |

| Area | Programs | Proposed support | Notes |
|---|---|---|---|
| core | 45 | yes | pure Forth; `reva.f` boots fully |
| lib | 37 | yes | `needs` + the libraries in `lib/` |
| os | 9 | yes, through the host | files: done (CLI and browser); processes, threads: CLI host first; browser where meaningful |
| net | 7 | CLI yes; browser decide | sockets, HTTP, CGI |
| db | 5 | yes (SQLite), later | decided: SQLite compiled into the VM as a virtual library on every host; MySQL dropped (GPL client) |
| gui | 29 | new UI library | decided: a host UI service (browser first, then the [desktop app](desktop.md), which exists now as a REPL window that packages can brand with their own title and icon); IUP/GTK/OpenGL/Win32 programs stay in `examples/` for reference |

## Decisions

Items 1 and 2 were decided on 2026-10-25, together with the license policy
in [licenses.md](licenses.md); item 3 is still open.

1. **GUI programs (29).** No native GUI libraries: the core reaches C
   libraries only through virtual libraries (Rust bindings inside the VM),
   because Forth passes 32-bit VM addresses that native 64-bit code cannot
   use. A small UI service (panes, text, key bindings, an SVG surface) will
   be served by the browser page and, for Linux and macOS (Windows later), by
   a Tauri desktop shell that reuses the same page. Selected demos get ported
   to it; the IUP/GTK/OpenGL/Win32 programs are not counted toward parity.
2. **Database and network.** Help (`help`) will read a text index built
   from `src/help.txt` instead of `help.db`. SQLite will later be compiled
   into the VM as a virtual `libsqlite3`, so `lib/db/sqlite` runs unchanged
   on the CLI, the desktop and in the browser. `lib/db/mysql` is dropped (its
   client library is GPL). Sockets and HTTP become host services when needed.
3. **Tutorial internals chapters.** The intermediate tutorial explains the
   x86 interpreter and compiler. The vx86 core keeps the same compiled code,
   so most of it stays true, but the "inner workings" chapters should be
   rewritten around the gateway design rather than ported.

## Keeping this page honest

- `rust/scripts/license-check.sh` must pass (no copyleft code, every binary
  reviewed in [licenses.md](licenses.md)).
- `rust/scripts/doc-check.sh ../docs/*.md` must pass (run by the regression
  suite as `docs-samples`).
- `rust/scripts/parity-examples.sh` output is a regression baseline
  (`parity-examples`): when an example's status changes, the suite fails
  until the change is accepted (`reg-rs rebase -p parity-examples`) and
  `--write` regenerates [parity-examples.md](parity-examples.md).
- Update the tables above in the same pull request that changes a status.
