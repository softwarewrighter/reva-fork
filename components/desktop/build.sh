#!/usr/bin/env bash
# Build the Reva desktop app.
#
#   components/desktop/build.sh            build target/release/reva-desktop
#   components/desktop/build.sh --install  ... and install it as `reva-desktop` (cargo install)
#   components/desktop/build.sh --bundle   ... and make installers (.deb/.rpm, .app/.dmg; needs tauri-cli)
#
# Steps: build the WebAssembly VM (scripts/build-web.sh), assemble the
# page into components/desktop/dist (the browser page, with components/desktop/host.js in place of
# the browser's), then compile the native shell, which embeds dist.
# Linux needs WebKitGTK 4.1 headers (Debian/Ubuntu: libwebkit2gtk-4.1-dev);
# macOS needs only Xcode's command line tools.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
"$here/../../scripts/build-web.sh" >/dev/null
www="$here/../web/crates/reva-web/www"
rm -rf "$here/dist" && mkdir -p "$here/dist"
cp "$www/index.html" "$www/reva.js" "$www/reva-worker.js" "$www/reva.wasm" \
   "$www/ui.js" "$www/canvas.html" "$www/split.js" "$www/package.js" "$www/about.js" "$www/lineedit.js" "$www/ansi.js" "$www/sqlite.js" "$www/favicon.png" "$here/dist/"
# the build stamp for the About dialog (reva-build meta tag)
stamp=$("$here/../../scripts/build-stamp.sh")
awk -v build="$stamp" '
    index($0, "<meta name=\"reva-build\" content=\"\">") { sub(/content=""/, "content=\"" build "\"") }
    { print }' "$www/index.html" > "$here/dist/index.html"
cp "$here/host.js" "$here/dist/host.js"
cd "$here"
case "${1:-}" in
    --install) cargo install --locked --path crates/reva-desktop ;;
    --bundle)  (cd crates/reva-desktop && cargo tauri build) ;;
    *)         cargo build --release ;;
esac
[[ "${1:-}" == --install ]] || echo "built: $(cd "$here/../.." && pwd)/target/release/reva-desktop"
