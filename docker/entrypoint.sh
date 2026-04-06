#!/usr/bin/env bash
set -euo pipefail

CFS_CPU1_DIR=/app/cfs/build/exe/cpu1
RUST_BRIDGE=/app/rust-bridge/target/release/rust-bridge
CFS_LOG=/app/cfs-cpu1.log

# Default msg_max (often 10) is too low for cFE Software Bus pipe depths; must run before core-cpu1.
# Requires a privileged container (see docker-compose.yml) so this sysctl is writable.
if command -v sysctl >/dev/null 2>&1; then
  sysctl -w fs.mqueue.msg_max=256 >/dev/null 2>&1 || true
fi
if [[ -w /proc/sys/fs/mqueue/msg_max ]]; then
  echo 256 > /proc/sys/fs/mqueue/msg_max 2>/dev/null || true
fi

cd "$CFS_CPU1_DIR"
./core-cpu1 2>&1 | tee "${CFS_LOG}" &
# Allow CI_LAB and bridge_reader to finish startup before the first UDP datagram.
sleep 2
exec "$RUST_BRIDGE"
