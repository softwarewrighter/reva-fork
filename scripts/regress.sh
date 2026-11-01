#!/usr/bin/env bash
# Run the reg-rs regression suite for the portable Reva core.
#
#   scripts/regress.sh            build, then run every test (summary)
#   scripts/regress.sh -v         ... with failure details (-vv: full diffs)
#   scripts/regress.sh -p NAME    ... only tests matching NAME
#
# Needs reg-rs (https://github.com/sw-cli-tools/reg-rs) on PATH, or $REG_RS
# pointing at the binary.  Tests run from the repository root against $REVA (default: the
# debug build), so the same suite checks debug and release builds:
#   REVA=target/release/reva scripts/regress.sh
set -euo pipefail
cd "$(dirname "$0")/.."
REG_RS="${REG_RS:-$(command -v reg-rs || true)}"
if [[ -z "$REG_RS" ]]; then
    echo "regress: reg-rs not found; install it or set REG_RS" >&2
    exit 2
fi
if [[ "${REVA:-target/debug/reva}" == target/release/reva ]]; then
    cargo build -q --release --manifest-path components/cli/Cargo.toml
else
    cargo build -q --manifest-path components/cli/Cargo.toml
fi
export REG_RS_DATA_DIR="$PWD/work/reg-rs"
exec "$REG_RS" run "$@"
