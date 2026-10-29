#!/bin/sh
# make-web-zip.sh: build the browser REPL as a static site and zip it, ready
# to upload to any web host, unzip, and serve as plain files.
#
#   scripts/make-web-zip.sh [options]          (from anywhere; POSIX sh)
#
#   --package DIR   include DIR (a folder with reva-package.json) as pkg/ and
#                   open it by default; its manifest must list its "files"
#   --name NAME     zip and folder name (default: reva, or the package's id)
#   --version V     version in the name (default: the workspace version)
#   --canvas MODE   default graphics placement: split (default), pane, window
#   --out DIR       where the zip goes (default: target/web)
#   --no-build      use the reva.wasm already in components/web/crates/reva-web/www
#
# Result: OUT/NAME-web-V.zip holding one folder NAME-web-V/ (the site) and
# the same tree unzipped at OUT/NAME-web-V/ (what reva-serve embeds).
#
# The site is static and relative: index.html, the JS modules, reva.wasm,
# canvas.html and, with a package, pkg/.  It needs no server logic, no
# special headers (no SharedArrayBuffer, so no COOP/COEP), and works under
# any subpath (https://host/some/dir/NAME-web-V/).  Files written by Forth
# are kept in the visitor's localStorage, which is per origin: a site moved
# to another host or port starts with no saved files.
#
# The zip is reproducible: files are listed in sorted order and stamped
# with the last commit's time (UTC), so the same commit gives the same zip.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/.." && pwd)
WWW=$ROOT/components/web/crates/reva-web/www
PKG= NAME= VERSION= CANVAS=split OUT=$ROOT/target/web BUILD=1

die() { echo "make-web-zip: $*" >&2; exit 1; }
while [ $# -gt 0 ]; do
    case $1 in
        --package) [ $# -ge 2 ] || die "$1 needs a value"; PKG=$2; shift 2 ;;
        --name) [ $# -ge 2 ] || die "$1 needs a value"; NAME=$2; shift 2 ;;
        --version) [ $# -ge 2 ] || die "$1 needs a value"; VERSION=$2; shift 2 ;;
        --canvas) [ $# -ge 2 ] || die "$1 needs a value"; CANVAS=$2; shift 2 ;;
        --out) [ $# -ge 2 ] || die "$1 needs a value"; OUT=$2; shift 2 ;;
        --no-build) BUILD=; shift ;;
        -h|--help) sed -n '2,25p' "$0" | cut -c3-; exit 0 ;;
        *) die "unknown option $1 (try --help)" ;;
    esac
done
case $CANVAS in split|pane|window) ;; *) die "--canvas $CANVAS: use split, pane or window" ;; esac
command -v zip >/dev/null 2>&1 || die "needs zip (macOS has it; Debian/Ubuntu: apt install zip)"

if [ -n "$PKG" ]; then
    [ -f "$PKG/reva-package.json" ] || die "$PKG: no reva-package.json"
    PKG=$(cd "$PKG" && pwd)
    grep -q '"files"' "$PKG/reva-package.json" \
        || die "$PKG/reva-package.json has no \"files\" list (a static server cannot list a folder)"
    if [ -z "$NAME" ]; then
        # the manifest's "id", else the folder name
        NAME=$(tr -d '\n' < "$PKG/reva-package.json" | awk '{ if (match($0, /"id"[ \t]*:[ \t]*"[A-Za-z0-9._-]+"/)) { s = substr($0, RSTART, RLENGTH); sub(/.*"id"[ \t]*:[ \t]*"/, "", s); sub(/"$/, "", s); print s } }')
        [ -n "$NAME" ] || NAME=$(basename "$PKG")
    fi
fi
NAME=${NAME:-reva}
case $NAME in *[!A-Za-z0-9._-]*|'') die "--name $NAME: use letters, digits, . _ -" ;; esac
if [ -z "$VERSION" ]; then
    VERSION=$(awk -F'"' '/^version *=/ { print $2; exit }' "$ROOT/components/vm/Cargo.toml")
fi

