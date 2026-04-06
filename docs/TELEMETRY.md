# Telemetry downlink (Rust bridge)

## Overview

- **Uplink** (commands): unchanged — `POST /api/send` → UDP → CI_LAB (see [MESSAGE_FLOW.md](MESSAGE_FLOW.md)).
- **Downlink** (telemetry): `bridge-server` listens on **UDP** for raw cFS / TO_LAB datagrams and broadcasts **JSON** to WebSocket clients at **`/api/tlm/ws`**. The **bridge-ui** telemetry dashboard is at route **`/telemetry`** (commands stay on **`/`**).

While **TO_LAB** subscriptions in `cfs/apps/to_lab/fsw/tables/to_lab_sub.c` remain commented out, cFS will not emit real HK telemetry; the listener is **ready but idle** until you enable TO output and subscriptions. Use the **mock script** below to validate the ground stack.

## Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `BRIDGE_TLM_BIND` | `127.0.0.1:5001` | UDP bind address for incoming telemetry |
| `BRIDGE_HTTP_BIND` | `127.0.0.1:8080` | HTTP + WebSocket bind |
| `BRIDGE_UDP_TARGET` | `127.0.0.1:1234` | Uplink UDP target (CI_LAB) |

Docker: [entrypoint.sh](../docker/entrypoint.sh) sets `BRIDGE_TLM_BIND` if unset.

## WebSocket protocol

- **URL:** `ws://<host>:<port>/api/tlm/ws` (same host as the UI; Vite dev proxies `/api` with WebSocket upgrade).
- **Messages:** one JSON object per telemetry datagram (tagged union, field `kind`):
  - `es_hk_v1` — parsed CFE Executive Services HK (Linux little-endian payload).
  - `parse_error` — raw datagram could not be parsed as ES HK v1 (includes `hex_preview`).

## Mock telemetry (no cFS changes)

From the repo root (Python 3):

```bash
python3 scripts/mock_es_hk_udp.py
# or
python3 scripts/mock_es_hk_udp.py 127.0.0.1:5001
```

Inside Docker (host network):

```bash
docker exec -it rust-cfs-bridge python3 /app/scripts/mock_es_hk_udp.py
```

You should see the **Telemetry overview** in the web UI update, or JSON in a WebSocket client.

## Uplink dictionary verification

Sends every command from `GET /api/commands` through `POST /api/send`:

```bash
python3 scripts/verify_uplink_dictionary.py
BRIDGE_HTTP_BASE=http://127.0.0.1:8080 python3 scripts/verify_uplink_dictionary.py
```

Confirm **bridge_reader** lines in `docker compose logs` or `/app/cfs-cpu1.log` match expected MsgId/APID (see [MESSAGE_FLOW.md](MESSAGE_FLOW.md)).

## Troubleshooting

| Symptom | Check |
|--------|--------|
| UI shows “Disconnected” for telemetry | WebSocket blocked; ensure dev proxy has `ws: true` for `/api`. Same origin as HTTP. |
| No UDP packets | Firewall; wrong `BRIDGE_TLM_BIND`; TO_LAB not sending (expected until configured). |
| `parse_error` only | Datagram size/layout differs from ES HK v1 (12 + 168 bytes LE); compare mission headers. |
| Logs | On start: `telemetry UDP listening on ...` from `bridge-server`. |

## Code quality commands

**Rust:** `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `cargo llvm-cov --all-targets --fail-under-lines 80`.

**bridge-ui:** `npm run lint`, `npm run lint:fix`, `npm run test:coverage`.
