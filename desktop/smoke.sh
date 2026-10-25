#!/usr/bin/env bash
# Smoke-test the desktop app without a person: start it with a private
# folder and a throwaway package, check what it prints and what it wrote.
#
#   desktop/smoke.sh        (after desktop/build.sh; on Linux it uses xvfb-run
#                            when there is no display)
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
app="${REVA_DESKTOP:-$here/target/release/reva-desktop}"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
run() {
    if [[ -z "${DISPLAY:-}" && "$(uname)" == Linux ]]; then
        timeout 60 xvfb-run -a "$app" "$@" 2>/dev/null
    else
        timeout 60 "$app" "$@" 2>/dev/null
    fi
}
fail=0
check() { # name, expected text, actual output
    if grep -qF -- "$2" <<<"$3"; then echo "ok   $1"; else echo "FAIL $1: expected '$2' in:"; sed 's/^/    /' <<<"$3"; fail=1; fi
}

out=$(run --root "$work/home" --print-output -e ': sq dup * ; 7 sq .' -e 'bye')
check "evaluates and exits on bye" "49" "$out"

run --root "$work/home" -e '" /saved.txt" creat value h  " kept" h write  h close' -e bye >/dev/null
check "writes files into its folder" "kept" "$(cat "$work/home/saved.txt" 2>/dev/null)"
out=$(run --root "$work/home" --print-output -e '" /saved.txt" slurp type' -e bye)
check "reads them back on the next start" "kept" "$out"

mkdir -p "$work/pkg/src"
echo '{"name": "Smoke", "start": ["src/start.f"]}' > "$work/pkg/reva-package.json"
echo ': hi ." package started" ; hi' > "$work/pkg/src/start.f"
out=$(run --package "$work/pkg" --print-output -e bye)
check "runs a package's start files" "package started" "$out"

exit $fail
