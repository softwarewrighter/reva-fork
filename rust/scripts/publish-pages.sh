#!/usr/bin/env bash
# Publish the live demo: the plain reva-fork web REPL (no package) as one
# orphan commit force-pushed to the gh-pages branch, which GitHub Pages
# serves at https://softwarewrighter.github.io/reva-fork/.
#
#   rust/scripts/publish-pages.sh            build from main and publish
#   rust/scripts/publish-pages.sh --dry-run  build the commit, do not push
#
# Only main is published, so the site always names a released commit (its
# footer shows the build commit): the tree must be clean and HEAD on
# origin/main.  --dry-run skips that check, to try the build from any
# commit.  The site is make-web-zip.sh's static tree (split view by
# default) plus .nojekyll, so GitHub serves every file as it is.  The
# working tree and the real index are not touched.
set -euo pipefail
cd "$(dirname "$0")/../.."
push=yes
[[ "${1:-}" == --dry-run ]] && push=no

if [[ $push == yes ]]; then
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "publish-pages: the tree is dirty; commit or stash first" >&2
        exit 1
    fi
    git fetch -q origin main
    if ! git merge-base --is-ancestor HEAD origin/main; then
        echo "publish-pages: HEAD is not on origin/main; publish from a pushed main" >&2
        exit 1
    fi
fi

out="$(mktemp -d)"
index="$(mktemp)"
trap 'rm -rf "$out" "$index"' EXIT
rm -f "$index"
rust/scripts/make-web-zip.sh --out "$out" >/dev/null
site="$(find "$out" -mindepth 1 -maxdepth 1 -type d -name 'reva-web-*')"
[[ -f "$site/index.html" ]] || { echo "publish-pages: no site built in $out" >&2; exit 1; }
touch "$site/.nojekyll"

# A tree of the site alone, from a throwaway index; -f because the
# repository's ignore rules are about the source tree, not the site.
head="$(git rev-parse --short HEAD)"
gitdir="$(git rev-parse --absolute-git-dir)"
tree="$(cd "$site" && export GIT_DIR="$gitdir" GIT_WORK_TREE=. GIT_INDEX_FILE="$index" &&
    git add -A -f . && git write-tree)"
commit="$(git commit-tree "$tree" -m "pages: the reva-fork web REPL built from $head")"
if [[ $push == no ]]; then
    echo "publish-pages: built $commit from $head ($(git ls-tree -r --name-only "$commit" | wc -l | tr -d ' ') files); not pushed (--dry-run)"
    exit 0
fi
git push -f origin "$commit:refs/heads/gh-pages"
echo "publish-pages: gh-pages is the site of main $head ($commit)"
echo "publish-pages: https://softwarewrighter.github.io/reva-fork/"
