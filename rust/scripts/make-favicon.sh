#!/bin/sh
# Make the browser page's favicon from the desktop app's icon (the blue Reva
# gear), so the web REPL, reva-serve and the desktop app look the same.
#
#   rust/scripts/make-favicon.sh     (needs ImageMagick `convert`)
#
# Writes into rust/reva-web/www/ (both committed; rerun when the icon changes):
#   favicon.png   64x64, the page's <link rel="icon"> (a package's icon replaces it)
#   favicon.ico   16, 32, 48: what browsers fetch from /favicon.ico on their own
set -eu
cd "$(dirname "$0")/.."
src=../desktop/reva-desktop/icons/icon.png
www=reva-web/www
convert "$src" -filter Lanczos -resize 64x64 -strip "PNG32:$www/favicon.png"
convert "$src" -filter Lanczos \
    \( -clone 0 -resize 16x16 \) \( -clone 0 -resize 32x32 \) \( -clone 0 -resize 48x48 \) \
    -delete 0 "$www/favicon.ico"
echo "make-favicon: $www/favicon.png $www/favicon.ico"
