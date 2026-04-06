# rust-cfs-bridge

Rust and web front end that talk to NASA [cFS](https://github.com/nasa/cFS) over UDP: CCSDS-style packets into CI_LAB and the Software Bus (for example **bridge_reader**). The `cfs/` tree is a **git submodule**; application code lives in `rust-bridge/` and `bridge-ui/`.

## Run with Docker

**Needs:** Linux (Compose uses `network_mode: host`), Docker, and this repo **with all submodules** checked out.

```bash
git clone --recurse-submodules https://github.com/GokServis/rust-cfs-bridge.git
cd rust-cfs-bridge
docker compose build
docker compose up
```

Then open **`http://127.0.0.1:8080`** (nginx serves the UI and proxies **`/api`** to **bridge-server** on **:8081**).

**Uplink and TO_LAB commands need CI_LAB.** The default stack (**`docker compose up`** / **`make up`**) does **not** start cFS, so nothing listens on **`127.0.0.1:1234`** and command sends can fail with “UDP target not reachable”. For the full flight stack, use **`docker compose --profile cfs up --build`** or **`make up-cfs`**. Telemetry WebSocket and the UI still load without cFS; use **`scripts/mock_es_hk_udp.py`** to exercise downlink without cFS ([docs/TELEMETRY.md](docs/TELEMETRY.md)).

More detail: [docker/README.md](docker/README.md), [bridge-ui/README.md](bridge-ui/README.md).

| File | Role |
|------|------|
| `docker-compose.yml` | **bridge-server** + **bridge-ui** (default); **`cfs`** with **`--profile cfs`** |
| `docker-compose.dev.yml` | Merge with the file above: bind-mount **`.:/app`** for dev ([docker/README.md](docker/README.md)) |
| `Makefile` | Shortcuts: **`make up`**, **`make up-cfs`**, **`make down`**, **`make logs-bridge`**, … |

## Repository

| Path | Role |
|------|------|
| [`cfs/`](cfs/) | cFS bundle (nested submodules: `ci_lab`, `cfe`, …) |
| [`rust-bridge/`](rust-bridge/README.md) | Rust library and `bridge-server` |
| [`bridge-ui/`](bridge-ui/README.md) | Web UI |
| [`docker/`](docker/README.md) | Dockerfile and runtime entrypoint |
| [`scripts/`](scripts/) | Helpers: `mock_es_hk_udp.py`, `verify_uplink_dictionary.py`, [ensure-github-forks.sh](scripts/ensure-github-forks.sh) |
| [`docs/TELEMETRY.md`](docs/TELEMETRY.md) | Telemetry UDP / WebSocket / troubleshooting |
| [`docs/AVAILABLE_TELEMETRY.md`](docs/AVAILABLE_TELEMETRY.md) | cFS topic inventory, TO_LAB path, Rust parser matrix |

If you already cloned without submodules: `git submodule update --init --recursive`. Remotes point at the [GokServis](https://github.com/GokServis) organization’s forks so commits are fetchable without NASA write access.

## License

Apache 2.0 for files in this repo that we authored; see [LICENSE](LICENSE). The `cfs/` submodule follows its own licenses ([cfs/LICENSE](cfs/LICENSE)).
