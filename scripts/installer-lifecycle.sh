#!/usr/bin/env bash
# An installer's life on this OS, as a user meets it (CI; step
# local-installer-tests).  With the demo package (tests/installer-demo):
#
#   1. build its installer at 1.0.0 and at 1.0.1 (scripts/make-installer.sh)
#   2. install 1.0.0 as a user does: macOS, the .dmg mounted and the app
#      copied out; Linux, the .deb with apt; Windows, setup.exe /S (per user)
#   3. check the installed app (tests/check-installed.sh) and take a
#      screenshot of it with an IUP dialog and turtle drawing open
#   4. add a file of the user's, install 1.0.1 over 1.0.0, check again: the
#      new version runs and the user's folder is kept
#   5. uninstall: the app is gone and the user's folder is still there
#
#   scripts/installer-lifecycle.sh [OUT]    screenshots and installers into OUT
#                                           (default target/installer-life)
#
# Installs and uninstalls on this machine (Linux: sudo apt): for CI runners.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$root/target/installer-life}"
rm -rf "$out" && mkdir -p "$out"
work=$(mktemp -d)
export REVA_DOCUMENTS="$work/Documents"
mkdir -p "$REVA_DOCUMENTS"
os=$(uname -s)
case "$os" in MINGW*|MSYS*|CYGWIN*) os=Windows ;; esac
step() { echo; echo "== $*"; }

build() { # version → the installer, copied into $out
    local v="$1" pkg="$work/pkg-$1"
    cp -R "$root/tests/installer-demo" "$pkg"
    node -e 'const f=process.argv[1], fs=require("fs"); const m=JSON.parse(fs.readFileSync(f,"utf8"));
             m.about.version=process.argv[2]; fs.writeFileSync(f, JSON.stringify(m, null, 2))' \
        "$pkg/reva-package.json" "$v"
    "$root/scripts/make-installer.sh" --native "$pkg" | grep '^installer:' >&2
    local f
    f=$(find "$root/target/installer" -maxdepth 1 \( -name '*.dmg' -o -name '*.deb' -o -name '*-setup.exe' \) | head -1)
    cp "$f" "$out/"
    echo "$out/$(basename "$f")"
}

install() { # installer → the installed program
    case "$os" in
        Darwin)
            local mnt="$work/mnt"
            mkdir -p "$mnt" "$work/Applications"
            hdiutil attach -nobrowse -readonly -mountpoint "$mnt" "$1" >/dev/null
            rm -rf "$work/Applications/Reva Demo.app"
            cp -R "$mnt/Reva Demo.app" "$work/Applications/"
            hdiutil detach "$mnt" >/dev/null
            echo "$work/Applications/Reva Demo.app/Contents/MacOS/reva-demo" ;;
        Linux)
            sudo apt-get install -y --allow-downgrades "$1" >/dev/null
            echo /usr/bin/reva-demo ;;
        Windows)
            "$1" //S
            echo "$LOCALAPPDATA/Reva Demo/reva-demo.exe" ;;
    esac
}

uninstall() {
    case "$os" in
        Darwin) rm -rf "$work/Applications/Reva Demo.app" ;;
        Linux) sudo apt-get remove -y reva-demo >/dev/null ;;
        Windows)
            "$LOCALAPPDATA/Reva Demo/uninstall.exe" //S
            # the uninstaller copies itself and returns at once
            for _ in $(seq 60); do [[ -e "$LOCALAPPDATA/Reva Demo/reva-demo.exe" ]] || break; sleep 1; done ;;
    esac
}

screenshot() { # app, file: the app with an IUP dialog and turtle drawing open
    local app="$1" shot="$2"
    local forth='needs ui/gui with~ ~ui dialog[ " Installed: an IUP dialog" label ]d " Reva Demo check" title show
needs turtle/fractals 4 koch-demo'
    case "$os" in
        Linux)
            xvfb-run -a -s "-screen 0 1440x900x24" bash -c '
                "$1" -e "$2" & pid=$!; sleep 10; import -window root "$3"; kill $pid' _ "$app" "$forth" "$shot" ;;
        Darwin)
            "$app" -e "$forth" & local pid=$!
            sleep 10; screencapture -x "$shot" || true; kill $pid ;;
        Windows)
            "$app" -e "$forth" & local pid=$!
            sleep 10
            powershell -NoProfile -Command "Add-Type -AssemblyName System.Windows.Forms,System.Drawing;
                \$b = [System.Windows.Forms.SystemInformation]::VirtualScreen;
                \$i = New-Object System.Drawing.Bitmap \$b.Width, \$b.Height;
                [System.Drawing.Graphics]::FromImage(\$i).CopyFromScreen(\$b.Location, [System.Drawing.Point]::Empty, \$b.Size);
                \$i.Save('$(cygpath -w "$shot")')" || true
            kill $pid 2>/dev/null || true ;;
    esac
    if [[ -s "$shot" ]]; then echo "ok   screenshot: $shot"; else echo "note: no screenshot on $os"; fi
}

fail=0
step "build 1.0.0 and 1.0.1"
v1=$(build 1.0.0)
v2=$(build 1.0.1)

step "install 1.0.0 ($(basename "$v1"))"
app=$(install "$v1")
"$root/tests/check-installed.sh" "$app" 1.0.0 || fail=1
screenshot "$app" "$out/screenshot-$os-1.0.0.png" || true
user="$REVA_DOCUMENTS/Reva Demo"
echo "the user's own" > "$user/mine.txt"
echo "my notes" > "$user/notes.txt"

step "install 1.0.1 over it ($(basename "$v2"))"
app=$(install "$v2")
"$root/tests/check-installed.sh" "$app" 1.0.1 || fail=1
check() { if [[ "$(cat "$2" 2>/dev/null)" == "$3" ]]; then echo "ok   $1"; else echo "FAIL $1"; fail=1; fi; }
check "the upgrade kept the user's file" "$user/mine.txt" "the user's own"
check "the upgrade kept the user's seed copy (notes.txt)" "$user/notes.txt" "my notes"
if [[ $os == Windows ]]; then
    step "the same checks without Git Bash (tests/check-installed.ps1)"
    powershell -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w "$root/tests/check-installed.ps1")" \
        -App "$(cygpath -w "$app")" -PackageVersion 1.0.1 || fail=1
fi

step "uninstall"
uninstall
if [[ -e "$app" ]]; then echo "FAIL the app is gone"; fail=1; else echo "ok   the app is gone"; fi
check "the user's folder is kept" "$user/mine.txt" "the user's own"
rm -rf "$work"
exit $fail
