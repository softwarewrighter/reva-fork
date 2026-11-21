#!/usr/bin/env bash
# Run a command with a wall-clock limit, as GNU `timeout` does: its exit
# code, or 124 when the limit ended it.  Uses `timeout`, else `gtimeout`
# (Homebrew's coreutils), else a portable fallback (the command in the
# background, a watcher that ends it), so scripts run on a stock Mac too.
#
#   scripts/run-limit.sh SECONDS COMMAND [ARG ...]
set -uo pipefail
[[ $# -ge 2 ]] || { echo "usage: run-limit.sh SECONDS COMMAND [ARG ...]" >&2; exit 2; }
limit=$1
shift
# RUN_LIMIT_PORTABLE=1: always the fallback (to test it)
[[ -n "${RUN_LIMIT_PORTABLE:-}" ]] || for t in timeout gtimeout; do
    if command -v "$t" >/dev/null; then exec "$t" "$limit" "$@"; fi
done
flag=$(mktemp)
rm -f "$flag"
# <&0: a background command's stdin would otherwise be /dev/null
"$@" <&0 &
pid=$!
(
    sleep "$limit"
    if kill -0 "$pid" 2>/dev/null; then
        : > "$flag"
        kill -TERM "$pid" 2>/dev/null
        sleep 2
        kill -KILL "$pid" 2>/dev/null
    fi
) </dev/null >/dev/null 2>&1 &
watcher=$!
wait "$pid" 2>/dev/null  # 2>: no "Terminated" notice from bash
code=$?
kill "$watcher" 2>/dev/null
wait "$watcher" 2>/dev/null
if [[ -e "$flag" ]]; then
    rm -f "$flag"
    exit 124
fi
exit "$code"
