#!/bin/sh
# build-stamp.sh: print "VERSION COMMIT DATE" for this checkout, the value
# of the page's `reva-build` meta tag (shown in the About dialog).
#
#   VERSION  the workspace version (rust/Cargo.toml)
#   COMMIT   the short commit hash, with "-dirty" if tracked files changed
#   DATE     the commit's date (UTC), not today's, so a rebuild of the same
#            commit gives the same stamp
# Outside a git checkout: "VERSION unknown unknown".
set -eu
RUST=$(cd "$(dirname "$0")/.." && pwd)
version=$(awk -F'"' '/^version *=/ { print $2; exit }' "$RUST/Cargo.toml")
commit=$(git -C "$RUST" rev-parse --short HEAD 2>/dev/null || echo unknown)
if [ "$commit" != unknown ] && ! git -C "$RUST" diff --quiet HEAD -- 2>/dev/null; then
    commit=$commit-dirty
fi
date=$(TZ=UTC git -C "$RUST" log -1 --format=%cd --date=format-local:%Y-%m-%d 2>/dev/null || echo unknown)
echo "$version $commit ${date:-unknown}"
