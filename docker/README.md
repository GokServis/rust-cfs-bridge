# Docker image and Compose

Repository: [GokServis/rust-cfs-bridge](https://github.com/GokServis/rust-cfs-bridge).

Build context is the **repository root** (so `cfs/`, `rust-bridge/`, and `bridge-ui/` are available).

## Layout

| Image / service | Role |
|-----------------|------|
| **`rust-cfs-bridge:local`** (from [`Dockerfile`](Dockerfile)) | Builds cFS, **Node** (for `bridge-ui/dist` baked into the image), and **Rust** `bridge-server`. Used by **`bridge-server`** and **`cfs`** Compose services. |
| **bridge-ui** (from [`Dockerfile.bridge-ui`](Dockerfile.bridge-ui)) | Multi-stage **Node** build → **nginx** serving **`dist/`** on **:8080**, reverse-proxying **`/api`** and **`/health`** to **`127.0.0.1:8081`**. |

Default **`docker compose up`** starts:

1. **`bridge-server`** — [`entrypoint-bridge.sh`](entrypoint-bridge.sh): **`BRIDGE_HTTP_BIND=127.0.0.1:8081`**, no `BRIDGE_STATIC_DIR` (API + WebSocket only; telemetry UDP default **`127.0.0.1:2234`**).
2. **`bridge-ui`** — nginx on **http://127.0.0.1:8080** (SPA + proxy to the bridge).

Optional cFS:

```bash
docker compose --profile cfs up --build
```

This adds **`cfs`**: [`entrypoint-cfs.sh`](entrypoint-cfs.sh) runs **`core-cpu1`** (foreground with **`tee`** to **`/app/cfs-cpu1.log`**). **Privileged** is required only for this service (mqueue limits).

## Legacy monolithic entrypoint

[`entrypoint.sh`](entrypoint.sh) still runs **cFS** in the background, then **`exec` bridge-server** with **`BRIDGE_STATIC_DIR=/app/bridge-ui/dist`** so a single process serves **http://127.0.0.1:8080** (API + UI). The **Dockerfile** `ENTRYPOINT` remains this script for **`docker run`** without Compose overrides.

## Dockerfile (main)

- **Base:** `ubuntu:22.04`
- **Packages:** `build-essential`, `cmake`, `git`, `python3`, `curl`
- **Rust:** `rustup` stable
- **cFS:** patch + `make` / `make install` (see inline comments)
- **Node.js 20:** `bridge-ui` **`npm ci`** + **`npm run build`**
- **Rust:** `cargo build --release` in `rust-bridge/`

## Compose files

| File | Use |
|------|-----|
| [`../docker-compose.yml`](../docker-compose.yml) | Default: **bridge-server** + **bridge-ui**; **`cfs`** with **`--profile cfs`**. Host network. |
| [`../docker-compose.dev.yml`](../docker-compose.dev.yml) | Merge with `docker-compose.yml`: bind-mount **`.` → `/app`** on **bridge-server** and **cfs** for live editing. |

**Direct API (debug):** **`http://127.0.0.1:8081`** — **User-facing UI:** **`http://127.0.0.1:8080`** (through nginx).

## Manual build

```bash
docker build -f docker/Dockerfile -t rust-cfs-bridge:local .
docker build -f docker/Dockerfile.bridge-ui -t rust-cfs-bridge-ui:local .
```

## Logs and verification

| Service | What to expect |
|---------|----------------|
| **bridge-server** | `listening on http://127.0.0.1:8081`; `telemetry UDP listening on 127.0.0.1:2234` (or `BRIDGE_TLM_BIND`). |
| **bridge-ui** | nginx access logs; **`GET /api/health`** via **8080** proxies to bridge. |
| **cfs** (profile) | **core-cpu1** boot; **BRIDGE_READER** subscription lines; **`/app/cfs-cpu1.log`** inside the container. |

Manual check: open **http://127.0.0.1:8080**, confirm commands and telemetry. With **cfs** running, send **CMD_HEARTBEAT** / **CMD_PING** and match **bridge_reader** lines in **`docker compose --profile cfs logs -f cfs`**.

Telemetry mock (no cFS): `docker exec -it rust-cfs-bridge-server python3 /app/scripts/mock_es_hk_udp.py` — see [docs/TELEMETRY.md](../docs/TELEMETRY.md).
