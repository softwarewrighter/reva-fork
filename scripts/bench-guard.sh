#!/usr/bin/env bash
# A quick timing guard for refactors of hot code (gate dispatch, decode,
# exec): a few workloads on the release CLI, the median of several runs
# each, compared with this machine's recorded baseline.  Not a benchmark
# suite (that is the perf-baseline step): its only job is to notice a
# slowdown before it is committed.
#
#   scripts/bench-guard.sh            measure and compare (fail if slower)
#   scripts/bench-guard.sh --record   measure and record this host's baseline
#
# Workloads, each about a second on an Apple M-series Mac:
#   boot    ten launches of `reva -e bye` (all of reva.f)
#   fib     recursive fib: calls and returns
#   loop    a fetch in a tight do-loop (bench/bench.f's `dl`)
#   dragon  dragon curves: turtle gateways and the canvas
# Timings only compare on the same machine, so the baseline file
# (work/bench-guard.txt) keeps one line per host and workload.  A median
# more than TOLERANCE percent above the baseline fails (default 15: the
# run-to-run spread on the reference Mac is up to about 10%).
set -euo pipefail
cd "$(dirname "$0")/.."
RUNS="${RUNS:-5}"
TOLERANCE="${TOLERANCE:-15}"
BASELINE=work/bench-guard.txt
REVA=target/release/reva
host="$(hostname -s 2>/dev/null || hostname)"
record=0
[[ "${1:-}" == --record ]] && record=1

cargo build -q --release --manifest-path components/cli/Cargo.toml

declare -a names=(boot fib loop dragon)
work() {
    case $1 in
        boot) for _ in 1 2 3 4 5 6 7 8 9 10; do "$REVA" -e bye </dev/null >/dev/null; done ;;
        fib) "$REVA" -e ': fib dup 2 < if drop 1 ;then 1- dup fib swap 1- fib + ; 30 fib drop bye' </dev/null >/dev/null ;;
        loop) "$REVA" -e 'create joe 0 , : dl 100000 0do joe @ drop loop ; : dlt 0do dl loop ; 100 dlt bye' </dev/null >/dev/null ;;
        dragon) "$REVA" -e 'needs turtle/fractals : dragons 0do 14 dragon-demo loop ; 12 dragons bye' </dev/null >/dev/null ;;
    esac
}
# milliseconds since some start, for differences (perl: portable, and BSD
# date has no %N)
now() { perl -MTime::HiRes=time -e 'printf "%.0f\n", time*1000'; }

failed=0
out=""
for name in "${names[@]}"; do
    samples=()
    for _ in $(seq "$RUNS"); do
        t0=$(now); work "$name"; t1=$(now)
        samples+=($((t1 - t0)))
    done
    median=$(printf '%s\n' "${samples[@]}" | sort -n | awk '{a[NR]=$1} END {print a[int((NR+1)/2)]}')
    base=$(awk -v h="$host" -v w="$name" '$1==h && $2==w {print $3}' "$BASELINE" 2>/dev/null || true)
    if [[ $record -eq 1 || -z "$base" ]]; then
        printf '%-7s %6d ms  (recorded)\n' "$name" "$median"
    else
        pct=$(( (median - base) * 100 / base ))
        verdict=ok
        [[ $pct -gt $TOLERANCE ]] && { verdict="SLOWER than ${TOLERANCE}%"; failed=1; }
        printf '%-7s %6d ms  baseline %6d ms  %+4d%%  %s\n' "$name" "$median" "$base" "$pct" "$verdict"
    fi
    out+="$host $name $median"$'\n'
done

if [[ $record -eq 1 ]] || ! grep -q "^$host " "$BASELINE" 2>/dev/null; then
    { grep -v "^$host " "$BASELINE" 2>/dev/null || true; printf '%s' "$out"; } | sort > "$BASELINE.tmp"
    mv "$BASELINE.tmp" "$BASELINE"
    echo "bench-guard: recorded the baseline for $host in $BASELINE (commit it)"
fi
exit $failed
