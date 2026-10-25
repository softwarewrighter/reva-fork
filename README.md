# reva-fork

Reva Forth, carried forward: a fork of Ron Aaron's
[Reva](README.reva) — a compact, fast, native-code Forth for 32-bit x86 —
with a **portable core** that runs the same Forth on any 64-bit machine and
in the browser (WebAssembly).

## What is here

- **The Reva Forth system** (`src/reva.f`, `lib/`, `examples/`, `tests/`,
  `tools/`, …), as imported. Its README is preserved as
  [README.reva](README.reva).
- **The original x86 implementation** — assembly core, build machinery,
  prebuilt 32-bit binaries — is **deprecated** and lives under
  [`legacy/`](legacy/); see [DEPRECATED.md](DEPRECATED.md).
- **The portable core** ([`rust/`](rust/)). Reva compiles Forth straight to
  x86 machine code, and much of `src/reva.f` is *written* in machine code.
  Rather than rewrite all of that, the port executes the small IA-32 subset
  Reva actually uses, with the assembly core's routines re-implemented in
  Rust. `src/reva.f` runs unchanged — byte-for-byte machine-code words,
  control structures, `does>`, `catch`/`throw` — on Linux, macOS and Windows,
  on x86-64 and ARM64, and in a web page.

Documentation: [docs/](docs/README.md) — rebuilt from the portable core, with
tested samples; the old wiki in `www/` is deprecated
([parity tracker](docs/parity.md)).

Rationale, trade-offs and roadmap (including native-speed translation to
x86-64, ARM64 and WebAssembly): [rust/DESIGN.md](rust/DESIGN.md).

## Quick start

```bash
cd rust
cargo build --release
echo ': sq dup * ; 7 sq .' | target/release/reva      # prints: 49
target/release/reva                                  # interactive
scripts/build-web.sh && python3 -m http.server -d reva-web/www
```

Tests: `cargo test`, plus a golden-output regression suite run with
[reg-rs](https://github.com/sw-cli-tools/reg-rs): `scripts/regress.sh`.
See [rust/README.md](rust/README.md).

## Status

The portable core boots `src/reva.f` up to its embedded x86 assembler.
Next: the rest of `reva.f`, file and FFI words, x87 floats, saved images
via `(save)`, then native translators.

## Branches

- `dev` — default branch; integration of finished work.
- feature branches — one per change, branched from and merged back into `dev`.
- `master` — the original import, kept for reference.

## License

- Original Reva Forth: public domain, with a fall-back permission grant —
  [LICENSE](LICENSE).
- New portions (the portable core and accompanying documents): MIT,
  portions copyright © 2026 Michael A Wright — [LICENSE-MIT](LICENSE-MIT).

Details: [LICENSING.md](LICENSING.md). Original author: Ron Aaron
(ron@ronware.org) and the Reva development team.
