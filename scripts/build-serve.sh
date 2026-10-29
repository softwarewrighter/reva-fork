#!/bin/sh
# build-serve.sh: build reva-serve, the one-binary local web app, for
# release: the static site (make-web-zip.sh) embedded in a small server.
#
#   scripts/build-serve.sh [options] [TARGET ...]        (POSIX sh)
#
#   --site DIR      embed DIR (a site tree, e.g. from make-web-zip.sh)
#                   instead of building one
#   --package DIR   build the site with this package (make-web-zip.sh --package)
#   --name NAME     app name: binary NAME-serve (default: reva, or the package id)
#   --port N        preferred port (default 8421)
#   --out DIR       where binaries go (default: target/serve)
#   --no-build      do not rebuild reva.wasm
#
# TARGET is a Rust target triple, or one of these shorthands:
#   mac      universal macOS binary (arm64 + x86_64, joined with lipo; macOS only)
#   linux    static Linux binary for this machine's CPU (x86_64/aarch64 musl)
#   windows  x86_64-pc-windows-gnu (needs mingw-w64 when not on Windows)
# Default: mac on macOS, linux on Linux, the host triple elsewhere.
#
# Output: OUT/NAME-serve-VERSION-TARGET[.exe], plus a .sha256 beside each.
# Targets are added with `rustup target add` as needed.  The binary has no
# crate dependencies (std only), so any target rustup has will do.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/.." && pwd)
SITE= PKG= NAME= PORT=8421 OUT=$ROOT/target/serve NOBUILD=
die() { echo "build-serve: $*" >&2; exit 1; }
while [ $# -gt 0 ]; do
    case $1 in
        --site) [ $# -ge 2 ] || die "$1 needs a value"; SITE=$2; shift 2 ;;
        --package) [ $# -ge 2 ] || die "$1 needs a value"; PKG=$2; shift 2 ;;
        --name) [ $# -ge 2 ] || die "$1 needs a value"; NAME=$2; shift 2 ;;
        --port) [ $# -ge 2 ] || die "$1 needs a value"; PORT=$2; shift 2 ;;
        --out) [ $# -ge 2 ] || die "$1 needs a value"; OUT=$2; shift 2 ;;
        --no-build) NOBUILD=--no-build; shift ;;
        -h|--help) sed -n '2,24p' "$0" | cut -c3-; exit 0 ;;
        -*) die "unknown option $1 (try --help)" ;;
        *) break ;;
    esac
done
VERSION=$(awk -F'"' '/^version *=/ { print $2; exit }' "$ROOT/components/vm/Cargo.toml")

# The site: given, or built the way the uploadable zip is.
if [ -z "$SITE" ]; then
    zipargs="--out $ROOT/target/web $NOBUILD"
    [ -n "$PKG" ] && zipargs="$zipargs --package $PKG"
    [ -n "$NAME" ] && zipargs="$zipargs --name $NAME"
    # shellcheck disable=SC2086 # word splitting is wanted (paths without spaces)
    line=$("$HERE/make-web-zip.sh" $zipargs | head -1)
    zip=${line#make-web-zip: }
    SITE=${zip%%.zip *}
    # pick up the name make-web-zip chose (the package id)
    base=$(basename "$SITE")
    NAME=${NAME:-${base%-web-*}}
fi
[ -f "$SITE/index.html" ] || die "$SITE: no index.html"
SITE=$(cd "$SITE" && pwd)
NAME=${NAME:-reva}

if [ $# -eq 0 ]; then
    case $(uname -s) in
        Darwin) set -- mac ;;
        Linux) set -- linux ;;
        *) set -- "$(rustc -vV | awk '/^host:/ { print $2 }')" ;;
    esac
fi
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)

# build TRIPLE: cargo build of reva-serve for TRIPLE; prints the binary path.
build() {
    rustup target list --installed 2>/dev/null | grep -qx "$1" || rustup target add "$1" >&2
    linker=
    case $1 in
        x86_64-pc-windows-gnu)
            [ "$(uname -s)" = Linux ] && linker=x86_64-w64-mingw32-gcc ;;
    esac
    (cd "$ROOT" && env REVA_SERVE_SITE="$SITE" REVA_SERVE_NAME="$NAME" REVA_SERVE_PORT="$PORT" \
        ${linker:+CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=$linker} \
        cargo build -q --release --manifest-path components/serve/Cargo.toml --target "$1" >&2)
    exe=
    case $1 in *windows*) exe=.exe ;; esac
    echo "$ROOT/target/$1/release/reva-serve$exe"
}

sha() {
    if command -v sha256sum >/dev/null 2>&1; then (cd "$(dirname "$1")" && sha256sum "$(basename "$1")")
    else (cd "$(dirname "$1")" && shasum -a 256 "$(basename "$1")"); fi
}

for t in "$@"; do
    case $t in
        mac)
            [ "$(uname -s)" = Darwin ] || die "mac: the universal binary needs macOS (lipo and Apple's linker)"
            a=$(build aarch64-apple-darwin)
            x=$(build x86_64-apple-darwin)
            dest=$OUT/$NAME-serve-$VERSION-macos-universal
            lipo -create -output "$dest" "$a" "$x"
            ;;
        linux)
            case $(uname -m) in
                arm64|aarch64) t=aarch64-unknown-linux-musl ;;
                *) t=x86_64-unknown-linux-musl ;;
            esac
            dest=$OUT/$NAME-serve-$VERSION-$t
            cp "$(build "$t")" "$dest"
            ;;
        windows)
            dest=$OUT/$NAME-serve-$VERSION-x86_64-pc-windows-gnu.exe
            cp "$(build x86_64-pc-windows-gnu)" "$dest"
            ;;
        *)
            exe=
            case $t in *windows*) exe=.exe ;; esac
            dest=$OUT/$NAME-serve-$VERSION-$t$exe
            cp "$(build "$t")" "$dest"
            ;;
    esac
    chmod +x "$dest"
    sha "$dest" > "$dest.sha256"
    echo "build-serve: $dest ($(wc -c < "$dest" | tr -d ' ') bytes)"
done
