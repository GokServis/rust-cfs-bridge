#!/usr/bin/env bash
# Lint (fmt + clippy), test, and line coverage for the library (≥90%).
# Requires: rustfmt + clippy (default stable toolchain), llvm-tools-preview, cargo-llvm-cov.
set -euo pipefail
cd "$(dirname "$0")"

echo "== cargo fmt =="
cargo fmt --all -- --check

echo "== cargo clippy =="
cargo clippy --all-targets --all-features -- -D warnings

echo "== cargo test =="
cargo test

echo "== cargo llvm-cov (lib, fail-under-lines 90) =="
cargo llvm-cov --lib --fail-under-lines 90

echo "All checks passed."
