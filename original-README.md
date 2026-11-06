# reva-fork: earlier README sections

These sections were part of [README.md](README.md) during the first
releases (October and November 2026). They are kept here as written, for
the record, and are no longer maintained: counts, plans and the gate list
below are out of date. What runs today is in
[docs/parity.md](docs/parity.md) and [docs/README.md](docs/README.md); how
changes are gated is in [CLAUDE.md](CLAUDE.md) and
[components/README.md](components/README.md).

## Status

- **Works:** all of `reva.f`; the REPL; `include` and `needs`; contexts;
  the embedded assembler; the file words (real files on the CLI; in the
  browser, in memory and kept in IndexedDB, plus an Upload button);
  `time&date`, `ms@`,
  `getenv`; saved images (`save`, `--image`, also in the browser) and
  `restart`. 69 of the 132
  programs in `examples/` run unchanged
  ([generated report](docs/parity-examples.md)).
- **Hosts:** the `reva` command on any 64-bit platform Rust supports, and a
  zero-import WebAssembly module with a minimal web console that runs it in
  a Web Worker (the page never freezes; Stop interrupts a running word).
- **x87 floating point** with double-precision registers:
  `lib/math/floats` passes its own tests.
- **Not yet:** native C libraries, and translation to native code.

## Roadmap

1. Virtual libraries (Rust bindings inside the VM; no generic native FFI,
   see [docs/portability.md](docs/portability.md)): SQLite, then a UI
   service (panes, keys, SVG, later WebGL 3D) for the browser and the
   desktop app.
2. Translators that turn hot vx86 code into x86-64, ARM64 and WebAssembly,
   to recover native speed (the interpreter is the reference semantics; the
   translations are caches over it).

The first practical goal on this road is a **static WebAssembly demo of
[CoSy](http://cosy.com/)**, Bob Armstrong's APL/K-like vocabulary built on
Reva: a directory of HTML, JavaScript, CSS and a `.wasm` file that can be
served from any static web server, with no installation. The steps above
that it needs (files, then floats) come first; CoSy-specific code is kept
out of this repository, which receives only general-purpose Reva changes.

## Quality

The port is developed with these qualities in mind, each with a concrete
mechanism described in [docs/design.md §4](docs/design.md#4-non-functional-requirements):

- **Portability and availability:** pure Rust, no `unsafe` in the VM, no
  runtime dependencies; one core for the CLI and the browser.
- **Reliability:** bounds-checked memory, stack checks between words, and
  faults that recover to the REPL and roll back a half-made definition.
- **Traceability and serviceability:** faults name the Forth word and
  offset; `reva --trace` logs every step; a probe tool disassembles words.
- **Maintainability and understandability:** `reva.f` is run verbatim, not
  forked; small modules with literate headers; documented deviations.
- **Testability:** unit and API tests (`cargo test`), a golden-output
  regression suite (`scripts/regress.sh`, using
  [reg-rs](https://github.com/sw-cli-tools/reg-rs)), and executed
  documentation samples.
- **Upgradability:** images hold only vx86 bytes and are versioned; native
  translations are never persisted.

## Branches

- `dev`: the default branch. Every change merges here first.
- feature branches: one per change, branched from `dev` and merged back.
- `main`: the tested release line. `dev` merges into `main` (with a merge
  commit, never a squash, so the two histories stay aligned) once the gates
  pass: `cargo test --release --workspace`, `scripts/doc-check.sh` and the
  reg-rs suite. The original x86 import is the first commit, `edc8cbe`.

