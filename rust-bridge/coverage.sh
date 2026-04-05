#!/usr/bin/env bash
# Line coverage for the library (excludes the thin main binary). Requires:
#   rustup component add llvm-tools-preview
#   cargo install cargo-llvm-cov
set -euo pipefail
cd "$(dirname "$0")"
exec cargo llvm-cov --lib --fail-under-lines 90 "$@"
