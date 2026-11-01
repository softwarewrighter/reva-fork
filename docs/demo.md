# Demo script

A walk through everything the portable core can show today, in the order
that builds on itself: the terminal REPL, the browser REPL, the desktop app,
and a package. Each step says what to type and what you should see. The
Forth samples in `forth`/`output` blocks are executed by
`scripts/doc-check.sh`, so the output shown is what `reva` prints.
CoSy, which runs as a package, has its own script in
[ba-cosy-zip-fork](https://github.com/softwarewrighter/ba-cosy-zip-fork)
(`reva-fork-subset/DEMO.md`).

Allow about 15 minutes, plus a few minutes for the first desktop build.

## 0. Pull and build

```bash
git switch dev && git pull          # or main, once dev is merged there
cargo build --release --manifest-path components/cli/Cargo.toml   # ~20 s; the binary is target/release/reva (or: just cli)
export PATH="$PWD/target/release:$PATH"
just test                           # optional: the full test suite
```

The gate scripts (`scripts/doc-check.sh`, `scripts/regress.sh`) also need
[sed-rs](https://github.com/sw-cli-tools/sed-rs) and
[reg-rs](https://github.com/sw-cli-tools/reg-rs) on `PATH` (`cargo install
--path .` in each checkout).

The binary is self-contained: `reva.f` and `lib/` are compiled in.

## 1. The terminal REPL

Run `reva` with no arguments. It prints the banner and a prompt:

```text
reva-fork 2026.1026.0 portable
ok>
```

Type these lines one at a time (the prompt is left out below):

```forth
: sq dup * ;
: cube dup sq * ;
3 cube .
```
```output
27
```

**Libraries** load with `needs`, from the copy built into the binary:

```forth
needs string/soundex
" Robert" soundex type cr
```
```output
R163
```

**Errors do not end the session.** An unknown word skips the rest of its
line; a fault (here, reading address 0) names the word it happened in and
returns to the prompt with an empty stack:

```forth
nosuchword 2 .
0 @ .
1 2 + .
```
```output

ERROR: unknown word:  nosuchword
...
*** bad memory access: 4 byte(s) at 0x00000000 (in @+0x2, eip 0x00040c1d)
3
```

**OS commands are off by default** (see [portability.md](portability.md)):

```text
ok> needs os/shell
ok> " echo hi" shell
*** shell (running an OS command) is not allowed here: the CLI allows it with
--allow-shell; the browser never does (see docs/portability.md) (in core:CallFn, ...)
```

`reva --allow-shell` lets the same line print `hi`.

**`version`** prints the build report (also `reva -V`): version, license,
repository, and the host, commit and time of the build.

Leave with `bye` or Ctrl-D.

## 2. Batch use, files, images and `restart`

From a scratch directory:

```bash
echo '2 3 + .' | reva                                   # 5, no banner or prompt
echo '3 ~sys.(bye)' | reva; echo $?                     # 3: exit status from Forth
echo ': hi ." file ok" cr ; hi' > t.f && reva t.f </dev/null   # file ok
```

Files are real files on the command line:

```forth
" reva-demo.txt" creat value h
" hello, file" h write  h close
" reva-demo.txt" slurp type cr
" reva-demo.txt" delete
```
```output
hello, file
```

A saved image starts without compiling anything again:

```bash
echo ': hi ." from the image" cr ;  save app.img' | reva   # Saving: app.img ...
echo hi | reva --image app.img                            # from the image
```

`restart` starts a fresh system in the same process; definitions are gone
and the remaining input is read by the new system:

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

## 3. The browser REPL

```bash
scripts/build-web.sh                        # components/web/crates/reva-web/www/reva.wasm (~1 MB)
python3 -m http.server -d components/web/crates/reva-web/www      # any static server works
```

Open <http://localhost:8000> (over HTTP: a worker cannot start from
`file://`). Then:

| Type | You see |
|---|---|
| `: sq dup * ;` then `7 sq .` | `49` |
| `" notes.txt" creat value h  " kept" h write  h close` | nothing (ok) |
| reload the page, then `" notes.txt" slurp type` | `kept`: files persist in `localStorage` |
| `: forever repeat again ; forever` | the page stays responsive; **Stop** is enabled |
| click **Stop** (or Ctrl-C) | `*** interrupted (in forever+0x0, ...)` and `ok>` |
| `2 3 + .` | `5`: definitions survived the interrupt |

URL parameters: `?eval=TEXT` feeds a line once, `?include=FILE` includes a
file (from the page's own files) at start and after each `restart`,
`?image=app.img` starts from a saved image served next to the page. Try
<http://localhost:8000/?eval=%3A%20sq%20dup%20*%20%3B%209%20sq%20.>, which
prints `81`.

## 4. The desktop app

Needs WebKitGTK 4.1 headers on Linux (`sudo apt install
libwebkit2gtk-4.1-dev build-essential`) or Xcode's command line tools on
macOS. From the repository root:

```bash
components/desktop/build.sh --install    # first build ~4 min; installs reva-desktop into ~/.cargo/bin
components/desktop/smoke.sh              # five checks, all "ok" (uses xvfb-run without a display)
reva-desktop                  # the REPL in a window
```

In the window, everything from step 3 works the same way, because it is the
same page and the same WebAssembly VM. The difference is where files live:

| Type | You see |
|---|---|
| `" notes.txt" creat value h  " on disk" h write  h close` | then `cat ~/Reva/notes.txt` in a terminal prints `on disk` |
| `bye` | the window closes |

Batch mode, which is how `smoke.sh` tests it:

```bash
reva-desktop --root /tmp/r --print-output -e ': sq dup * ; 7 sq .' -e bye   # ... 49
```

## 5. A package

A package is a folder with a `reva-package.json` naming start files
([desktop.md](desktop.md#packages)). Make one:

```bash
mkdir -p /tmp/hello/src
echo '{"name": "Hello", "start": ["src/start.f"]}' > /tmp/hello/reva-package.json
echo ': hi ." package started" cr ; hi' > /tmp/hello/src/start.f
reva-desktop --package /tmp/hello
```

The window is titled `Hello` and shows `package started`. Type `: gone ;`,
then `restart`: `package started` appears again and `gone` is unknown, so
the start files ran in a fresh system. CoSy is the real example:
`reva-desktop --package <ba-cosy-zip-fork>/reva-fork-subset/build`.

## What is not there yet

x87 floating point, native C libraries (IUP, SQLite, ...), the UI service
(package pages, drawing) and 3D. [parity.md](parity.md) tracks the details;
[packaging.md](packaging.md) covers handing builds to other people, and
Windows.
