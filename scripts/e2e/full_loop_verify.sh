#!/usr/bin/env bash
# Full-loop certification: Docker Compose (cFS profile) + brain upload + log golden sequence.
# Usage (from repo root):
#   ./scripts/e2e/full_loop_verify.sh
# Env:
#   BRIDGE_HTTP_BASE  default http://127.0.0.1:8080  (nginx → bridge-server :8081)
#   COMPOSE_PROJECT    optional docker compose project name
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

BRIDGE_HTTP_BASE="${BRIDGE_HTTP_BASE:-http://127.0.0.1:8080}"
WATCHER_TIMEOUT="${WATCHER_TIMEOUT:-420}"
UPLOAD_DELAY_SEC="${UPLOAD_DELAY_SEC:-45}"

echo "[full_loop_verify] repo: $ROOT"
echo "[full_loop_verify] bringing up stack (profile cfs)..."

docker compose --profile cfs down --remove-orphans 2>/dev/null || true
docker compose --profile cfs up --build -d

echo "[full_loop_verify] waiting for bridge HTTP health (${BRIDGE_HTTP_BASE})..."
ok=0
for _ in $(seq 1 90); do
  if curl -sf "${BRIDGE_HTTP_BASE%/}/api/health" >/dev/null 2>&1; then
    ok=1
    break
  fi
  sleep 2
done
if [[ "$ok" -ne 1 ]]; then
  echo "[full_loop_verify] ERROR: bridge health check failed after ~180s" >&2
  docker compose --profile cfs ps
  exit 1
fi

echo "[full_loop_verify] triggering brain upload in ${UPLOAD_DELAY_SEC}s (background)..."
# Allow cFS / udp_cfdp_ingest / SCH to settle; upload is async on bridge-server.
(sleep "${UPLOAD_DELAY_SEC}" && curl -sS -X POST "${BRIDGE_HTTP_BASE%/}/api/brain/upload" && echo "[full_loop_verify] POST /api/brain/upload sent") &
UP_PID=$!

echo "[full_loop_verify] watching cfs logs (timeout ${WATCHER_TIMEOUT}s)..."
set +e
python3 "$ROOT/scripts/e2e/e2e_log_watcher.py" --timeout "${WATCHER_TIMEOUT}"
WATCH_ST=$?
set -e

kill "$UP_PID" 2>/dev/null || true
wait "$UP_PID" 2>/dev/null || true

if [[ "$WATCH_ST" -ne 0 ]]; then
  echo "[full_loop_verify] log watcher exit $WATCH_ST — last cfs logs:" >&2
  docker compose --profile cfs logs --no-color --tail 120 cfs >&2 || true
fi

exit "$WATCH_ST"
