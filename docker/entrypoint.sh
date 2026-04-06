#!/usr/bin/env bash
# Legacy monolith: cFS + bridge-server + static UI on one HTTP port (default 8080).
# For Compose, prefer docker-compose.yml: bridge-server + bridge-ui (nginx) and optional --profile cfs.
set -euo pipefail

CFS_CPU1_DIR=/app/cfs/build/exe/cpu1
BRIDGE_SERVER=/app/rust-bridge/target/release/bridge-server
CFS_LOG=/app/cfs-cpu1.log

export BRIDGE_STATIC_DIR=/app/bridge-ui/dist
export BRIDGE_TLM_BIND="${BRIDGE_TLM_BIND:-127.0.0.1:2234}"

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
exec "$BRIDGE_SERVER"
