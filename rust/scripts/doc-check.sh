#!/usr/bin/env bash
# Check that every Forth sample in the given Markdown files produces the
# output the document shows.  A sample is a ```forth block followed by an
# ```output block; each sample runs in a fresh `reva` with the block as stdin.
#
#   scripts/doc-check.sh ../docs/*.md        (from rust/)
#
# Comparison ignores trailing whitespace on each line and trailing blank
# lines (Forth's `.` prints a trailing space that Markdown would lose).
# Prints one summary line per file; details for failures; exit 1 on any.
set -euo pipefail
cd "$(dirname "$0")/.."
REVA="${REVA:-target/debug/reva}"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
norm() { sed -e 's/[[:space:]]*$//' | sed -e ':a' -e '/^\n*$/{$d;N;ba' -e '}'; }
status=0
for md in "$@"; do
    rm -f "$work"/*
    # Split into numbered sample files: N.f (source), N.out (expected), N.line
    awk -v dir="$work" '
        /^```forth[[:space:]]*$/  { n++; mode = "f"; file = dir "/" n ".f"; printf "" > file
                                    print NR > (dir "/" n ".line"); next }
        /^```output[[:space:]]*$/ { if (n && !(n in done)) { mode = "o"; file = dir "/" n ".out"; printf "" > file }; next }
        /^```/                    { if (mode == "o") done[n] = 1; mode = ""; next }
        mode != ""                { print >> file }
    ' "$md"
    total=0; failed=0
    for src in "$work"/*.f; do
        [[ -e "$src" ]] || continue
        n=$(basename "$src" .f)
        [[ -e "$work/$n.out" ]] || { echo "$md:$(cat "$work/$n.line"): forth sample has no output block"; failed=$((failed+1)); total=$((total+1)); continue; }
        total=$((total+1))
        got=$("$REVA" < "$src" 2>&1 | norm)
        want=$(norm < "$work/$n.out")
        if [[ "$got" != "$want" ]]; then
            failed=$((failed+1))
            echo "$md:$(cat "$work/$n.line"): sample output differs"
            diff <(echo "$want") <(echo "$got") | sed 's/^/    /' || true
        fi
    done
    if [[ $failed -eq 0 ]]; then
        echo "$md: $total sample(s) ok"
    else
        echo "$md: $failed of $total sample(s) FAILED"
        status=1
    fi
done
exit $status
