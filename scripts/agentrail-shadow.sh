#!/usr/bin/env bash
# Copy agentrail's session logs into the private shadow repository,
# xz-compressed, then commit and push them there.
#
#   scripts/agentrail-shadow.sh            (from anywhere in the repository)
#
# The logs (.agentrail/sessions/*.jsonl) are not committed in reva-fork
# (.gitignore): one passed GitHub's 100 MiB file limit, and they quote every
# file read in a session, CoSy's among them, while this repository is public
# (docs/agentrail-asks.md). Run it after each saga-record commit.
# REVA_AGENTRAIL_SHADOW names the shadow clone (default ../reva-fork-agentrail,
# a clone of the private softwarewrighter/reva-fork-agentrail).
set -euo pipefail
cd "$(dirname "$0")/.."
shadow="${REVA_AGENTRAIL_SHADOW:-../reva-fork-agentrail}"
if [ ! -d "$shadow/.git" ]; then
    echo "agentrail-shadow: no shadow clone at $shadow" >&2
    echo "  gh repo clone softwarewrighter/reva-fork-agentrail $shadow" >&2
    exit 1
fi
mkdir -p "$shadow/sessions"
for log in .agentrail/sessions/*.jsonl; do
    [ -f "$log" ] || continue
    out="$shadow/sessions/$(basename "$log").xz"
    xz -T0 -9 -c "$log" > "$out.tmp"
    mv "$out.tmp" "$out"
done
cd "$shadow"
git add sessions
if git diff --cached --quiet; then
    echo "agentrail-shadow: nothing new"
    exit 0
fi
git commit -q -m "sessions: $(date -u +%Y-%m-%dT%H:%MZ)"
git push -q
echo "agentrail-shadow: $(git log -1 --format='%h %s') pushed"
