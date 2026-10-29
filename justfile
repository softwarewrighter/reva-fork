# justfile: common builds, as `just RECIPE` (https://just.systems).
# Every recipe is a thin wrapper over a script in scripts/ or components/desktop/,
# so nothing here needs just: the script lines below work as they are.
#
#   just              list the recipes
#   just web-zip      the static site zip (target/web/reva-web-VERSION.zip)

# list the recipes
default:
    @just --list

# the WebAssembly VM (components/web/crates/reva-web/www/reva.wasm)
wasm:
    scripts/build-web.sh

# the browser REPL as an uploadable static zip; extra args go to make-web-zip.sh (--package DIR, --canvas MODE, ...)
web-zip *ARGS:
    scripts/make-web-zip.sh {{ARGS}}

# publish the live demo (plain reva-fork, from main) to gh-pages; --dry-run builds without pushing
publish *ARGS:
    scripts/publish-pages.sh {{ARGS}}

# serve the browser REPL from the source tree at http://localhost:8000/?canvas=split
web-dev: wasm
    python3 -m http.server -d components/web/crates/reva-web/www 8000

# reva-serve, the one-binary local web app, for this machine (or TARGETs: mac, linux, windows); args go to build-serve.sh
serve-build *ARGS:
    scripts/build-serve.sh {{ARGS}}

# build reva-serve for this machine from the source tree and run it (opens the browser; Ctrl-C stops)
serve:
    cargo run --release --manifest-path components/serve/Cargo.toml

# the CLI (target/release/reva)
cli:
    cargo build --release --manifest-path components/cli/Cargo.toml

# the desktop app (target/release/reva-desktop)
desktop:
    components/desktop/build.sh

# the gate before every commit: fmt, clippy, tests, goldens, licenses, sw-checklist (--fast skips the desktop)
check *ARGS:
    scripts/check.sh {{ARGS}}

# timing guard for refactors of hot code: medians vs this host's baseline (--record to set it)
bench-guard *ARGS:
    scripts/bench-guard.sh {{ARGS}}

# tests of the portable core and the browser build
test: wasm
    for c in x86 host core vlib words-base words-interp words-dict words-os words vm sqlite cli web serve; do (cd components/$c && cargo test --release --workspace) || exit 1; done
    node components/web/crates/reva-web/test/smoke.mjs
    node components/web/crates/reva-web/test/lineedit.mjs
    node components/web/crates/reva-web/test/ansi.mjs
