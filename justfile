# justfile: common builds, as `just RECIPE` (https://just.systems).
# Every recipe is a thin wrapper over a script in scripts/ or components/desktop/,
# so nothing here needs just: the script lines below work as they are.
#
#   just              list the recipes
#   just web-zip      the static site zip (target/web/reva-web-VERSION.zip)
#
# Arguments reach the programs and scripts as given, quoted (`just cli -e
# "6 7 * ."`): positional arguments, not split at spaces.

set positional-arguments

# list the recipes
default:
    @just --list

# the WebAssembly VM (components/web/crates/reva-web/www/reva.wasm)
wasm:
    scripts/build-web.sh

# the browser REPL as an uploadable static zip; extra args go to make-web-zip.sh (--package DIR, --canvas MODE, ...)
web-zip *ARGS:
    scripts/make-web-zip.sh "$@"

# publish the live demo (plain reva-fork, from main) to gh-pages; --dry-run builds without pushing
publish *ARGS:
    scripts/publish-pages.sh "$@"

# serve the browser REPL from the source tree at http://localhost:8000/?canvas=split (node, no python)
web-dev: wasm
    node scripts/serve-dir.mjs components/web/crates/reva-web/www 8000

# reva-serve, the one-binary local web app, for this machine (or TARGETs: mac, linux, windows); args go to build-serve.sh
serve-build *ARGS:
    scripts/build-serve.sh "$@"

# build the web page and reva-serve for this machine and run it (opens the browser; Ctrl-C stops); args go to reva-serve
serve *ARGS: wasm
    cargo run -q --release --manifest-path components/serve/Cargo.toml -- "$@"

# build the CLI and run it: the REPL, or `just cli FILE`, `just cli -e TEXT`, `just cli --version`
cli *ARGS: cli-build
    target/release/reva "$@"

# build the CLI only (target/release/reva)
cli-build:
    cargo build -q --release --manifest-path components/cli/Cargo.toml

# build the desktop app and run it (warning if an older reva-desktop is on PATH); args go to the app
desktop *ARGS: desktop-build
    @scripts/release-check.sh --path-only
    target/release/reva-desktop "$@"

# build the desktop app only (target/release/reva-desktop)
desktop-build:
    components/desktop/build.sh

# build the desktop app and install it as reva-desktop on PATH, replacing an older copy
desktop-install:
    components/desktop/build.sh --install

# every deliverable (CLI, web, reva-serve, desktop) built through these recipes shows VERSION and this commit
release-check *ARGS:
    scripts/release-check.sh "$@"

# the gate before every commit: fmt, clippy, tests, goldens, licenses, sw-checklist (--fast skips the desktop)
check *ARGS:
    scripts/check.sh "$@"

# timing guard for refactors of hot code: medians vs this host's baseline (--record to set it)
bench-guard *ARGS:
    scripts/bench-guard.sh "$@"

# tests of the portable core and the browser build
test: wasm
    for c in macros x86-types x86-decode x86-alu x86-exec x86 host-base vfs host core-parts canvas cd ui ui-layout ui-grid ui-web core core-run iup vlib words-base words-interp words-dict words-os words vm sqlite cli-edit cli-host cli web serve; do (cd components/$c && cargo test --release --workspace) || exit 1; done
    node components/web/crates/reva-web/test/smoke.mjs
    node components/web/crates/reva-web/test/lineedit.mjs
    node components/web/crates/reva-web/test/ansi.mjs
