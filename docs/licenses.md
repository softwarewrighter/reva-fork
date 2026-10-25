# Third-party licenses

Who wrote what in this repository, under which terms, and the rule that keeps
it that way. [LICENSING.md](../LICENSING.md) gives the terms of the project's
own parts; this page covers everything else.

## Policy

- **No copyleft.** The repository and everything built from it (the `reva`
  CLI, the wasm module, future desktop builds) contain no GPL, LGPL or AGPL
  code. Permissive terms only: public domain, MIT, BSD, Apache-2.0, ISC,
  zlib and similar.
- **No native libraries at run time.** The portable core reaches C libraries
  only through *virtual libraries*: Rust code inside the VM that serves the
  old library names (libc today; see [DESIGN.md](../rust/DESIGN.md) §3.6).
  A library that is not served resolves to 0, as a failed `dlopen` did. So
  nothing in `legacy/bin/` is loaded by the portable core.
- **Every binary is reviewed.** A tracked binary (executable, shared library,
  object file, archive) must be listed below with its license.

Why: the project should be usable commercially without conditions beyond
credit, on every platform, and a reader should be able to see where each
file came from.

## Enforcement

`rust/scripts/license-check.sh` (run it from `rust/`) fails when:

1. any tracked file carries GPL, LGPL or AGPL license text;
2. a tracked binary is not on its reviewed list, or a listed binary is
   missing from this page;
3. `cargo deny check licenses` fails ([`rust/deny.toml`](../rust/deny.toml)
   allows MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, Unicode-3.0, CC0-1.0
   and Unlicense). This step runs when `cargo-deny` is installed; the
   workspace has no crate dependencies today.

The repository has no CI, so run it with the other local release checks
(`cargo test --release`, `scripts/doc-check.sh`, the regression suite).

## Removed

Deleted from the tree on 2026-10-25 because they are copyleft. Nothing in the
portable core used them. They remain in git history, as they were already
published upstream; history was deliberately not rewritten, since that would
break every existing clone.

