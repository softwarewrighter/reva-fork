#!/usr/bin/env bash
# Make desktop installers: reva-desktop with a package (or plain Reva).
#
#   scripts/make-installer.sh [--native] [PACKAGE_DIR]     (just installer ...)
#
# macOS: a universal .app in a .dmg (--native: this Mac's architecture
# only, quicker); Linux: .deb and .rpm; Windows (Git Bash): an NSIS
# setup.exe installing per user.  The installers land in target/installer/,
# named NAME_PKGVERSION_reva-VERSION_ARCH.EXT (plain Reva: Reva_VERSION_ARCH.EXT).
#
# The package's files ship read-only in the app (resources, package/); the
# app finds them there when started without --package, and keeps the user's
# files in Documents/<userFolder> (docs/packaging.md, "Installers").  A
# manifest with a "files" list ships the manifest, its start, files, seed,
# icon and image files; one without ships the whole folder but dot files.
# Unsigned (the user, 2026-11-22): macOS ad-hoc signed, unless
# APPLE_SIGNING_IDENTITY names a real identity.  Needs tauri-cli
# (cargo install tauri-cli --version '^2' --locked) and node.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
usage() { sed -n '2,19p' "$0" | cut -c3-; exit "${1:-0}"; }
native=0 pkg=""
for a in "$@"; do
    case "$a" in
        --native) native=1 ;;
        -h|--help) usage ;;
        -*) echo "make-installer: unknown option $a" >&2; usage 2 >&2 ;;
        *) pkg="$a" ;;
    esac
done
cargo tauri --version >/dev/null 2>&1 \
    || { echo "make-installer: needs tauri-cli: cargo install tauri-cli --version '^2' --locked" >&2; exit 1; }
version=$(tr -d ' \r\n' < "$root/VERSION")
out="$root/target/installer"
rm -rf "$out" && mkdir -p "$out"

stage=""
if [[ -n "$pkg" ]]; then
    [[ -f "$pkg/reva-package.json" ]] || { echo "make-installer: no reva-package.json in $pkg" >&2; exit 1; }
    pkg="$(cd "$pkg" && pwd)" stage="$out/package"
    mkdir -p "$stage"
    # the files to ship, one per line, or "*" for the whole folder
    list=$(node -e '
        const m = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8"));
        const s = (v) => (typeof v === "string" ? [v] : Array.isArray(v) ? v.filter((x) => typeof x === "string") : []);
        if (!Array.isArray(m.files)) { console.log("*"); process.exit(0); }
        const all = ["reva-package.json", ...s(m.start), ...s(m.files), ...s(m.seed), ...s(m.icon), ...s(m.image)];
        console.log([...new Set(all)].join("\n"));' "$pkg/reva-package.json")
    if [[ "$list" == "*" ]]; then
        cp -R "$pkg/." "$stage/"
        find "$stage" -mindepth 1 -name '.*' -prune -exec rm -rf {} +
    else
        while IFS= read -r f; do
            case "$f" in /*|*..*) echo "make-installer: $f is not a path inside the package" >&2; exit 1 ;; esac
            [[ -f "$pkg/$f" ]] || { echo "make-installer: $f (in reva-package.json) is missing" >&2; exit 1; }
            mkdir -p "$stage/$(dirname "$f")" && cp "$pkg/$f" "$stage/$f"
        done <<< "$list"
    fi
fi

IFS=$'\t' read -r name pkgversion icon < <(node "$root/scripts/installer-config.mjs" "$version" "$out" "$stage")
config=("$out/tauri.json")
if [[ -n "$icon" ]]; then
    # the bundles' icon set (.icns, .ico, PNGs) from the package's PNG
    cargo tauri icon "$stage/$icon" -o "$out/icons" >/dev/null
    printf '{"bundle": {"icon": ["%s/icons/32x32.png", "%s/icons/128x128.png", "%s/icons/icon.icns", "%s/icons/icon.ico"]}}\n' \
        "$out" "$out" "$out" "$out" > "$out/icons.json"
    config+=("$out/icons.json")
fi

target=() tdir="$root/target/release"
case "$(uname -s)" in
    Darwin)
        bundles=app,dmg arch=$(uname -m)
        # no Finder window: the dmg script skips its AppleScript layout under CI
        export CI="${CI:-true}"
        if [[ $native == 0 ]]; then
            rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null
            target=(--target universal-apple-darwin) tdir="$root/target/universal-apple-darwin/release" arch=universal
        fi ;;
    Linux) bundles=deb,rpm arch=$(uname -m) ;;
    MINGW*|MSYS*|CYGWIN*) bundles=nsis arch=$(uname -m) ;;
    *) echo "make-installer: no installers for $(uname -s)" >&2; exit 1 ;;
esac
rm -rf "$tdir/bundle"
args=()
for c in "${config[@]}"; do args+=(--config "$c"); done
"$root/components/desktop/build.sh" --bundle "${args[@]}" --bundles "$bundles" ${target[@]+"${target[@]}"}

# one name for every installer: the package's version and reva-fork's
base="${name// /-}_${pkgversion:+${pkgversion}_reva-}${version}_${arch}"
[[ -n "$pkgversion" ]] || base="${name// /-}_${version}_${arch}"
n=0
while IFS= read -r f; do
    ext="${f##*.}"
    cp "$f" "$out/$base.$ext"
    echo "installer: $out/$base.$ext"
    n=$((n + 1))
done < <(find "$tdir/bundle" \( -name '*.dmg' -o -name '*.deb' -o -name '*.rpm' -o -name '*-setup.exe' \) -type f)
[[ $n -gt 0 ]] || { echo "make-installer: no installer was made" >&2; exit 1; }
