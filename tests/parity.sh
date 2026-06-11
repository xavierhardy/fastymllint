#!/usr/bin/env bash
# Side-by-side parity check: runs the real yamllint (from .venv) and
# fastymllint on the same files and compares output (yamllint standard
# format) and exit codes.
set -u

cd "$(dirname "$0")/.."

YAMLLINT=.venv/bin/yamllint
FASTYMLLINT=${FASTYMLLINT:-target/release/fastymllint}

if [ ! -x "$YAMLLINT" ]; then
    echo "yamllint not found; run: python3 -m venv .venv && .venv/bin/pip install yamllint" >&2
    exit 2
fi
if [ ! -x "$FASTYMLLINT" ]; then
    echo "fastymllint not found; run: cargo build --release" >&2
    exit 2
fi

configs=(
    "extends: default"
    "extends: relaxed"
)
config_files=(tests/configs/*.yaml)

files=(examples/yaml/*.yaml tests/data/tricky/*.yaml)

total=0
failures=0

compare() {
    local desc="$1"; shift
    local yl_out fy_out yl_code fy_code
    yl_out=$("${yl_cmd[@]}" "$@" 2>/dev/null); yl_code=$?
    fy_out=$("${fy_cmd[@]}" "$@" 2>/dev/null); fy_code=$?
    total=$((total + 1))
    if [ "$yl_out" != "$fy_out" ] || [ "$yl_code" != "$fy_code" ]; then
        failures=$((failures + 1))
        echo "MISMATCH: $desc"
        echo "  exit codes: yamllint=$yl_code fastymllint=$fy_code"
        diff <(printf '%s\n' "$yl_out") <(printf '%s\n' "$fy_out") | head -10 | sed 's/^/  /'
    fi
}

for config in "${configs[@]}"; do
    yl_cmd=("$YAMLLINT" -f standard -d "$config")
    fy_cmd=("$FASTYMLLINT" -f yamllint -d "$config")
    for f in "${files[@]}"; do
        compare "[-d '$config'] $f" "$f"
    done
    # All files at once
    compare "[-d '$config'] all files" "${files[@]}"
done

for cf in "${config_files[@]}"; do
    yl_cmd=("$YAMLLINT" -f standard -c "$cf")
    fy_cmd=("$FASTYMLLINT" -f yamllint -c "$cf")
    for f in "${files[@]}"; do
        compare "[-c $cf] $f" "$f"
    done
done

# strict mode and --no-warnings
yl_cmd=("$YAMLLINT" -f standard -s)
fy_cmd=("$FASTYMLLINT" -f yamllint -s)
compare "[strict] all files" "${files[@]}"

yl_cmd=("$YAMLLINT" -f standard --no-warnings)
fy_cmd=("$FASTYMLLINT" -f yamllint --no-warnings)
compare "[no-warnings] all files" "${files[@]}"

echo
echo "parity: $((total - failures))/$total comparisons matched"
if [ "$failures" -gt 0 ]; then
    exit 1
fi
