#!/bin/bash
set -euo pipefail

echo "Running MegaLinter with provided arguments: $@"
cargo run -- "$@"
