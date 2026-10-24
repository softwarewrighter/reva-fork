# Deprecated: the 32-bit x86 implementation

The portable core in [`rust/`](rust/) replaces the 32-bit x86 implementation.
The following are **deprecated** and will be retired in stages:

| Item | What it is |
|---|---|
| `src/newcore.asm`, `src/coremac.asm`, `src/common.asm`, `src/brieflz.asm` | x86 assembly core |
| `src/linux.asm`, `src/macosx.asm`, `src/windows.asm` | per-OS x86 layers |
| `src/Makefile`, top-level `Makefile`, `src/revares.o`, `src/reva.rc`, `src/zero`, `test344.bat` | x86 build machinery |
| `bin/lin/`, `bin/mac/`, `bin/win/` | prebuilt 32-bit binaries and libraries |
| top-level scratch files (`a.f`, `x.asm`, `x.c`, `sha1*`, `z*.tex`/`.pdf`/`.log`, …) | experiments from the original tree |

**Not deprecated:** `src/reva.f`, `src/help.txt`, `lib/`, `examples/`,
`tests/`, `bench/`. These are the Forth system itself; the portable core runs
them.

## Plan

Each step is a separate change, so history stays readable and any step can be
reverted on its own:

1. **Deprecate** (this notice). Nothing moves; the old build keeps working
   for anyone with a 32-bit x86 toolchain.
2. **Move** the items above under `legacy/` (with `git mv`, preserving
   history), adjusting the legacy Makefile's paths so it still builds.
3. **Delete** `legacy/` once the portable core covers what it provided
   (full `reva.f` boot, file words, `(save)`, FFI) and nothing references it.
   Git history and the `master` branch keep the original import.

The original licensing is unaffected by any of this; see
[LICENSING.md](LICENSING.md).
