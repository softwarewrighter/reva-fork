# justfile: common builds, as `just RECIPE` (https://just.systems).
# Every recipe is a thin wrapper over a script in rust/scripts/ or desktop/,
# so nothing here needs just: the script lines below work as they are.
#
#   just              list the recipes
#   just web-zip      the static site zip (rust/target/web/reva-web-VERSION.zip)

# list the recipes
default:
    @just --list

# the WebAssembly VM (rust/reva-web/www/reva.wasm)
wasm:
    rust/scripts/build-web.sh

# the browser REPL as an uploadable static zip; extra args go to make-web-zip.sh (--package DIR, --canvas MODE, ...)
web-zip *ARGS:
    rust/scripts/make-web-zip.sh {{ARGS}}

# serve the browser REPL from the source tree at http://localhost:8000/?canvas=split
web-dev: wasm
    python3 -m http.server -d rust/reva-web/www 8000

# reva-serve, the one-binary local web app, for this machine (or TARGETs: mac, linux, windows); args go to build-serve.sh
serve-build *ARGS:
    rust/scripts/build-serve.sh {{ARGS}}

# build reva-serve for this machine from the source tree and run it (opens the browser; Ctrl-C stops)
serve:
    cd rust && cargo run --release -p reva-serve

# the CLI (rust/target/release/reva)
cli:
    cd rust && cargo build --release -p reva-cli

# the desktop app (desktop/target/release/reva-desktop)
desktop:
    desktop/build.sh

# tests of the portable core and the browser build
test: wasm
    cd rust && cargo test --release --workspace
    node rust/reva-web/test/smoke.mjs
