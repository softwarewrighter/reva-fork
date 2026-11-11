# Getting started

Replaces the old wiki's install pages (`linux.wiki`, `mac.wiki`): the
portable core is one self-contained binary, so there is no `PATH` /
`LD_LIBRARY_PATH` / `DYLD_LIBRARY_PATH` setup, no `/proc` lookup, and no
per-OS `bin/` directory.

## Build

You need a Rust toolchain (`rustup`, Rust 1.89 or newer: the code is edition 2024)
and [just](https://just.systems) for the recipes (each is a thin wrapper
over a script, so it is optional).

```bash
git clone https://github.com/softwarewrighter/reva-fork
cd reva-fork
just cli          # build the CLI and start the REPL (or: just cli-build)
```

The binary is `target/release/reva`. [tour.md](tour.md) shows how to get
around once it runs. It runs on Linux, macOS and
Windows, on x86-64 and ARM64 alike. `reva.f` is compiled into it; nothing
else needs to be installed.

## Using `reva`

Interactively, `reva` shows an `ok>` prompt; type Forth and press Enter.
Leave with Ctrl-D (end of input) or `bye`.

Piped input runs as a batch and exits at the end of input:

```bash
echo '2 3 + .' | reva
```

which prints:

```forth
2 3 + .
```
```output
5
```

Definitions work as you would expect:

```forth
: sq dup * ;
: cube dup sq * ;
3 cube .
```
```output
27
```

Other ways to run code:

