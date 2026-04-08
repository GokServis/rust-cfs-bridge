#!/usr/bin/env bash
set -euo pipefail

CFS_CPU1_DIR=/app/cfs/build/exe/cpu1
CFS_LOG=/app/cfs-cpu1.log

if command -v sysctl >/dev/null 2>&1; then
  sysctl -w fs.mqueue.msg_max=256 >/dev/null 2>&1 || true
fi
if [[ -w /proc/sys/fs/mqueue/msg_max ]]; then
  echo 256 > /proc/sys/fs/mqueue/msg_max 2>/dev/null || true
fi

cd "$CFS_CPU1_DIR"

# Ensure host-mapped "cf" volume directories exist for CFDP RX temp/fail paths.
mkdir -p ./cf/tmp ./cf/fail

set -o pipefail
./core-cpu1 2>&1 | tee "${CFS_LOG}"
