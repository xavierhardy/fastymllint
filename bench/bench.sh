#!/usr/bin/env bash
# Speed comparison between yamllint and fastymllint.
#
# Scenarios:
#   1. single small file
#   2. single large file (generated)
#   3. many files (generated corpus)
#
# Uses hyperfine when available, otherwise falls back to a simple timing
# loop.
set -eu

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

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

# --- Generate fixtures -------------------------------------------------------

SMALL=examples/yaml/multiple_issues.yaml

LARGE="$WORK/large.yaml"
{
    echo "---"
    for i in $(seq 1 5000); do
        echo "key_$i:"
        echo "  name: item-$i"
        echo "  enabled: true"
        echo "  values: [1, 2, 3]"
        echo "  description: \"item number $i in a large generated file\""
    done
} > "$LARGE"

MANY="$WORK/many"
mkdir -p "$MANY"
i=0
while [ $i -lt 400 ]; do
    for f in examples/yaml/*.yaml; do
        cp "$f" "$MANY/$(basename "$f" .yaml)_$i.yaml"
        i=$((i + 1))
        [ $i -ge 400 ] && break
    done
done

run_bench() {
    local name="$1"; shift
    local target="$1"; shift
    echo
    echo "=== $name ==="
    if command -v hyperfine > /dev/null 2>&1; then
        hyperfine --warmup 1 -i \
            -n yamllint "$YAMLLINT -f parsable $target" \
            -n fastymllint "$FASTYMLLINT -f text $target"
    else
        for tool in "$YAMLLINT -f parsable" "$FASTYMLLINT -f text"; do
            local total=0 runs=5
            # one warmup
            $tool $target > /dev/null 2>&1 || true
            local start end
            start=$(python3 -c 'import time; print(time.time())')
            local n=0
            while [ $n -lt $runs ]; do
                $tool $target > /dev/null 2>&1 || true
                n=$((n + 1))
            done
            end=$(python3 -c 'import time; print(time.time())')
            python3 -c "print(f'  {\"$tool\".split(\"/\")[-1].split()[0]:>12}: {($end - $start) / $runs * 1000:8.1f} ms/run (avg of $runs)')"
        done
    fi
}

run_bench "single small file ($(wc -l < "$SMALL" | tr -d ' ') lines)" "$SMALL"
run_bench "single large file ($(wc -l < "$LARGE" | tr -d ' ') lines)" "$LARGE"
run_bench "many files (400 files)" "$MANY"
