# Reva Forth — portable core

A Rust port of the Reva Forth core that runs on any 64-bit host and in the
browser (WebAssembly), while running Reva's own Forth source — including the
words written as x86 machine code — unchanged.

It does this by executing the small subset of 32-bit x86 that Reva's compiler
and primitives use (a "vx86" VM), with the assembly core's routines
re-implemented in Rust. See [design.md](../docs/design.md) for the rationale.

## Status

- All of `src/reva.f` boots verbatim (assembler, contexts, `needs`, the
  FFI layer, the Linux personality) and restarts as its saved executable
  did: banner, `ok>` prompt, `reva.f`'s error handling. See design.md §3.6.
- `needs` loads libraries from `lib/` (via the virtual prefix `/reva/lib/`;
  set `$REVA_HOME` to use another tree).
- A virtual libc (`getenv`, `putenv`, `localtime`, `opendir`/`readdir`)
  and a portable `syscall` subset; `time&date`, `ms@`, `getenv` work.
- File words (`open/r`, `read`, `seek`, `stat`, `rename`, …) through
  `Host::fs`: real files on the CLI, an in-memory file system in the
  browser (kept in `localStorage`); `lib/` is built in. See design.md §3.7.
- Hosts: `reva` CLI (Linux/macOS/Windows, any CPU) and a WebAssembly module
  with a minimal web console; the VM runs in a Web Worker with time slices
  and a Stop button (design.md §3.8).
- Saved images: `save NAME` from Forth or `reva --save-image`; start one
  with `reva --image` or in the browser (`start(..., { image })`).
  `restart` makes the VM again the same way (design.md §3.9).
- x87 floating point with `f64` registers and 80-bit memory operands;
  `lib/math/floats` passes its own tests (design.md §3.10).
- Not yet: native libraries, native translation. See the roadmap in design.md.

## Layout

One Cargo workspace per component (the rules: [code-metrics.md](../docs/code-metrics.md)),
each `components/<name>/` with its `Cargo.toml`, `Cargo.lock` and
`crates/<crate>/`. All of them build into the repository's `target/`.

```
components/
  x86/      reva-x86     vx86: CPU types, decoder, executor, x87, memory, faults, layout
  host/     reva-host    the host side: Host trait, host calls, virtual file system,
                         embedded reva.f and lib/, version, encodings (build script)
  core/     reva-core    the Machine, its run loop, gates, heap, images, canvas
            reva-macros  machine_trait!: methods on Machine from other crates
  vlib/     reva-vlib    virtual libraries: libc, turtle, highlight, sqlite, iconv
  words/    reva-words   the core's words: gate dispatch and the gate families
  vm/       reva-vm      the facade the front ends use (re-exports, Machine with
                         dispatch installed; ../docs/design.md 3.12)
  sqlite/   reva-sqlite  SQLite for the CLI and the desktop app (rusqlite, safe API)
  cli/      reva-cli     the `reva` command
  web/      reva-web     wasm32 module + www/ (reva.js, index.html) + test/*.mjs
  serve/    reva-serve   one-binary local web app: embeds a site, serves 127.0.0.1
                         (std only; ../docs/serve.md)
  desktop/  reva-desktop the Tauri app, with build.sh, smoke.sh, host.js
scripts/      build-web.sh, regress.sh, add-regress.sh, check.sh, bench-guard.sh,
              doc-check.sh (run docs/ samples), parity-examples.sh, ...
tests/forth/  Forth scripts used by the regression suite
work/reg-rs/  reg-rs regression tests (.rgt specs + golden .out/.err)
```

## Build and run

From the repository root:

```bash
just cli                                  # or: cargo build --release --manifest-path components/cli/Cargo.toml
target/release/reva                       # REPL (ok> prompt)
target/release/reva file.f -e '...'       # include files, evaluate text
echo ': sq dup * ; 7 sq .' | target/release/reva
target/release/reva --help
```

WebAssembly:

```bash
scripts/build-web.sh                      # → components/web/crates/reva-web/www/reva.wasm
python3 -m http.server -d components/web/crates/reva-web/www    # open http://localhost:8000
node components/web/crates/reva-web/test/smoke.mjs              # headless check
scripts/make-web-zip.sh                   # → target/web/reva-web-VERSION.zip (../docs/web.md)
scripts/build-serve.sh                    # → target/serve/reva-serve-VERSION-TARGET (../docs/serve.md)
```

## Tests

The gate before every commit, from the repository root:

```bash
just check            # fmt, clippy, tests, wasm + node tests, desktop, goldens, licenses, sw-checklist
just check --fast     # the same without the desktop component
```

`scripts/check.sh` lists the goldens known to fail on each OS and the
most sw-checklist FAILs allowed; both may only go down.

Refactors of hot code (gate dispatch, decode, exec) also run the timing
guard before and after:

```bash
just bench-guard            # boot, fib, a loop, dragons: medians vs this host's baseline
just bench-guard --record   # (re)record this host's baseline in work/bench-guard.txt
```

It fails when a median is more than 15% (`TOLERANCE`) slower than the
host's recorded baseline; timings only compare on the same machine, so
each host has its own lines. It is not part of `just check` (it takes
about 17 s and timing is noisy on a busy machine). The pieces of the
gate:

```bash
just test                                 # unit + API tests, every component
scripts/regress.sh                        # reg-rs golden-output suite
scripts/regress.sh -vv -p core-catch      # one test, full diff on failure
REVA=target/release/reva scripts/regress.sh
```

The regression suite uses [reg-rs](https://github.com/sw-cli-tools/reg-rs)
(`reg-rs` on PATH, or `REG_RS=/path/to/reg-rs`). Add a test with

```bash
scripts/add-regress.sh NAME '${REVA:-target/debug/reva} < tests/forth/NAME.f' 'what it covers'
```

and review the captured `work/reg-rs/NAME.out` before committing: a baseline
asserts the output is *correct*. Accept an intended change with
`REG_RS_DATA_DIR=work/reg-rs reg-rs rebase -p NAME`.

Source as color-highlighted HTML or PNG, for slides, docs and blog figures
(Python 3 standard library; `--png` uses headless Chrome or Chromium):

```bash
scripts/forth-highlight.py lib/turtle/fractals --lines 22-30 --title 'Koch' --png koch.png
```

Documentation lives in [`../docs/`](../docs/README.md); its Forth samples are
checked by `scripts/doc-check.sh docs/*.md`, and
`scripts/parity-examples.sh` reports how far every program in `examples/`
gets (both are part of the regression suite).

The scripts filter text with [sed-rs](https://github.com/sw-cli-tools/sed-rs)
(`cargo install --path .` in its checkout, or set `$SED_RS`), never the
system sed, so they behave the same on every platform. Patterns use
sed-rs syntax: Rust regex, where `( ) | + ?` are operators; write a
literal one as a bracket (`[(]`, `[?]`).

## Debugging

- Faults name the Forth word: `*** bad memory access ... (in bad+0xc, ...)`.
- `reva --trace` logs every step to stderr.
- Code the core cannot run (native libraries, `shell`, MMX/SSE and
  privileged instructions) stops with a message pointing to
  [docs/portability.md](../docs/portability.md); `reva --allow-shell` lets
  `shell` run OS commands.
- `cargo run -p reva-vm --example probe -- ': f 1 2 + ;' 'see f'` runs
  lines and disassembles words.

## License

MIT, Copyright (c) 2026 Michael A. Wright, for the code in this
directory (see [LICENSE](LICENSE)). The Reva
Forth sources it embeds and runs (`src/reva.f`) remain in the public domain
under the original terms; see [../LICENSING.md](../LICENSING.md).
