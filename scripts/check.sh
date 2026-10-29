#!/usr/bin/env bash
# The gate: everything that must pass before a commit (`just check`).
#
#   scripts/check.sh            all of it
#   scripts/check.sh --fast     skip the desktop component (Tauri)
#
# Runs, and stops at the first failure:
#   1. every component (components/*, one Cargo workspace each) but the
#      desktop: cargo fmt --check, clippy -D warnings, tests (release);
#      and every component on the same version
#   2. the WebAssembly build and its node tests
#   3. components/desktop: cargo fmt --check, clippy -D warnings, tests
#   4. the reg-rs goldens on the release CLI; a failure is accepted only
#      if KNOWN_FAILURES lists it for this OS
#   5. the license check
#   6. sw-checklist: its FAIL count may not exceed SW_CHECKLIST_MAX_FAILS
#
# Bash 3.2 compatible (macOS /bin/bash).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"

# Goldens known to fail, per OS, until the step that fixes them
# (.agentrail: local-portable-gates).  The list may only shrink.
case "$(uname)" in
    Darwin) KNOWN_FAILURES="docs-samples" ;;
    *)      KNOWN_FAILURES="" ;;
esac

# sw-checklist FAILs: 51 on 2026-10-26, 49 after edition 2024, 47 after
# the web checks, 46 after the split of reva-vm, 40 after the word
# families (local-comp-words).  Lower this as steps fix them
# (.agentrail: local-edition-2024 ... local-split-front-ends); at 0 the
# checklist is required outright.
SW_CHECKLIST_MAX_FAILS=40

fast=0
[[ "${1:-}" == --fast ]] && fast=1

step() { printf '\n== %s\n' "$*"; }
fail() { printf '\ncheck: FAILED: %s\n' "$*" >&2; exit 1; }

# The components the core builds from, dependencies first; the desktop
# (Tauri, a few hundred crates) is step 3.
COMPONENTS="x86 host core vlib words-base words-interp words-dict words-os words vm sqlite cli web serve"

version() { awk -F'"' '/^version *=/ { print $2; exit }' "$root/components/$1/Cargo.toml"; }
for c in $COMPONENTS; do
    step "components/$c: fmt, clippy, tests"
    (cd "$root/components/$c" && cargo fmt --check) || fail "$c: cargo fmt --check (run cargo fmt in components/$c)"
    (cd "$root/components/$c" && cargo clippy -q --workspace --all-targets -- -D warnings) || fail "$c: clippy"
    (cd "$root/components/$c" && cargo test -q --release --workspace) || fail "$c: tests"
    [[ "$(version "$c")" == "$(version vm)" ]] || fail "$c: version $(version "$c"), not $(version vm) as components/vm"
done

step "web: wasm build, node tests"
"$root/scripts/build-web.sh" >/dev/null || fail "web: build-web.sh"
for t in smoke lineedit ansi; do
    node "$root/components/web/crates/reva-web/test/$t.mjs" >/dev/null || fail "web: test/$t.mjs (run it for details)"
done

if [[ $fast -eq 0 ]]; then
    step "desktop: fmt, clippy, tests"
    (cd "$root/components/desktop" && cargo fmt --check) || fail "desktop: cargo fmt --check"
    "$root/components/desktop/build.sh" >/dev/null 2>&1 || fail "desktop: build.sh (run it for details)"
    (cd "$root/components/desktop" && cargo clippy -q --workspace --all-targets -- -D warnings) || fail "desktop: clippy"
    (cd "$root/components/desktop" && cargo test -q --workspace) || fail "desktop: tests"
fi

step "goldens (reg-rs, release CLI)"
out=$(cd "$root" && REVA=target/release/reva scripts/regress.sh 2>&1) || true
echo "$out" | grep -E 'passed|FAIL' || fail "goldens: no summary from regress.sh"
unexpected=""
for t in $(echo "$out" | sed -n 's/^ *FAIL: *//p'); do
    case " $KNOWN_FAILURES " in
        *" $t "*) echo "   (known failure on $(uname): $t)" ;;
        *)        unexpected="$unexpected $t" ;;
    esac
done
[[ -z "$unexpected" ]] || fail "goldens:$unexpected (details: REVA=target/release/reva scripts/regress.sh -v -p NAME)"

step "licenses"
"$root/scripts/license-check.sh" || fail "license-check.sh"

step "sw-checklist"
# sw-checklist exits 1 while it has FAILs; the count below is the test.
summary=$(cd "$root" && { sw-checklist 2>&1 || true; } | grep '^Summary:') || fail "sw-checklist did not run"
echo "$summary"
fails=$(echo "$summary" | sed -n 's/.* \([0-9][0-9]*\) failed.*/\1/p')
[[ -n "$fails" ]] || fail "sw-checklist: cannot read the FAIL count"
[[ "$fails" -le "$SW_CHECKLIST_MAX_FAILS" ]] ||
    fail "sw-checklist: $fails FAILs, more than $SW_CHECKLIST_MAX_FAILS (sw-checklist -v | grep FAIL)"

printf '\ncheck: ok\n'
