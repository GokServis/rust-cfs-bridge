# Architecture

This repository connects a **Rust HTTP/UDP bridge**, an optional **web UI**, and a **NASA cFS** runtime: telecommands leave the ground stack as **CCSDS-style frames with a CRC-16 trailer**, enter **CI_LAB** over UDP, and are republished on the **cFE Software Bus** for applications such as **bridge_reader**.

## Major components

| Layer | Location | Role |
|--------|----------|------|
| **bridge-server** | `rust-bridge/` | Axum HTTP API (`/api`), builds wire packets, sends UDP to CI_LAB. |
| **bridge-ui** | `bridge-ui/` | Static SPA (Vite build) served by `bridge-server` when `BRIDGE_STATIC_DIR` is set. |
| **cFS bundle** | `cfs/` (git submodule) | cFE, OSAL, PSP, lab apps (`ci_lab`, `to_lab`, …) and custom **bridge_reader**. |
| **Docker image** | `docker/` | Builds cFS, Rust, and UI; entrypoint runs `core-cpu1` then `bridge-server`. |

## Repository layout

- **`rust-bridge/`** — Library (`CcsdsPacket`, `SpaceCommand`, CRC-16, `UdpSender`) and `bridge-server` binary.
- **`cfs/`** — Full cFS mission tree; includes **bridge_reader** (Software Bus subscriber) and **ci_lab** (UDP ingest).
- **`bridge-ui/`** — TypeScript UI calling `/api/commands` and `POST /api/send`.
- **`docker/`** — `Dockerfile`, `entrypoint.sh` (cFS boot + `bridge-server`).

## Runtime (Docker / host)

Compose uses **`network_mode: host`** so the bridge and CI_LAB share loopback UDP without extra port mapping. The entrypoint:

1. Optionally raises **`fs.mqueue.msg_max`** (cFE Software Bus pipes use POSIX message queues; container needs **`privileged: true`** in compose).
2. Starts **`core-cpu1`** from `cfs/build/exe/cpu1` (cFS expects to run from that directory).
3. Waits briefly so CI_LAB and **bridge_reader** finish registration.
4. Runs **`bridge-server`** with `BRIDGE_STATIC_DIR` pointing at the built UI.

Default HTTP bind: **`127.0.0.1:8080`**. Default UDP target: **`127.0.0.1:1234`** (CI_LAB listen port).

## cFS applications relevant to the bridge

- **CI_LAB** — Listens on UDP, decodes CCSDS-style frames, maps **wire APID** → **Software Bus MsgId**, publishes SB messages.
- **bridge_reader** — Subscribes to the bridge MsgIds (`0x18F0`, `0x18F1` in the stock dictionary), validates CRC, logs payload bytes for verification.

## Configuration sources of truth

- **Rust ↔ C alignment** — Wire APID, SB MsgId, and payload rules are defined in `rust-bridge/src/lib.rs` and must stay consistent with `cfs/apps/bridge_reader/fsw/inc/bridge_reader_mission_ids.h` and CI_LAB routing tables.

## Related documentation

- [MESSAGE_FLOW.md](MESSAGE_FLOW.md) — End-to-end path from JSON to Software Bus logs.
- [docker/README.md](../docker/README.md) — Image contents, build steps, and log expectations.
