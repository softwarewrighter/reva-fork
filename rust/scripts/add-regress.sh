#!/usr/bin/env bash
# Add a regression test and capture its current output as the baseline.
#
#   scripts/add-regress.sh NAME 'COMMAND' 'description'
#
# COMMAND runs under `sh -c` from rust/; use ${REVA:-target/debug/reva} for
# the interpreter and feed stdin explicitly (< file or < /dev/null).
# Review the captured .out/.err before committing: a baseline is a claim
# that the output is *correct*, not just current.
set -euo pipefail
cd "$(dirname "$0")/.."
[[ $# -ge 2 ]] || { sed -n '2,9p' "$0"; exit 2; }
REG_RS="${REG_RS:-$(command -v reg-rs || true)}"
[[ -n "$REG_RS" ]] || { echo "add-regress: reg-rs not found; set REG_RS" >&2; exit 2; }
cargo build -q -p reva-cli
export REG_RS_DATA_DIR="$PWD/work/reg-rs"
mkdir -p "$REG_RS_DATA_DIR"
"$REG_RS" create -t "$1" -c "$2" --desc "${3:-}" --timeout 60
f="$REG_RS_DATA_DIR/$1"
echo "--- $f.out"; cat "$f.out" 2>/dev/null || true
if [[ -s "$f.err" ]]; then echo "--- $f.err"; cat "$f.err"; fi
