# legacy/ — the original 32-bit x86 implementation

Deprecated (see [../DEPRECATED.md](../DEPRECATED.md)). Kept so the original
build stays reproducible until the portable core in [`../rust/`](../rust/)
covers everything it provides; it will then be deleted in a separate change.
Git history follows every file here (`git log --follow legacy/...`), and the
first commit (`edc8cbe`, the fossil import) keeps the original layout.

| Path | Was | What |
|---|---|---|
| `src/*.asm` | `src/` | x86 assembly core and per-OS layers |
| `src/Makefile`, `src/revares.o`, `src/reva.rc`, `src/reva.ico`, `src/zero` | `src/` | build pieces (`src/Makefile` was already stale in the import) |
| `Makefile` | `/` | top-level build, paths updated |
| `tools/build.f` | `tools/` | Reva's build program, paths updated |
| `bin/` | `bin/` | prebuilt 32-bit binaries, libraries, `help.db` (copyleft DLLs removed; see [../docs/licenses.md](../docs/licenses.md)) |
| `lib` → `../lib` | — | symlink: the prebuilt binaries find `lib/` two levels above `bin/<os>/` |
| `test344.bat`, `exclude.lst`, `include.lst` | `/` | Windows test artifact, distribution file lists |
| `scratch/` | `/` | experiments from the original tree (inline-code trials, SHA-1 variants, test binaries, LaTeX output) |

## Building (32-bit x86 toolchain required)

From the repository root, as before but with `-f legacy/Makefile`:

```bash
make -f legacy/Makefile bootstrap   # nasm + gcc -m32: core, then reva, then build
make -f legacy/Makefile             # rebuild reva with the existing build tool
```

The Forth sources the build reads (`src/reva.f`, `tests/`, `tools/genhelp.f`,
`lib/`) are not legacy and stay where they were. `nasm -Ilegacy/` lets the
cores' `%include 'src/...'` lines resolve without editing them; the moved
core assembles to the same code, data and symbols as before the move for all
three targets (checked with `objdump -s` and `nm`).
