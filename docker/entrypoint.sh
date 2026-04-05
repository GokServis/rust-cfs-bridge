#!/usr/bin/env bash
set -euo pipefail

CFS_CPU1_DIR=/app/cfs/build/exe/cpu1
RUST_BRIDGE=/app/rust-bridge/target/release/rust-bridge

cd "$CFS_CPU1_DIR"
./core-cpu1 &
exec "$RUST_BRIDGE"
