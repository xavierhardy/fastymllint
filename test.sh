#!/bin/bash
set -euo pipefail

# The parity tests compare against the real yamllint; set it up once with:
#   python3 -m venv .venv && .venv/bin/pip install yamllint
if [ ! -x .venv/bin/yamllint ]; then
    echo "warning: .venv/bin/yamllint missing; parity tests will fail." >&2
    echo "         Run: python3 -m venv .venv && .venv/bin/pip install yamllint" >&2
fi

echo "Running tests for fastymllint..."
cargo test

echo "All tests passed!"
