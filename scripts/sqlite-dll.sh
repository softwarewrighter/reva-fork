#!/usr/bin/env bash
# SQLite for the Windows builds: the official sqlite3.dll from sqlite.org,
# checked against its published SHA3-256, plus the import library
# (sqlite3.lib) that MSVC links against, made from the zip's sqlite3.def
# with MSVC's lib.exe.  Nothing is compiled.  macOS and Linux link the
# system's SQLite instead (docs/sqlite.md), so this is for Windows only.
#
#   scripts/sqlite-dll.sh [x64|arm64]     (Git Bash; default x64)
#
# Leaves target/sqlite/{sqlite3.dll,sqlite3.def,sqlite3.lib}.  Then build
# with SQLITE3_LIB_DIR set to that folder and the folder on PATH (tests
# and programs load the DLL from there); the installers ship the DLL next
# to the program.  Run it again after changing the version below.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=3.54.1 YEAR=2026 NUM=3540100
arch=${1:-x64}
case $arch in
    x64)   sha3=c2d09c8babb9399277055c057c97a53f1ffd7c5f11d0c1f2118ffdfa39f2daa4 ;;
    arm64) sha3=58a563f64ba4c5df34c3ce667175678e2c1df503b264cd600200eb37aa334201 ;;
    *) echo "sqlite-dll: unknown architecture $arch (x64 or arm64)" >&2; exit 2 ;;
esac
out=target/sqlite
zip=sqlite-dll-win-$arch-$NUM.zip

if [[ -f $out/sqlite3.lib && -f $out/sqlite3.dll && "$(cat "$out/version" 2>/dev/null)" == "$VERSION $arch" ]]; then
    echo "sqlite-dll: $out has SQLite $VERSION ($arch)"
    exit 0
fi
rm -rf "$out" && mkdir -p "$out"
curl -fsSL -o "$out/$zip" "https://sqlite.org/$YEAR/$zip"
got=$(openssl dgst -sha3-256 -r "$out/$zip" | awk '{print $1}')
[[ "$got" == "$sha3" ]] || { echo "sqlite-dll: $zip: SHA3-256 $got, expected $sha3" >&2; exit 1; }
# Windows' own tar reads zip files (Git Bash's GNU tar does not)
if command -v unzip >/dev/null; then
    unzip -q -o "$out/$zip" -d "$out"
else
    (cd "$out" && "${SYSTEMROOT:-C:/Windows}/System32/tar.exe" -xf "$zip")
fi
rm -f "$out/$zip"

# lib.exe: on PATH in a Developer shell, else found with vswhere
lib=$(command -v lib.exe || true)
if [[ -z "$lib" ]]; then
    vswhere="${PROGRAMFILES:-C:/Program Files} (x86)/Microsoft Visual Studio/Installer/vswhere.exe"
    [[ -x "$vswhere" ]] || vswhere="C:/Program Files (x86)/Microsoft Visual Studio/Installer/vswhere.exe"
    host=$([[ "${PROCESSOR_ARCHITECTURE:-}" == ARM64 ]] && echo Hostarm64 || echo Hostx64)
    lib=$("$vswhere" -latest -products '*' -find "VC/Tools/MSVC/*/bin/$host/$arch/lib.exe" | tr -d '\r' | head -1)
fi
[[ -n "$lib" ]] || { echo "sqlite-dll: lib.exe not found (install the Visual Studio Build Tools)" >&2; exit 1; }
(cd "$out" && MSYS2_ARG_CONV_EXCL='*' "$lib" -nologo -def:sqlite3.def -out:sqlite3.lib -machine:"$arch")
echo "$VERSION $arch" > "$out/version"
echo "sqlite-dll: $out has SQLite $VERSION ($arch): set SQLITE3_LIB_DIR=$PWD/$out and put it on PATH"
