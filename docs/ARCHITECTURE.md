# Architecture

This repository connects a **Rust HTTP/UDP bridge**, an optional **web UI**, and a **NASA cFS** runtime: telecommands leave the ground stack as **CCSDS-style frames with a CRC-16 trailer**, enter **CI_LAB** over UDP, and are republished on the **cFE Software Bus** for applications such as **bridge_reader**.

## Major components

| Layer | Location | Role |
|--------|----------|------|
| **bridge-server** | `rust-bridge/` | Axum HTTP API (`/api`), builds wire packets, sends UDP to CI_LAB. |
| **bridge-ui** | `bridge-ui/` | Static SPA (Vite build). With Compose, **nginx** serves **`dist/`** on **:8080** and proxies **`/api`** to **bridge-server** on **:8081**; legacy **`entrypoint.sh`** still serves the SPA from **`bridge-server`** when `BRIDGE_STATIC_DIR` is set. |
| **cFS bundle** | `cfs/` (git submodule) | cFE, OSAL, PSP, lab apps (`ci_lab`, `to_lab`, …) and custom **bridge_reader**. |
| **Docker** | `docker/` | Main **Dockerfile** builds cFS, Rust, and UI assets; **Dockerfile.bridge-ui** builds the nginx front end. Compose runs **bridge-server** + **bridge-ui** by default; **`cfs`** is optional (`--profile cfs`). |

## Repository layout

- **`rust-bridge/`** — Library (`CcsdsPacket`, `SpaceCommand`, CRC-16, `UdpSender`) and `bridge-server` binary.
- **`cfs/`** — Full cFS mission tree; includes **bridge_reader** (Software Bus subscriber) and **ci_lab** (UDP ingest).
- **`bridge-ui/`** — TypeScript UI calling `/api/commands` and `POST /api/send`.
- **`docker/`** — `Dockerfile`, `entrypoint.sh` (cFS boot + `bridge-server`).

## Runtime (Docker / host)

Compose uses **`network_mode: host`** so the bridge and CI_LAB share loopback UDP without extra port mapping.

- **Default stack:** **bridge-server** listens on **`127.0.0.1:8081`** (HTTP + WebSocket); **nginx** (bridge-ui) listens on **`127.0.0.1:8080`** and proxies **`/api`** to the bridge.
- **Optional `cfs` service** (`docker compose --profile cfs`): raises **`fs.mqueue.msg_max`** (**privileged**), runs **`core-cpu1`** from `cfs/build/exe/cpu1`.
- **Legacy single-container** [`entrypoint.sh`](../docker/entrypoint.sh): starts **cFS** in the background, waits, then **`bridge-server`** with **`BRIDGE_STATIC_DIR`** (UI on **8080** in one process).

Default **`BRIDGE_HTTP_BIND`** in Rust: **`127.0.0.1:8080`** if unset; Compose sets **8081** for **bridge-server**. Default UDP target: **`127.0.0.1:1234`** (CI_LAB listen port).

## cFS applications relevant to the bridge

- **CI_LAB** — Listens on UDP, decodes CCSDS-style frames, maps **wire APID** → **Software Bus MsgId**, publishes SB messages.
- **bridge_reader** — Subscribes to the bridge MsgIds (`0x18F0`, `0x18F1` in the stock dictionary), validates CRC, logs payload bytes for verification.

## Configuration sources of truth

- **Rust ↔ C alignment** — Wire APID, SB MsgId, and payload rules are defined in `rust-bridge/src/lib.rs` and must stay consistent with `cfs/apps/bridge_reader/fsw/inc/bridge_reader_mission_ids.h` and CI_LAB routing tables.

## Related documentation

- [MESSAGE_FLOW.md](MESSAGE_FLOW.md) — End-to-end path from JSON to Software Bus logs.
- [docker/README.md](../docker/README.md) — Image contents, build steps, and log expectations.
