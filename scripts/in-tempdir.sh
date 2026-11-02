#!/usr/bin/env bash
# Run a command in a fresh temporary directory, then remove it, so a test
# that writes files (the notebook's notebook.txt, ...) never touches the
# checkout.  REVA (default target/debug/reva) and ROOT (the checkout) are
# exported as absolute paths; the command's exit status is the script's.
#
#   scripts/in-tempdir.sh sh -c '"$REVA" --ui-script "$ROOT/tests/ui/x.ui" < /dev/null'
set -u
ROOT=$(cd "$(dirname "$0")/.." && pwd)
REVA=${REVA:-target/debug/reva}
case "$REVA" in /*) ;; *) REVA="$ROOT/$REVA" ;; esac
export ROOT REVA
dir=$(mktemp -d) || exit 1
(cd "$dir" && "$@")
status=$?
rm -rf "$dir"
exit $status
