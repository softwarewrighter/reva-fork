# Getting started

Replaces the old wiki's install pages (`linux.wiki`, `mac.wiki`): the
portable core is one self-contained binary, so there is no `PATH` /
`LD_LIBRARY_PATH` / `DYLD_LIBRARY_PATH` setup, no `/proc` lookup, and no
per-OS `bin/` directory.

## Build

You need a Rust toolchain (`rustup`, Rust 1.82 or newer).

```bash
git clone https://github.com/softwarewrighter/reva-fork
cd reva-fork/rust
cargo build --release
```

The binary is `rust/target/release/reva`. It runs on Linux, macOS and
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
| `reva --save-image app.img -e '...'` | save the state after loading, then exit |
| `reva --image app.img` | start from a saved image |
| `reva --help` | all options |

The exit status is 0 after `bye` or at end of input. `reva.f` keeps the
core's `(bye)`, which exits with a status from the stack, in the `~sys`
context:

```bash
echo '3 ~sys.(bye)' | reva; echo $?      # 3
```

When standard input is a terminal, `reva` starts as Reva always did: it
prints the banner (`Reva 2011.2 Linux`) and an `ok>` prompt before each
line. Piped or redirected input gets neither, so batch output is just what
the program prints.

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
cd rust
scripts/build-web.sh                        # builds reva-web/www/reva.wasm
python3 -m http.server -d reva-web/www      # then open http://localhost:8000
```

The page is a small console running the same core. Long-running or looping
code does not freeze the page: the VM runs in slices and yields to the
browser between them.

In the browser, files live in memory: `lib/` is built in (so `needs`
works), and files your code writes are kept in the browser's
`localStorage`, so they are still there after a reload. A page can bundle
its own files too; see `Reva.load` in `reva-web/www/reva.js`.

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

## What works today

All of `src/reva.f` boots: the core words, the assembler, contexts,
`needs` and the libraries in `lib/` that do not call native C libraries,
and the file words. Floating point (x87) and native libraries are next.
[parity.md](parity.md) tracks this in detail.
