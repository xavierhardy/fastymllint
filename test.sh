#!/bin/bash
set -euo pipefail

echo "Running tests for MegaLinter..."
cargo test

echo "All tests passed!"
