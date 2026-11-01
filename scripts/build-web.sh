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

# The TUI of IUP programs (tui.js): its own module, built with wasm-bindgen
# (Yew), so reva.wasm keeps no imports.  The wasm-bindgen CLI must match
# the crate's pinned version (components/ui-web).
want=$(awk -F'"' '/^wasm-bindgen = /{print $2}' components/ui-web/crates/reva-ui-web/Cargo.toml | tr -d '=')
have=$(wasm-bindgen --version 2>/dev/null | awk '{print $2}')
[ "$have" = "$want" ] || { echo "build-web: wasm-bindgen $want is needed, found ${have:-none} (cargo install wasm-bindgen-cli --version $want)" >&2; exit 1; }
cargo build -q --release --target wasm32-unknown-unknown --manifest-path components/ui-web/Cargo.toml
wasm-bindgen --target web --no-typescript --out-dir components/web/crates/reva-web/www/ui \
    target/wasm32-unknown-unknown/release/reva_ui_web.wasm
echo "components/web/crates/reva-web/www/ui/reva_ui_web_bg.wasm: $(wc -c < components/web/crates/reva-web/www/ui/reva_ui_web_bg.wasm) bytes"