[ -n "$BUILD" ] && "$HERE/build-web.sh" >/dev/null
[ -f "$WWW/reva.wasm" ] || die "no $WWW/reva.wasm (run without --no-build)"

SITE=$NAME-web-$VERSION
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
STAGE=$OUT/$SITE
ZIP=$OUT/$SITE.zip
rm -rf "$STAGE" "$ZIP"
mkdir -p "$STAGE"
for f in index.html host.js reva.js reva-worker.js reva.wasm ui.js canvas.html split.js package.js about.js lineedit.js ansi.js sqlite.js favicon.png favicon.ico; do
    cp "$WWW/$f" "$STAGE/$f"
done
cp "$ROOT/LICENSE" "$STAGE/LICENSE.txt"
cp "$ROOT/LICENSE-MIT" "$STAGE/LICENSE-MIT.txt"

# The page's defaults and the build stamp (About), as meta tags (index.html reads them when the URL
# has no ?package= or ?canvas=).
pkgmeta=
if [ -n "$PKG" ]; then
    mkdir -p "$STAGE/pkg"
    # every file of the package except dot files (.git and friends)
    (cd "$PKG" && find . -type f ! -path '*/.*' | while IFS= read -r f; do
        mkdir -p "$STAGE/pkg/$(dirname "$f")"
        cp "$f" "$STAGE/pkg/$f"
    done)
    pkgmeta=pkg/
fi
build=$("$HERE/build-stamp.sh")
awk -v pkg="$pkgmeta" -v canvas="$CANVAS" -v build="$build" '
    index($0, "<meta name=\"reva-package\" content=\"\">") { sub(/content=""/, "content=\"" pkg "\"") }
    index($0, "<meta name=\"reva-canvas\" content=\"\">") { sub(/content=""/, "content=\"" canvas "\"") }
    index($0, "<meta name=\"reva-build\" content=\"\">") { sub(/content=""/, "content=\"" build "\"") }
    { print }' "$WWW/index.html" > "$STAGE/index.html"
grep -q "name=\"reva-canvas\" content=\"$CANVAS\"" "$STAGE/index.html" \
    || die "index.html has no reva-canvas meta tag to set"

cat > "$STAGE/README.txt" <<EOF
$SITE: Reva Forth in the browser (WebAssembly), as a static site.

Serve this folder with any static web server and open index.html:

  - on a web host: upload the zip, unzip it anywhere (a subfolder is fine),
    and open https://your.host/path/$SITE/
  - locally: python3 -m http.server -d $SITE 8000, then http://localhost:8000/
    (or run the reva-serve binary, which embeds the same files)

Opening index.html as a file:// URL does not work: browsers do not run
Web Workers or fetch files from file:// pages.

Files you write in Forth are kept in your browser's localStorage for this
site's origin (scheme, host and port): another host or port starts empty.

URL options: ?canvas=split|pane|window, ?package=DIR/ (another package
folder on the same site), ?include=FILE, ?eval=TEXT, ?image=FILE.

Licenses: LICENSE.txt (Reva, public domain), LICENSE-MIT.txt (the portable
core).$( [ -n "$PKG" ] && printf '\nThe package in pkg/ has its own terms; see its files.' )
EOF

# Reproducible: fixed timestamps (the last commit's, in UTC) and sorted names.
stamp=$(cd "$ROOT" && TZ=UTC git log -1 --format=%cd --date=format-local:%Y%m%d%H%M 2>/dev/null || true)
stamp=${stamp:-202601010000}
find "$STAGE" -exec env TZ=UTC touch -t "$stamp" {} +
(cd "$OUT" && find "$SITE" | LC_ALL=C sort | TZ=UTC zip -qX -@ "$ZIP")

n=$(find "$STAGE" -type f | wc -l | tr -d ' ')
size=$(wc -c < "$ZIP" | tr -d ' ')
echo "make-web-zip: $ZIP ($n files, $size bytes)"
echo "make-web-zip: site at $STAGE (try: python3 -m http.server -d $STAGE 8000)"
