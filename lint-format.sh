#!/bin/bash
set -euo pipefail

echo "Running Rust formatter (cargo fmt)..."
cargo fmt --all

echo "Running Rust linter (cargo clippy)..."
cargo clippy -- -D warnings

echo "Linting and formatting complete."
