#!/usr/bin/env bash
# release-check.sh: every deliverable built from this checkout shows the
# release version (VERSION) and this commit, where a user sees them: the
# CLI's `--version`, the web page's VM (banner and footer), reva-serve and
# the page inside it, and the desktop app (its version, and the page and VM
# built into it, as `reva-desktop --about` reports them). A release waits
# until this passes; the gate runs it.
#
#   scripts/release-check.sh                 build through the just recipes, then check
#   scripts/release-check.sh --no-desktop    the same without the desktop app (just check --fast)
#   scripts/release-check.sh --no-build      check what is built, building nothing
#
# Copies on PATH (reva, reva-desktop, reva-serve) of another version are
# reported as warnings: they are what a user runs, but not part of a build.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

build=1 desktop=1 path_only=0
for a in "$@"; do
    case "$a" in
        --no-desktop) desktop=0 ;;
        --no-build) build=0 ;;
        --path-only) path_only=1 ;;
        *) echo "release-check: unknown option $a" >&2; exit 2 ;;
    esac
done

version=$(tr -d ' \n' < VERSION)
head=$(git rev-parse --short HEAD)
failed=0

# what a user runs from PATH, if it is another version (a warning)
path_copies() {
    local b p v
    for b in reva reva-desktop reva-serve; do
        p=$(command -v "$b" 2>/dev/null) || continue
        case "$p" in "$ROOT"/*) continue ;; esac
        v=$("$p" --version 2>/dev/null | head -1 || true)
        [[ "$v" == *" $version" ]] || echo "warn  $b on PATH ($p) says '$v', not $version"
    done
}
if [[ $path_only == 1 ]]; then path_copies; exit 0; fi
ok() { echo "ok    $1"; }
bad() { echo "FAIL  $1"; failed=1; }
# has FILE TEXT: FILE holds TEXT (bytes, not lines)
has() { LC_ALL=C grep -a -q -F -- "$2" "$1"; }
# run LABEL CMD...: CMD's stdout, through just when building (the recipes
# are what a user runs), else from target/release
run() { if [[ $build == 1 ]]; then just "$@"; else "$ROOT/target/release/$1" "${@:2}"; fi; }

# the CLI: `reva --version` names the release and this commit
out=$(run cli --version 2>/dev/null || true)
[[ "$(printf '%s\n' "$out" | head -1)" == "reva-fork $version" ]] \
    && ok "cli: reva --version is reva-fork $version" \
    || bad "cli: reva --version says '$(printf '%s\n' "$out" | head -1)', not reva-fork $version"
printf '%s\n' "$out" | grep -q "Commit: $head\$" \
    && ok "cli: built from $head" \
    || bad "cli: not built from $head ($(printf '%s\n' "$out" | grep Commit: || echo no commit))"

# the web page: its VM's banner and build (the footer)
[[ $build == 1 ]] && just wasm > /dev/null
wasm=components/web/crates/reva-web/www/reva.wasm
{ has "$wasm" "reva-fork $version portable" && has "$wasm" "$head"; } \
    && ok "web: reva.wasm is $version, built from $head" \
    || bad "web: $wasm is not $version from $head (scripts/build-web.sh)"

# reva-serve: its version, and the page inside it
out=$(run serve --version 2>/dev/null | tail -1 || true)
[[ "$out" == "reva-serve reva $version" ]] \
    && ok "serve: reva-serve --version is $version" \
    || bad "serve: reva-serve --version says '$out', not reva-serve reva $version"
{ has target/release/reva-serve "reva-fork $version portable" && has target/release/reva-serve "$head"; } \
    && ok "serve: the page inside is $version, built from $head" \
    || bad "serve: the page inside reva-serve is not $version from $head"

# the desktop app: its version, and the page and VM built into it
if [[ $desktop == 1 ]]; then
    out=$(run desktop --about 2>/dev/null || true)
    line() { printf '%s\n' "$out" | awk -v k="$1: " 'index($0, k) == 1 { print substr($0, length(k) + 1) }'; }
    [[ "$(line app)" == "reva-desktop $version" ]] \
        && ok "desktop: the app is $version" \
        || bad "desktop: the app says '$(line app)', not reva-desktop $version"
    page=$(line page)
    [[ "$page" == "$version $head "* || "$page" == "$version $head-dirty "* ]] \
        && ok "desktop: its page (About) is $version, built from $head" \
        || bad "desktop: its page says '$page', not $version from $head (components/desktop/build.sh)"
    [[ "$(line vm)" == "reva-fork $version portable" && "$(line 'vm build')" == "$head (in reva.wasm)" ]] \
        && ok "desktop: its VM (banner, footer) is $version, built from $head" \
        || bad "desktop: its VM says '$(line vm)', '$(line 'vm build')'"
fi

path_copies

[[ $failed == 0 ]] && echo "release-check: every deliverable is $version from $head"
exit $failed
