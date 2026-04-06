#!/usr/bin/env bash
# Lint (fmt + clippy), test, and line coverage for all targets (≥80%).
# Requires: rustfmt + clippy (default stable toolchain), llvm-tools-preview, cargo-llvm-cov.
set -euo pipefail
cd "$(dirname "$0")"

echo "== cargo fmt =="
cargo fmt --all -- --check

echo "== cargo clippy =="
cargo clippy --all-targets --all-features -- -D warnings

echo "== cargo test =="
cargo test

echo "== cargo llvm-cov (all targets, fail-under-lines 80) =="
cargo llvm-cov --all-targets --fail-under-lines 80

echo "All checks passed."
