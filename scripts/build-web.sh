#!/usr/bin/env bash
# Build the WebAssembly module into components/web/crates/reva-web/www/ (serve that directory with
# any static file server, e.g. `python3 -m http.server -d components/web/crates/reva-web/www`).
set -euo pipefail
cd "$(dirname "$0")/.."
rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown \
    || rustup target add wasm32-unknown-unknown
cargo build -q --release --target wasm32-unknown-unknown --manifest-path components/web/Cargo.toml
cp target/wasm32-unknown-unknown/release/reva_web.wasm components/web/crates/reva-web/www/reva.wasm
echo "components/web/crates/reva-web/www/reva.wasm: $(wc -c < components/web/crates/reva-web/www/reva.wasm) bytes"
