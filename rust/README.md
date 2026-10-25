# Reva Forth — portable core

A Rust port of the Reva Forth core that runs on any 64-bit host and in the
browser (WebAssembly), while running Reva's own Forth source — including the
words written as x86 machine code — unchanged.

It does this by executing the small subset of 32-bit x86 that Reva's compiler
and primitives use (a "vx86" VM), with the assembly core's routines
re-implemented in Rust. See [DESIGN.md](DESIGN.md) for the rationale.

## Status

- All of `src/reva.f` boots verbatim (assembler, contexts, `needs`, the
  FFI layer, the Linux personality) and restarts as its saved executable
  did: banner, `ok>` prompt, `reva.f`'s error handling. See DESIGN.md §3.6.
- `needs` loads libraries from `lib/` (via the virtual prefix `/reva/lib/`;
  set `$REVA_HOME` to use another tree).
- A virtual libc (`getenv`, `putenv`, `localtime`) and a portable `syscall`
  subset; `time&date`, `ms@`, `getenv` work.
- Hosts: `reva` CLI (Linux/macOS/Windows, any CPU) and a WebAssembly module
  with a minimal web console.
- Not yet: file I/O words (`open/r` …), `(save)`, native libraries, x87
  floats, native translation. See the roadmap in DESIGN.md.

## Layout

```
rust/
  reva-vm/     the core library (VM, gateways, images, translator hooks)
  reva-cli/    the `reva` command
  reva-web/    wasm32 module + www/ (reva.js, index.html) + test/smoke.mjs
  tests/forth/ Forth scripts used by the regression suite
  work/reg-rs/ reg-rs regression tests (.rgt specs + golden .out/.err)
  scripts/     build-web.sh, regress.sh, add-regress.sh,
               doc-check.sh (run docs/ samples), parity-examples.sh
```

## Build and run

```bash
cd rust
cargo build --release
target/release/reva                       # REPL (ok> prompt)
target/release/reva file.f -e '...'       # include files, evaluate text
echo ': sq dup * ; 7 sq .' | target/release/reva
target/release/reva --help
```

WebAssembly:

```bash
scripts/build-web.sh                      # → reva-web/www/reva.wasm
python3 -m http.server -d reva-web/www    # open http://localhost:8000
node reva-web/test/smoke.mjs              # headless check
```

## Tests

```bash
cargo test                                # unit + API tests
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

Documentation lives in [`../docs/`](../docs/README.md); its Forth samples are
checked by `scripts/doc-check.sh ../docs/*.md`, and
`scripts/parity-examples.sh` reports how far every program in `examples/`
gets (both are part of the regression suite).

## Debugging

- Faults name the Forth word: `*** bad memory access ... (in bad+0xc, ...)`.
- `reva --trace` logs every step to stderr.
- `cargo run -p reva-vm --example probe -- ': f 1 2 + ;' 'see f'` runs
  lines and disassembles words.

## License

MIT, portions copyright © 2026 Michael A Wright, for the code in this
directory (see [LICENSE](LICENSE)). The Reva
Forth sources it embeds and runs (`src/reva.f`) remain in the public domain
under the original terms; see [../LICENSING.md](../LICENSING.md).
