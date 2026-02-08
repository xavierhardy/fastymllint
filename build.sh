#!/bin/bash
set -euo pipefail

echo "Building MegaLinter..."
cargo build --release

echo "Build complete. Executable is at target/release/megalinter"
