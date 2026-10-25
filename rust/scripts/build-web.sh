#!/usr/bin/env bash
# Build the WebAssembly module into reva-web/www/ (serve that directory with
# any static file server, e.g. `python3 -m http.server -d reva-web/www`).
set -euo pipefail
cd "$(dirname "$0")/.."
rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown \
    || rustup target add wasm32-unknown-unknown
cargo build -q --release --target wasm32-unknown-unknown -p reva-web
cp target/wasm32-unknown-unknown/release/reva_web.wasm reva-web/www/reva.wasm
echo "reva-web/www/reva.wasm: $(wc -c < reva-web/www/reva.wasm) bytes"
