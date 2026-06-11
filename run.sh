#!/bin/bash
set -euo pipefail

cargo run --release -- "$@"
