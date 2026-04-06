# Docker image

Repository: [GokServis/rust-cfs-bridge](https://github.com/GokServis/rust-cfs-bridge).

Build context is the **repository root** (so `cfs/` and `rust-bridge/` are available). The compose files reference `docker/Dockerfile` from that context.

## Dockerfile

- **Base:** `ubuntu:22.04`
- **Packages:** `build-essential`, `cmake`, `git`, `python3`, `curl`
- **Rust:** installed with `rustup` (stable), on `PATH` as `/root/.cargo/bin`
- **cFS:** in `/app/cfs`, copies `cfe/cmake/Makefile.sample` → `Makefile` and `sample_defs`, then:
  - `make BUILDTYPE=release prep`
  - `make` and `make install`
- **Environment for cFS:** `SIMULATION=native` (host 64-bit GCC on amd64), `OMIT_DEPRECATED=true`
- **Node.js 20:** installed from NodeSource so **`bridge-ui`** can be built (`npm ci`, `npm run build` → `/app/bridge-ui/dist`). See [bridge-ui/README.md](../bridge-ui/README.md).
- **Rust:** `cargo build --release` in `/app/rust-bridge` (produces **`bridge-server`** and the one-shot **`rust-bridge`** binary).

## Entrypoint

[entrypoint.sh](entrypoint.sh) runs:

1. Raises **`fs.mqueue.msg_max`** (for example to **256**) when possible. cFE Software Bus pipes use POSIX message queues; the default limit is often too low. This needs a **privileged** container (see [docker-compose.yml](../docker-compose.yml)); avoid `ipc: host` with a tiny host `msg_max` unless you raise it on the host.
2. `cd /app/cfs/build/exe/cpu1`
3. `./core-cpu1` in the background, with **stdout/stderr** copied to **`/app/cfs-cpu1.log`** via `tee` so logs are visible from `docker compose logs` as well as on disk in the container.
4. **`sleep 2`** so CI_LAB and **bridge_reader** finish registration before the Rust bridge sends UDP.
5. `exec /app/rust-bridge/target/release/bridge-server` in the foreground, with **`BRIDGE_STATIC_DIR=/app/bridge-ui/dist`** and **`BRIDGE_TLM_BIND`** defaulting to **`127.0.0.1:5001`** (telemetry UDP) so the web UI is served on **http://127.0.0.1:8080** (API under **`/api`**, WebSocket **`/api/tlm/ws`**). The process runs until **SIGINT** (for example **Ctrl+C** or `docker compose stop`).

cFS expects to be started from `build/exe/cpu1` so it can find its startup script and loadable modules next to the executable.

## Compose

| File | Use |
|------|-----|
| `../docker-compose.yml` | Default: host network, no volume over `/app`; runs pre-built image contents. |
| `../docker-compose.dev.yml` | Bind mount `.` → `/app`; build cFS and Rust inside the container before `up`. |

Both use `network_mode: host` for straightforward UDP between the bridge and cFS lab apps on the host loopback. The default service is **`privileged: true`** so the entrypoint can adjust **mqueue** limits inside the container.

## Manual build

From the repository root:

```bash
docker build -f docker/Dockerfile -t rust-cfs-bridge:local .
```

## Logs and verification

After `docker compose up`, use `docker compose logs -f` (or read `/app/cfs-cpu1.log` inside the container) and look for:

| Component | What to expect |
|-----------|----------------|
| **core-cpu1** | cFS boots; **BRIDGE_READER** prints `Initialized … subscribed to 2 bridge MsgId(s): 0x18F0 0x18F1` (two Software Bus topics for the bridge dictionary). |
| **CI_LAB** | No repeated ingest errors when sending commands from the UI. |
| **bridge-server** | HTTP access to `/api/commands` and successful `POST /api/send` (returns `bytes_sent` / `wire_length`); stderr line **`telemetry UDP listening on 127.0.0.1:5001`** (or `BRIDGE_TLM_BIND`). |
| **BRIDGE_READER** | For each sent packet: `Bridge Reader: SB MsgId 0xXXXX wire APID 0xYYY payload: [ … ]` — **CMD_HEARTBEAT** uses MsgId `0x18F0` and wire APID `0x006`; **CMD_PING** uses MsgId `0x18F1` and wire APID `0x007`. |

Manual check: open **http://127.0.0.1:8080**, confirm two commands in the dropdown, send each and match the MsgId/APID lines above.

Telemetry (downlink) validation without changing cFS: [docs/TELEMETRY.md](../docs/TELEMETRY.md) — run `python3 /app/scripts/mock_es_hk_udp.py` inside the container and confirm the UI **Telemetry overview** updates. Uplink dictionary script: `python3 /app/scripts/verify_uplink_dictionary.py`.
