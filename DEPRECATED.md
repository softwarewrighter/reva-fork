# Deprecated: the 32-bit x86 implementation

The portable core in [`components/`](components/README.md) replaces the 32-bit x86 implementation.
The following are **deprecated** and are being retired in stages.

**Current stage: 2 — moved.** Everything below now lives under
[`legacy/`](legacy/) (see [legacy/README.md](legacy/README.md) for the map
and for building it).

| Item | What it is |
|---|---|
| `src/newcore.asm`, `src/coremac.asm`, `src/common.asm`, `src/brieflz.asm` | x86 assembly core |
| `src/linux.asm`, `src/macosx.asm`, `src/windows.asm` | per-OS x86 layers |
| `src/Makefile`, top-level `Makefile`, `tools/build.f`, `src/revares.o`, `src/reva.rc`, `src/reva.ico`, `src/zero`, `test344.bat`, `exclude.lst`, `include.lst` | x86 build and distribution machinery |
| `bin/` (`lin/`, `mac/`, `win/`, `help.db`, …) | prebuilt 32-bit binaries and libraries |
| top-level scratch files (`a.f`, `x.asm`, `x.c`, `sha/`, `sha1*`, `z*.tex`/`.pdf`/`.log`, test binaries, …) | experiments from the original tree |

**Not deprecated:** `src/reva.f`, `src/help.txt`, `lib/`, `examples/`,
`tests/`, `bench/`. These are the Forth system itself; the portable core runs
them.

## Plan

Each step is a separate change, so history stays readable and any step can be
reverted on its own:

1. ✅ **Deprecate** (this notice). Nothing moved.
2. ✅ **Move** the items above under `legacy/` (with `git mv`, preserving
   history), adjusting the legacy Makefile's and build tool's paths so it
   still builds from the repository root (`make -f legacy/Makefile`).
3. **Delete** `legacy/` once the portable core covers what it provided
   (full `reva.f` boot, file words, `(save)`, FFI) and nothing references it.
   Git history keeps the original import (the first commit, `edc8cbe`,
   "imported from fossil repo"). Before
deleting, salvage any scratch `.f` files worth keeping as regression tests.

## Documentation: the old wiki (`www/`)

The wiki pages in [`www/`](www/) describe the x86 distribution and are
**deprecated** too, on their own track:

1. ✅ **Deprecate**: notice in [www/README.md](www/README.md); new
   documentation starts in [`docs/`](docs/), written from the portable
   implementation, with every Forth sample executed by the test suite.
2. **Move** `www/` to `legacy/www/` once [docs/parity.md](docs/parity.md)
   shows every still-supported topic rebuilt (or explicitly superseded or
   dropped).
3. **Delete** `legacy/www/` with the rest of `legacy/`.

The original licensing is unaffected by any of this; see
[LICENSING.md](LICENSING.md).
