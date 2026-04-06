#!/usr/bin/env bash
set -euo pipefail

BRIDGE_SERVER=/app/rust-bridge/target/release/bridge-server

export BRIDGE_TLM_BIND="${BRIDGE_TLM_BIND:-127.0.0.1:2234}"
# Default: API + WebSocket only; nginx (bridge-ui) serves the SPA on :8080 and proxies /api.

if [[ ! -x "$BRIDGE_SERVER" ]]; then
  echo "entrypoint-bridge: missing $BRIDGE_SERVER (build rust-bridge first)" >&2
  exit 1
fi

exec "$BRIDGE_SERVER"
