#!/usr/bin/env bash
# License gate: keep copyleft code and unreviewed binaries out of the tree.
#
#   scripts/license-check.sh          (checks the whole repository)
#
# 1. No tracked file may carry GPL, LGPL or AGPL license text.
# 2. Every tracked binary (ELF, PE, Mach-O, object file, archive) must be on
#    the reviewed list below, with its license.  Each listed path must also
#    appear in docs/licenses.md, so the inventory and the gate cannot drift.
# 3. If cargo-deny is installed, `cargo deny check licenses` must pass in
#    each Cargo workspace, components/*/ (deny.toml holds the list).  Without
#    it, a note is printed.
#
# Only git-tracked files are checked, so build output never trips the gate.
# Exit 1 on any failure.  Works with the bash 3.2 that macOS ships.
set -euo pipefail
cd "$(dirname "$0")/.."
status=0
fail() { echo "license-check: $*"; status=1; }

# Reviewed binaries: path, then license (see docs/licenses.md for the source
# of each judgement).  Adding a binary means reviewing it and adding it here.
# Read with `read -d ''`, not `$(cat <<EOF ...)`: bash 3.2 (macOS /bin/bash)
# mis-parses a here-document inside `$( )` when the text has an apostrophe.
read -r -d '' reviewed <<'EOF' || true
legacy/bin/lin/build                    Reva (public domain)
legacy/bin/lin/core                     Reva (public domain)
legacy/bin/lin/reva                     Reva (public domain)
legacy/bin/lin/libimath.so              IMath (MIT)
legacy/bin/lin/libiupall.so             IUP (MIT)
legacy/bin/lin/shabal.so                Reva scratch, Ron Aaron (presumed public domain)
legacy/bin/mac/build                    Reva (public domain), UPX-packed
legacy/bin/mac/core                     Reva (public domain), UPX-packed
legacy/bin/mac/reva                     Reva (public domain), UPX-packed
legacy/bin/mac/libcd.dylib              CD (MIT)
legacy/bin/mac/libcdcontextplus.dylib   CD (MIT)
legacy/bin/mac/libcdgdk.dylib           CD (MIT); links the system GDK
legacy/bin/mac/libfreetype.dylib        FreeType (FTL, chosen from FTL/GPL-2.0)
legacy/bin/mac/libimath.dylib           IMath (MIT)
legacy/bin/mac/libiup.dylib             IUP (MIT)
legacy/bin/mac/libiupcd.dylib           IUP (MIT)
legacy/bin/mac/libiupcontrols.dylib     IUP (MIT)
legacy/bin/mac/libiupgl.dylib           IUP (MIT)
legacy/bin/mac/libiupim.dylib           IUP (MIT)
legacy/bin/mac/libiupimglib.dylib       IUP (MIT)
legacy/bin/win/build.exe                Reva (public domain)
legacy/bin/win/core.exe                 Reva (public domain)
legacy/bin/win/reva.exe                 Reva (public domain)
legacy/bin/win/curl.dll                 curl (curl license, MIT-style)
legacy/bin/win/glut32.dll               GLUT for Win32 (GLUT license: redistributable, not open source)
legacy/bin/win/imath.dll                IMath (MIT)
legacy/bin/win/iupall.dll               IUP (MIT)
legacy/bin/win/libexpat.dll             Expat (MIT)
legacy/bin/win/pcre.dll                 PCRE (BSD)
legacy/bin/win/pdcurses.dll             PDCurses (public domain)
legacy/bin/win/rss_process.dll          rss_process, Reinhold Software (permissive, see contrib/rss_process.zip)
legacy/bin/win/sqlite3.dll              SQLite (public domain)
legacy/bin/win/sqlite3.exe              SQLite (public domain)
legacy/bin/win/zlib1.dll                zlib (zlib)
legacy/src/revares.o                    Reva Windows resources (public domain)
legacy/scratch/mevarchim                Reva turnkey program (public domain)
legacy/scratch/zemanim                  Reva turnkey program (public domain)
legacy/scratch/sha1                     SHA-1 trial, Steve Reid's code (public domain)
legacy/scratch/sha1b                    SHA-1 trial, Steve Reid's code (public domain)
legacy/scratch/sha1c                    SHA-1 trial, Steve Reid's code (public domain)
legacy/scratch/sha1transform.o          SHA-1 trial (public domain)
legacy/scratch/sha/sha1                 SHA-1 trial, Steve Reid's code (public domain)
legacy/scratch/sha/sha1a                SHA-1 trial, Steve Reid's code (public domain)
legacy/scratch/sha/sha1transform.o      SHA-1 trial (public domain)
legacy/scratch/sha/sha.zip              archive of legacy/scratch/sha (public domain)
legacy/scratch/sha1hash/upx-dialog.exe  SHA-1 dialog demo, axm 2003 (terms unknown; UPX-packed)
contrib/rss_process.zip                 rss_process sources, Reinhold Software (permissive)
EOF
is_reviewed() { echo "$reviewed" | awk '{print $1}' | grep -qxF -- "$1"; }

# Binary by content (magic numbers) or by extension, so a renamed file is caught.
is_binary() {
    case "$1" in
        *.dll|*.so|*.so.*|*.dylib|*.exe|*.o|*.a|*.lib|*.zip|*.gz|*.tgz|*.tar|*.bz2|*.xz|*.7z|*.jar) return 0 ;;
    esac
    local magic
    magic=$(head -c4 -- "$1" 2>/dev/null | od -An -tx1 | tr -d ' \n')
    case "$magic" in
        7f454c46|4d5a*|cffaedfe|cefaedfe|feedface|feedfacf|cafebabe|504b0304) return 0 ;;
    esac
    return 1
}

# This script names the pattern without being under it, so it is exempt;
# so are agentrail's session transcripts, which quote files (this one too).
copyleft='GNU (Lesser |Library |Affero )?General Public License|SPDX-License-Identifier:.*(GPL|AGPL)'
hits=$(git ls-files -z -- . ':!scripts/license-check.sh' ':!.agentrail/sessions' \
       | xargs -0 grep -l -a -E "$copyleft" 2>/dev/null || true)
for f in $hits; do fail "$f: GPL-family license text"; done

nbin=0
while IFS= read -r f; do
    [[ -f "$f" && ! -L "$f" ]] || continue
    is_binary "$f" || continue
    nbin=$((nbin+1))
    is_reviewed "$f" || fail "$f: binary not on the reviewed list (review it, then add it to this script and docs/licenses.md)"
done < <(git ls-files)

for f in $(echo "$reviewed" | awk '{print $1}'); do
    [[ -e "$f" ]] || fail "$f: on the reviewed list but not in the tree (remove it from the list)"
    grep -qF -- "\`$f\`" docs/licenses.md || fail "$f: missing from docs/licenses.md"
done

if command -v cargo-deny >/dev/null 2>&1; then
    for ws in components/*/; do
        ws=${ws%/}
        (cd "$ws" && cargo deny --log-level error check --config ../../deny.toml licenses >/dev/null) \
            || fail "$ws: cargo deny check licenses failed (run it in $ws/ for details)"
    done
    crates="crates checked by cargo-deny in every component"
else
    crates="cargo-deny not installed: crate licenses not checked (cargo install cargo-deny)"
fi

if [[ $status -eq 0 ]]; then
    echo "license-check: ok ($nbin reviewed binaries, no GPL-family text; $crates)"
fi
exit $status
