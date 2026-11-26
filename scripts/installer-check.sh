#!/usr/bin/env bash
# Check an installed demo package app (tests/installer-demo, built by
# scripts/make-installer.sh): it starts from the install, seeds the user's
# folder, reads the user's copy over the package's, and writes only there.
#
#   scripts/installer-check.sh APP      APP: the installed program
#       (macOS: ".../Reva Demo.app/Contents/MacOS/reva-demo"; Linux: /usr/bin/reva-demo;
#       Windows: "$LOCALAPPDATA/Reva Demo/reva-demo.exe")
#
# Runs with a throwaway HOME, so Documents/Reva Demo is made there.  Windows
# finds Documents through the shell, not HOME: there the real
# Documents/Reva Demo is used, removed first (CI's throwaway account).  Opens
# a window (Linux: under xvfb-run when there is no display): CI, not the
# shared Mac.
set -euo pipefail
app="$1"
limit="$(cd "$(dirname "$0")" && pwd)/run-limit.sh"
home=$(mktemp -d)
trap 'rm -rf "$home"' EXIT
run() {
    if [[ -z "${DISPLAY:-}" && "$(uname)" == Linux ]]; then
        HOME="$home" XDG_CONFIG_HOME="$home/.config" "$limit" 60 xvfb-run -a "$app" --print-output "$@" 2>/dev/null
    else
        HOME="$home" "$limit" 60 "$app" --print-output "$@" 2>/dev/null
    fi
}
user="$home/Documents/Reva Demo"
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*)
        user="$(cygpath -u "$USERPROFILE")/Documents/Reva Demo"
        rm -rf "$user" ;;
esac
fail=0
check() { # name, expected text, actual
    if grep -qF -- "$2" <<<"$3"; then echo "ok   $1"; else echo "FAIL $1: expected '$2' in:"; sed 's/^/    /' <<<"$3"; fail=1; fi
}
if [[ "$(uname)" == Darwin ]]; then
    bundle="$(cd "$(dirname "$app")/../.." && pwd)"
    sig=$(codesign -dv "$bundle" 2>&1 || true)
    check "the app is ad-hoc signed" "Signature=adhoc" "$sig"
    codesign --verify --deep --strict "$bundle" && echo "ok   the signature verifies" || fail=1
fi
out=$(run -e bye)
check "starts the installed package" "Reva Demo started" "$out"
check "shows the seeded notes" "These notes are yours" "$out"
check "seeds the user's folder" "These notes are yours" "$(cat "$user/notes.txt" 2>/dev/null)"
echo "my own notes" > "$user/notes.txt"
out=$(run -e '" /mine.txt" creat value h  " kept" h write  h close' -e bye)
check "reads the user's copy over the package's" "my own notes" "$out"
check "writes into the user's folder" "kept" "$(cat "$user/mine.txt" 2>/dev/null)"
out=$(run -e bye)
check "an upgrade or restart keeps the user's notes" "my own notes" "$out"
exit $fail
