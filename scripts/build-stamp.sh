#!/bin/sh
# build-stamp.sh: print "VERSION COMMIT DATE" for this checkout, the value
# of the page's `reva-build` meta tag (shown in the About dialog).
#
#   VERSION  the version (components/vm/Cargo.toml, the one every component shares)
#   COMMIT   the short commit hash, with "-dirty" if tracked files changed
#   DATE     the commit's date (UTC), not today's, so a rebuild of the same
#            commit gives the same stamp
# Outside a git checkout: "VERSION unknown unknown".
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
version=$(awk -F'"' '/^version *=/ { print $2; exit }' "$ROOT/components/vm/Cargo.toml")
commit=$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)
if [ "$commit" != unknown ] && ! git -C "$ROOT" diff --quiet HEAD -- 2>/dev/null; then
    commit=$commit-dirty
fi
date=$(TZ=UTC git -C "$ROOT" log -1 --format=%cd --date=format-local:%Y-%m-%d 2>/dev/null || echo unknown)
echo "$version $commit ${date:-unknown}"
