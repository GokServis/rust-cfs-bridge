#!/usr/bin/env bash
# Line coverage for all targets. Requires:
#   rustup component add llvm-tools-preview
#   cargo install cargo-llvm-cov
set -euo pipefail
cd "$(dirname "$0")"
exec cargo llvm-cov --all-targets --fail-under-lines 80 "$@"
