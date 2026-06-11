#!/bin/bash
set -euo pipefail

echo "Building fastymllint..."
cargo build --release

echo "Build complete. Executable is at target/release/fastymllint"
