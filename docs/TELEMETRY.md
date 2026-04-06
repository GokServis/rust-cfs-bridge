# Telemetry downlink (Rust bridge)

## Overview

- **Uplink** (commands): unchanged — `POST /api/send` → UDP → CI_LAB (see [MESSAGE_FLOW.md](MESSAGE_FLOW.md)).
- **Downlink** (telemetry): `bridge-server` listens on **UDP** for raw cFS / TO_LAB datagrams and broadcasts **JSON** to WebSocket clients at **`/api/tlm/ws`**. The **bridge-ui** telemetry dashboard is at route **`/telemetry`** (commands stay on **`/`**).

**TO_LAB** subscriptions in `cfs/apps/to_lab/fsw/tables/to_lab_sub.c` include **ES HK** and **TO_LAB HK** MsgIds in this repo; onboard you must still run **`EnableOutput`** to the ground IP and drive HK (e.g. **SEND_HK** / SCH) so packets reach UDP. Use the **mock script** below to validate the ground stack without depending on cFS timing.

## Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `BRIDGE_TLM_BIND` | `127.0.0.1:2234` | UDP bind for incoming telemetry (matches default **`TO_LAB_MISSION_TLM_PORT`**) |
| `BRIDGE_HTTP_BIND` | `127.0.0.1:8080` | HTTP + WebSocket bind (Compose **bridge-server** uses **8081**; nginx on **8080** proxies `/api`) |
| `BRIDGE_UDP_TARGET` | `127.0.0.1:1234` | Uplink UDP target (CI_LAB) |

Docker: [entrypoint-bridge.sh](../docker/entrypoint-bridge.sh) / [entrypoint.sh](../docker/entrypoint.sh) set `BRIDGE_TLM_BIND` if unset.

## WebSocket protocol

- **URL:** `ws://<host>:<port>/api/tlm/ws` (same host as the UI; Vite dev proxies `/api` with WebSocket upgrade).
- **Messages:** one JSON object per telemetry datagram (tagged union, field `kind`):
  - `es_hk_v1` — parsed CFE Executive Services HK (Linux little-endian payload).
  - `to_lab_hk_v1` — parsed TO_LAB housekeeping (see [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md)).
  - `parse_error` — raw datagram did not match a known parser (includes `hex_preview`).

## Mock telemetry (no cFS changes)

From the repo root (Python 3):

```bash
python3 scripts/mock_es_hk_udp.py
# or
python3 scripts/mock_es_hk_udp.py 127.0.0.1:2234
```

Inside Docker (host network; **bridge-server** container):

```bash
docker exec -it rust-cfs-bridge-server python3 /app/scripts/mock_es_hk_udp.py
```

You should see the **Telemetry overview** and **telemetry log** (filters / pagination) on **`/telemetry`** update, or JSON in a WebSocket client.

## Manual check (Docker)

Use this after `docker compose build` and `docker compose up` on Linux with host networking ([docker/README.md](../docker/README.md)).

1. **Container / bridge-server** — In `docker compose logs -f` (or stderr), confirm a line like **`telemetry UDP listening on`** `127.0.0.1:2234` (or your `BRIDGE_TLM_BIND`).
2. **cFS** — Expect **core-cpu1** boot and **bridge_reader** subscription lines as in docker README; **`CI_LAB listening on UDP`** for uplink.
3. **Downlink smoke test** — `docker exec -it rust-cfs-bridge-server python3 /app/scripts/mock_es_hk_udp.py` — open **`http://127.0.0.1:8080/telemetry`**: **Bridge (API)** **Live**, **Downlink** **Live** after packets arrive, session packet count increases, **ES HK** panel and **log table** rows update.
4. **Filters** — Change **Kind** / **APID** / **Search** and use **Previous** / **Next** on the log; **Clear buffer** empties stored rows (WebSocket stays connected).
5. **Uplink (optional)** — Send **CMD_HEARTBEAT** / **CMD_PING** from `/` and match **bridge_reader** MsgId/APID lines in logs ([MESSAGE_FLOW.md](MESSAGE_FLOW.md)).

## Uplink dictionary verification

Sends every command from `GET /api/commands` through `POST /api/send`:

```bash
python3 scripts/verify_uplink_dictionary.py
BRIDGE_HTTP_BASE=http://127.0.0.1:8080 python3 scripts/verify_uplink_dictionary.py
```

Confirm **bridge_reader** lines in `docker compose logs` or `/app/cfs-cpu1.log` match expected MsgId/APID (see [MESSAGE_FLOW.md](MESSAGE_FLOW.md)).

## Live no-mock acceptance (optional)

With **`docker compose up`** and **no** `mock_es_hk_udp.py`:

1. Send **`CMD_TO_LAB_ENABLE_OUTPUT`** (e.g. `POST /api/send` with `{"command":"CMD_TO_LAB_ENABLE_OUTPUT","sequence_count":0}`).
2. With **cfs** running: **`docker compose --profile cfs logs`** or `docker exec rust-cfs-bridge-cfs grep -i 'telemetry output' /app/cfs-cpu1.log`, expect TO_LAB EVS text **`TO telemetry output enabled for IP`** (see `TO_LAB_EnableOutputCmd` in `to_lab_cmds.c`).
3. On **`ws://127.0.0.1:8080/api/tlm/ws`** (via nginx) or **`ws://127.0.0.1:8081/api/tlm/ws`** (direct to bridge-server), expect JSON with **`kind`** **`es_hk_v1`** and/or **`to_lab_hk_v1`** once SCH/HK drives subscribed packets (may take tens of seconds).

**Docker image:** the build applies [`docker/patches/sch_lab-hk-schedule.patch`](../docker/patches/sch_lab-hk-schedule.patch) so **SCH_LAB** requests **ES** and **TO_LAB** housekeeping (upstream `sch_lab` ships an empty schedule table otherwise). Without that patch, enable TO_LAB output but see no HK on UDP.

Automated check:

```bash
python3 scripts/verify_live_telemetry_no_mock.py
BRIDGE_HTTP_BASE=http://127.0.0.1:8080 python3 scripts/verify_live_telemetry_no_mock.py
python3 scripts/verify_live_telemetry_no_mock.py --check-docker-log
python3 scripts/verify_live_telemetry_no_mock.py --check-docker-log --require-both
```

Use **`--require-both`** only if you need **`es_hk_v1`** and **`to_lab_hk_v1`** in one run (TO_LAB HK may lag or depend on mission parsers).

## Troubleshooting

| Symptom | Check |
|--------|--------|
| UI shows “Disconnected” for telemetry | WebSocket blocked; ensure dev proxy has `ws: true` for `/api`. Same origin as HTTP. |
| No UDP packets | Firewall; wrong `BRIDGE_TLM_BIND`; TO_LAB not sending (expected until configured). |
| TO_LAB sends to wrong UDP port | Defaults use **2234** for both; if you change mission **`TO_LAB_MISSION_TLM_PORT`**, set **`BRIDGE_TLM_BIND`** to the same host:port. |
| `parse_error` only | Datagram size/layout differs from ES HK v1 (12 + 168 bytes LE); compare mission headers. |
| Logs | On start: `telemetry UDP listening on ...` from `bridge-server`. |

## Code quality commands

**Rust:** `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `cargo llvm-cov --all-targets --fail-under-lines 90`.

**bridge-ui:** `npm run lint`, `npm run lint:fix`, `npm run test:coverage` (line threshold **90%** in [vite.config.ts](../bridge-ui/vite.config.ts)).