| Path | What | License |
|---|---|---|
| `legacy/scratch/sha1a.c`, `legacy/scratch/sha1a.h` | PolarSSL SHA-1 source (an experiment; the binaries next to it use Steve Reid's public-domain SHA-1) | GPL-2.0-or-later |
| `legacy/bin/win/libmySQL.dll` | MySQL client, loaded by `lib/db/mysql` | GPL-2.0 with FOSS exception |
| `legacy/bin/win/libgpgme-11.dll`, `legacy/bin/win/libgpg-error-0.dll` | GnuPG Made Easy, loaded by `lib/crypt/gpg` | LGPL-2.1-or-later |
| `legacy/bin/win/libiconv2.dll`, `legacy/bin/lin/libiconv.so` | GNU libiconv, loaded by `lib/string/iconv` | LGPL-2.1-or-later |
| `legacy/bin/win/pthread.dll` | pthreads-win32, loaded by `lib/os/threads` | LGPL-2.1 |

Effect on the deprecated x86 build: it still builds (`make -f
legacy/Makefile`); programs that `needs` those libraries must get them from
the system, as they already did on Linux and macOS.

## Reviewed binaries

All under `legacy/` or `contrib/`; none is used by the portable core.

| Path | Origin | License |
|---|---|---|
| `legacy/bin/lin/build`, `legacy/bin/lin/core`, `legacy/bin/lin/reva` | Reva | public domain |
| `legacy/bin/mac/build`, `legacy/bin/mac/core`, `legacy/bin/mac/reva` | Reva, packed with UPX (whose license exempts packed programs) | public domain |
| `legacy/bin/win/build.exe`, `legacy/bin/win/core.exe`, `legacy/bin/win/reva.exe` | Reva | public domain |
| `legacy/src/revares.o` | Reva's Windows resources | public domain |
| `legacy/bin/lin/libiupall.so`, `legacy/bin/win/iupall.dll`, `legacy/bin/mac/libiup.dylib`, `legacy/bin/mac/libiupcd.dylib`, `legacy/bin/mac/libiupcontrols.dylib`, `legacy/bin/mac/libiupgl.dylib`, `legacy/bin/mac/libiupim.dylib`, `legacy/bin/mac/libiupimglib.dylib` | IUP, Tecgraf/PUC-Rio | MIT ([LICENSE.IUP](../LICENSE.IUP)) |
| `legacy/bin/mac/libcd.dylib`, `legacy/bin/mac/libcdcontextplus.dylib`, `legacy/bin/mac/libcdgdk.dylib` | CD, Tecgraf/PUC-Rio (`libcdgdk` links the system's GDK, which is not bundled) | MIT |
| `legacy/bin/mac/libfreetype.dylib` | FreeType | FreeType License (FTL), chosen from its FTL / GPL-2.0 dual license |
| `legacy/bin/lin/libimath.so`, `legacy/bin/mac/libimath.dylib`, `legacy/bin/win/imath.dll` | IMath, Michael J. Fromberger | MIT (`legacy/bin/imath.txt`) |
| `legacy/bin/lin/shabal.so` | Shabal hash trial from Reva's author (Forth version: `legacy/scratch/shabal.f`) | presumed public domain |
| `legacy/bin/win/curl.dll` | curl | curl license (MIT-style) |
| `legacy/bin/win/libexpat.dll` | Expat | MIT |
| `legacy/bin/win/pcre.dll` | PCRE | BSD ([LICENSE](../LICENSE) "other components") |
| `legacy/bin/win/pdcurses.dll` | PDCurses | public domain |
| `legacy/bin/win/sqlite3.dll`, `legacy/bin/win/sqlite3.exe` | SQLite | public domain |
| `legacy/bin/win/zlib1.dll` | zlib | zlib |
| `legacy/bin/win/rss_process.dll`, `contrib/rss_process.zip` | rss_process, Reinhold Software Services (sources in the zip) | permissive: "use and distribute it in unmodified or modified form as you want" |
| `legacy/bin/win/glut32.dll` | GLUT for Win32 (Kilgard's GLUT) | GLUT license: redistributable, but not open source. Flagged; freeglut (MIT) replaces it if OpenGL is ever supported |
| `legacy/scratch/mevarchim`, `legacy/scratch/zemanim` | turnkey programs built with Reva | public domain |
| `legacy/scratch/sha1`, `legacy/scratch/sha1b`, `legacy/scratch/sha1c`, `legacy/scratch/sha1transform.o`, `legacy/scratch/sha/sha1`, `legacy/scratch/sha/sha1a`, `legacy/scratch/sha/sha1transform.o`, `legacy/scratch/sha/sha.zip` | SHA-1 trials (symbols match Steve Reid's public-domain SHA-1, not PolarSSL) | public domain |
| `legacy/scratch/sha1hash/upx-dialog.exe` | SHA-1 dialog demo, "axm" 2003 (`legacy/scratch/sha1hash/dialog.asm`) | terms not stated. Flagged; goes with `legacy/` |

## Source files with third-party notices

| Path | Origin | License |
|---|---|---|
| assembler in `src/reva.f`, `lib/asm/`, `lib/util/disasm` | FirmWorks (Open Firmware) | MIT ([LICENSE](../LICENSE)) |
| `lib/ui/iup`, `lib/ui/gui`, `lib/ui/cd`, `lib/ui/im`, `lib/ui/gui-iup-img` | wrappers quoting the Tecgraf license | Reva's terms; the libraries are MIT |
| `lib/ui/gl`, `lib/ui/glu` | wrappers quoting Mesa's header notice | Reva's terms; Mesa headers are MIT |
| `lib/ui/glut` | wrapper quoting freeglut's header notice | Reva's terms; freeglut is MIT |
| `lib/crypt/sha1` | Wil Baden's SHA-1, ported by Ron Aaron | public domain |
| `lib/os/process` and others by Danny Reinhold | Reinhold Software Services | Reva's terms (stated in the files) |
| `legacy/src/brieflz.asm` | BriefLZ, Joergen Ibsen | zlib |

## Crate dependencies

**Portable core (`rust/`):** none. When one is added, `cargo deny check
licenses` must pass, and the crate gets a line here saying why it is needed.

**Desktop app (`desktop/`, its own workspace):** `tauri` (MIT/Apache-2.0)
for the window and webview, `serde`/`serde_json` (MIT/Apache-2.0) for the
package manifest and commands. Their ~320 transitive crates are permissive
except five Servo CSS crates that Tauri's HTML handling uses: `cssparser`,
`cssparser-macros`, `selectors`, `dtoa-short` and `option-ext`, under
**MPL-2.0**. MPL-2.0 is weak, per-file copyleft and not GPL-family: used
unmodified, it asks only that their source stays available (it is, on
crates.io) and that the notice ships with the app. `desktop/deny.toml`
allows MPL-2.0 for exactly these crates, so any other MPL dependency fails
the check until someone decides. On Linux the app links the system's
WebKitGTK (LGPL) dynamically and never bundles it: ship `.deb`/`.rpm`
packages that depend on the distribution's `libwebkit2gtk-4.1-0`, not an
AppImage that would carry it.

The app's icon (`desktop/reva-desktop/icons/icon.png`) is a copy of
`contrib/reva128.png` (Reva, public domain); `icon.ico`, which Windows
builds need, is the same image at 16 to 128 pixels.
