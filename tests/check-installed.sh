#!/usr/bin/env bash
# Check an installed Reva package app (scripts/make-installer.sh), on
# macOS, Linux or Windows (Git Bash).  For CI and by hand; CoSy's
# check-release.sh --installed can call it.  Windows without Git Bash:
# tests/check-installed.ps1.
#
#   tests/check-installed.sh APP [PACKAGE_VERSION [REVA_VERSION]]
#
#   APP: the installed program, for example
#     macOS:   "/Applications/Reva Demo.app/Contents/MacOS/reva-demo"
#     Linux:   /usr/bin/reva-demo
#     Windows: "$LOCALAPPDATA/Reva Demo/reva-demo.exe"
#   PACKAGE_VERSION: the package's about.version, checked in --about
#   REVA_VERSION: reva-fork's version (default: VERSION next to this script)
#
# The user's folder goes under $REVA_DOCUMENTS (the app reads it), a
# throwaway folder unless set, so a check never touches a user's real
# Documents.  Opens windows (Linux: under xvfb-run when there is no display).
set -euo pipefail
app="$1" pkgversion="${2:-}"
here="$(cd "$(dirname "$0")" && pwd)"
revaversion="${3:-$(tr -d ' \r\n' < "$here/../VERSION" 2>/dev/null || true)}"
limit="$here/../scripts/run-limit.sh"
[[ -x "$limit" ]] || limit=""
docs="${REVA_DOCUMENTS:-}"
if [[ -z "$docs" ]]; then
    docs=$(mktemp -d)
    trap 'rm -rf "$docs"' EXIT
fi
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) export REVA_DOCUMENTS="$(cygpath -w "$docs")" ;;
    *) export REVA_DOCUMENTS="$docs" ;;
esac
run() {
    local pre=()
    [[ -n "$limit" ]] && pre=("$limit" 60)
    if [[ -z "${DISPLAY:-}" && "$(uname)" == Linux ]]; then pre+=(xvfb-run -a); fi
    ${pre[@]+"${pre[@]}"} "$app" --print-output "$@" 2>/dev/null || true
}
fail=0
check() { # name, expected text, actual
    if grep -qF -- "$2" <<<"$3"; then echo "ok   $1"; else echo "FAIL $1: expected '$2' in:"; sed 's/^/    /' <<<"$3" | head -20; fail=1; fi
}
yes_no() { # name, command...
    if "${@:2}"; then echo "ok   $1"; else echo "FAIL $1"; fail=1; fi
}

# where the package was installed
dir="$(cd "$(dirname "$app")" && pwd)"
case "$(uname -s)" in
    Darwin) pkg="$dir/../Resources/package" ;;
    Linux) pkg=$( (dpkg -L "$(basename "$app")" 2>/dev/null || rpm -ql "$(basename "$app")" 2>/dev/null) \
               | grep '/package/reva-package.json$' | head -1)
           pkg="${pkg%/reva-package.json}" ;;
    *) pkg="$dir/package" ;;
esac
yes_no "the package is installed ($pkg)" test -f "$pkg/reva-package.json"
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) yes_no "sqlite3.dll is installed next to the app" test -f "$dir/sqlite3.dll" ;;
esac

if [[ "$(uname)" == Darwin ]]; then
    bundle="$(cd "$dir/../.." && pwd)"
    check "the app is ad-hoc signed (or signed)" "Signature=" "$(codesign -dv "$bundle" 2>&1 || true)"
    yes_no "the signature verifies" codesign --verify --deep --strict "$bundle"
fi

about=$("$app" --about 2>/dev/null || true)
sed 's/^/     /' <<<"$about"
check "--about: reva-fork $revaversion" "vm: reva-fork $revaversion" "$about"
[[ -z "$pkgversion" ]] || check "--about: the package's version $pkgversion" "$pkgversion" "$(grep '^package: ' <<<"$about" || true)"
check "--about: the package is the installed one" "(installed)" "$about"
# the user's folder, as the app names it (under $REVA_DOCUMENTS)
user=$(sed -n 's/^user folder: //p' <<<"$about" | tr -d '\r')
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) [[ -z "$user" ]] || user=$(cygpath -u "$user") ;; esac
fresh=1
[[ -d "$user" ]] && fresh=0
out=$(run -e '1 2 + .' -e version -e bye)
check "starts and evaluates" "3 " "$out"
check "version: reva-fork and its commit" "Commit:" "$out"
yes_no "made the user's folder ($user)" test -n "$user" -a -d "$user"
# on a fresh folder, what the first start made there are the seed files
if [[ $fresh == 1 ]]; then
    seeds=0
    while IFS= read -r f; do
        rel="${f#"$user"/}"
        yes_no "seeded $rel from the package" cmp -s "$f" "$pkg/$rel"
        seeds=$((seeds + 1))
    done < <(find "$user" -type f -not -name '.*' 2>/dev/null)
    echo "     ($seeds seed file(s))"
fi

printf 'the user copy\n' > "$user/reva-package.json"
out=$(run -e '" /reva-package.json" slurp type' -e bye)
check "reads the user's copy over the package's" "the user copy" "$out"
rm -f "$user/reva-package.json"

run -e '" /check-written.txt" creat value h  " written" h write  h close' -e bye >/dev/null
check "writes into the user's folder" "written" "$(cat "$user/check-written.txt" 2>/dev/null)"
yes_no "never into the install" test ! -e "$pkg/check-written.txt"

run -e ': check-saved ." from the saved image" ;' -e 'save check.img' -e bye >/dev/null
img="$user/check.img"
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) img="$(cygpath -w "$img")" ;; esac
out=$(run --image "$img" -e check-saved -e bye)
check "save, then restart from the image" "from the saved image" "$out"

out=$(run -e 'needs turtle/fractals 4 koch-demo' -e bye)
check "turtle graphics draw in their own window" "[canvas: 768 lines]" "$out"
exit $fail