| Command | Does |
|---|---|
| `reva file.f` | include `file.f`, then read stdin |
| `reva -e 'text'` | evaluate `text` (after any files), then read stdin |
| `reva file.f < /dev/null` | run a file as a batch |
| `reva prog.f -- a b` | run a program with arguments: `argc` is 4; `0 argv` is `reva`, `1 argv` `prog.f`, `2 argv` `a`, `3 argv` `b` (as in Reva; e.g. `reva examples/sha1.f -- FILE ...`) |
| `reva --save-image app.img -e '...'` | save the state after loading, then exit |
| `reva --image app.img` | start from a saved image |
| `reva --max-cpu SECONDS ...` | end a run that computes that long without waiting for input (`*** time limit`, exit 124; default 30 s with `--ui-script`) |
| `reva --underflow fail ...` | stop with an error when a word takes more than the stack holds (default `warn`: say so once and go on, as Reva did); `fail-at-word` stops in the word that did it, with its callers ([diagnostics.md](diagnostics.md#underflow-inside-a-definition)) |
| `reva --check-execute ...` | stop an `execute` of what is not code, naming the word that executed ([diagnostics.md](diagnostics.md#execute-of-what-is-not-code)) |
| `reva --trace-compile ...` | say each word compiled into a definition, and who compiled it ([diagnostics.md](diagnostics.md#what-is-compiled-and-by-whom)) |
| `reva --port-check [--fix DIR] FILE...` | find Windows-only paths and commands; write portable copies ([porting-from-windows.md](porting-from-windows.md)) |
| `reva --ui-script FILE prog.f` | run an IUP program headless, its events from FILE ([ui-model.md](ui-model.md#the-virtual-library-iup)) |
| `reva --allow-shell ...` | let `shell` run OS commands (off by default; [portability.md](portability.md)) |
| `reva --draw svg\|png\|off ...` | turtle drawings to new files `turtle-NNN.svg`/`.png` (default: at a terminal) ([turtle.md](turtle.md)) |
| `reva --color always\|never ...` | colour in `pp` listings (default: auto) ([highlight.md](highlight.md)) |
| `reva --no-edit` | at a terminal, plain lines: no history or line editing ([repl.md](repl.md)) |
| `reva --boot PATH`, `--no-boot`, `--show-boot`, `--trace` | for work on the core: another `reva.f`, none, what booting printed, every VM step |
| `reva --help` | all options |
| `reva -V` | version and build information (`--version`) |

The exit status is 0 after `bye` or at end of input. `reva.f` keeps the
core's `(bye)`, which exits with a status from the stack, in the `~sys`
context:

```bash
echo '3 ~sys.(bye)' | reva; echo $?      # 3
```

When standard input is a terminal, `reva` starts as Reva always did: it
prints the banner (`reva-fork YYYY.MMDD.N portable`, the release) and an `ok>` prompt before
each line. Piped or redirected input gets neither, so batch output is just
what the program prints.

At the prompt, `version` prints the same report as `reva -V`: the version,
copyright, license, repository, and the host, commit and time of the build.
`.ver` prints the banner. `revaver` still gives `2011.2`, the release of
`reva.f` this core runs, because libraries compare against it.

## Errors

An unknown word is reported by `reva.f`'s handler and the rest of that line
is skipped; the session continues. Inside an included file the message also
shows the source that follows (up to 64 characters):

```forth
1 .
nosuchword 2 .
3 .
```
```output
1
ERROR: unknown word:  nosuchword
...3
```

A fault (for example reading address 0) is reported with the Forth word it
happened in, and the session continues with an empty stack. A definition
that fails half way is removed.

## In the browser

```bash
just web-dev      # builds www/reva.wasm (scripts/build-web.sh), serves http://localhost:8000
just serve        # or: the page inside reva-serve, one binary, opened in your browser
```

The live demo is the same page: <https://softwarewrighter.github.io/reva-fork/>.

The page is a small console running the same core. The VM runs in a Web
Worker, so booting, long computations and endless loops never freeze the
page. **Stop** (or Ctrl-C) interrupts the running word and returns to the
`ok>` prompt with your definitions kept. Serve the directory over HTTP: a
worker cannot start from a `file://` page.

In the browser, files live in memory: `lib/` is built in (so `needs`
works), and files your code writes are kept in the browser (IndexedDB),
so they are still there after a reload. The **Upload** button brings files
from your computer in; then `include NAME`. A page can bundle
its own files too; see `start` in `reva-web/www/reva.js`.

## Files

The file words work on the command line (real files) and in the browser
(the in-memory files above). `ioerr` is 0 after success and a negative
error number after failure:

```forth
" reva-demo.txt" creat value h
" hello, file" h write  h close
" reva-demo.txt" open/r value r
pad 100 r read  pad swap type cr  r close
" reva-demo.txt" delete  ioerr @ .
" reva-demo.txt" open/r drop  ioerr @ .
```
```output
hello, file
0 -2
```

`-2` is "no such file". [gotchas.md](gotchas.md) lists the quirks.

## Saved images and `restart`

`save NAME` writes everything defined so far to an image file; starting
from it skips compiling `reva.f` and your program again. The original
wrote a native executable; here an image runs on every host:

```bash
echo ': hi ." from the image" cr ;  save app.img' | reva
echo hi | reva --image app.img          # from the image
```

In the browser, pass the image to `start` (`start("reva.wasm", { image:
"app.img" })` in `reva-web/www/reva.js`), or open the console page as
`index.html?image=app.img`; a package names its image in its manifest
(`"image": "app.img"`), and the desktop app takes `--image app.img`
([desktop.md](desktop.md)). `reva --save-image` keeps strings compiled
after boot as `save` does, so the two images are the same. A page's start files (`index.html?include=app.f`,
or a desktop package's `start` list) are included again after `restart`, as
the CLI re-runs its file arguments.

`restart` starts over the way this run started (from `reva.f`, or from the
image), as relaunching the program did. Definitions made since are gone;
files are kept, and the lines not read yet are read by the new system. On
the command line, files and `-e` texts given to `reva` run again.

```forth
: greet ." hello" cr ;
greet
restart
greet
2 3 + .
```
```output
hello

ERROR: unknown word:  greet
...5
```

## Floating point

`lib/math/floats` gives the float words (`f+`, `f.`, `fsqrt`, `fsin`, …),
which use a separate floating-point stack:

```forth
needs math/floats ~floats
2 s>f fsqrt f.
" 1.5" >float drop  2 s>f f*  f.
```
```output
1.41421 3.00000
```

`>float` leaves a flag (true if the text was a number); `f.` prints
`sigdig` (5) digits after the point, cut rather than rounded.
[gotchas.md](gotchas.md) says how the precision differs from the original.

## What works today

The portable core boots all of `src/reva.f`: the core words, control
structures and loops, `create`/`does>`, `catch`/`throw`, strings and `."`,
the embedded assembler, contexts, `include`, `needs` with the libraries
in `lib/`, the file words (`open/r`, `read`, …), saved images with `save`
and `restart`, floating point (`needs math/floats`), turtle graphics
([turtle.md](turtle.md)), IUP dialogs and CD drawing ([iup.md](iup.md)),
and SQLite ([sqlite.md](sqlite.md)). Native C libraries cannot be called;
iconv, SQLite, IUP and CD are served by Rust in their place
([portability.md](portability.md)).
[parity.md](parity.md) tracks this in detail, and
[parity-examples.md](parity-examples.md) lists every example program's
status.
