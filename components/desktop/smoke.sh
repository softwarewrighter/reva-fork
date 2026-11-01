#!/usr/bin/env bash
# Smoke-test the desktop app without a person: start it with a private
# folder and a throwaway package, check what it prints and what it wrote.
#
#   components/desktop/smoke.sh        (after components/desktop/build.sh; on Linux it uses xvfb-run
#                            when there is no display)
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
app="${REVA_DESKTOP:-$here/../../target/release/reva-desktop}"
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
# a fresh system: the start file ran again, the word defined before is gone
out=$(run --package "$work/pkg" --print-output -e ': gone ;' -e restart -e gone -e bye | tr -d '\0\n')
check "restart includes the start files again" "portableok> package startedok> ERROR: unknown word:  gone" "$out"

# branding: the manifest's title, icon and id reach the X11 window
# (needs xprop and xdotool; skipped without them, and on macOS)
if [[ "$(uname)" == Linux ]] && command -v xprop >/dev/null && command -v xdotool >/dev/null; then
    cp -r "$work/pkg" "$work/brand"
    printf '\x89PNG\r\n\x1a\n' > "$work/brand/not.png"
    base64 -d > "$work/brand/icon.png" <<'PNG'
iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAFElEQVR42mM8kWL0n4GBgYGJAQoAJJ4CYfpMJrQAAAAASUVORK5CYII=
PNG
    echo '{"name": "Smoke", "title": "Smoke Pkg", "icon": "icon.png", "id": "reva-smoke", "start": ["src/start.f"]}' \
        > "$work/brand/reva-package.json"
    # a script, not a function: xvfb-run is sh, which drops exported functions
    cat > "$work/probe" <<'SH'
#!/usr/bin/env bash
# probe app args...: start the app, print its window's WM_CLASS, WM_NAME, icon size
"$@" & pid=$!
for _ in $(seq 60); do
    w=$(xdotool search --class reva-smoke 2>/dev/null | tail -1)
    [[ -n "$w" ]] && break
    sleep 0.5
done
[[ -n "$w" ]] && xprop -id "$w" WM_CLASS WM_NAME && xprop -id "$w" -notype 32c _NET_WM_ICON | cut -d, -f1-2
kill $pid
SH
    chmod +x "$work/probe"
    if [[ -z "${DISPLAY:-}" ]]; then probe_run() { timeout 60 xvfb-run -a "$work/probe" "$@" 2>/dev/null; }
    else probe_run() { timeout 60 "$work/probe" "$@" 2>/dev/null; }; fi
    out=$(probe_run "$app" --package "$work/brand")
    check "the manifest names the window" 'WM_NAME(STRING) = "Smoke Pkg"' "$out"
    check "the manifest sets the app id" 'WM_CLASS(STRING) = "reva-smoke"' "$out"
    check "the manifest sets the icon" '_NET_WM_ICON = 2, 2' "$out"
    echo '{"name": "Smoke", "icon": "not.png", "start": ["src/start.f"]}' > "$work/brand/reva-package.json"
    out=$(run --package "$work/brand" --print-output -e bye)
    check "a broken icon is a note, not a failure" "package started" "$out"
fi

# turtle graphics: the drawing reaches the second (canvas) window
out=$(run --root "$work/home" --print-output -e 'needs turtle/fractals 4 koch-demo' -e bye)
check "turtle graphics draw in their own window" "[canvas: 768 lines]" "$out"

# SQLite (host calls answered natively, crates/reva-desktop/src/sql.rs): a
# database written in one run is read in the next, as a file of the folder
sq="needs db/sqlite with~ ~db \" pets.db\" sql_open value db"
run --root "$work/home" --print-output -e "$sq" \
    -e "db \" create table p(n text); insert into p values('cat');\" sql_exec db sql_close" -e bye >/dev/null
out=$(run --root "$work/home" --print-output -e "$sq" -e 'db " select n from p" sql_fetch$ type cr' -e bye)
check "SQLite keeps a database in the folder" "cat" "$out"
check "the database is a file of the folder" "SQLite format 3" "$(head -c 15 "$work/home/pets.db" 2>/dev/null)"
out=$(run --root "$work/home" --print-output -e 'needs db/sqlite with~ ~db " ../out.db" sql_open .' -e bye)
check "a database outside the folder does not open" "0 " "$out"

exit $fail
