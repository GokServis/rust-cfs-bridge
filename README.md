# rust-cfs-bridge

Rust and web front end that talk to NASA [cFS](https://github.com/nasa/cFS) over UDP: CCSDS-style packets into CI_LAB and the Software Bus (for example **bridge_reader**). The `cfs/` tree is a **git submodule**; application code lives in `rust-bridge/` and `bridge-ui/`.

## Run with Docker

**Needs:** Linux (Compose uses `network_mode: host`), Docker, and this repo **with all submodules** checked out.

```bash
git clone --recurse-submodules https://github.com/macaris64/rust-cfs-bridge.git
cd rust-cfs-bridge
docker compose build
docker compose up
```

Then open **`http://127.0.0.1:8080`**. More detail (Vite dev on **:5173**, image layout, dev compose): [docker/README.md](docker/README.md), [bridge-ui/README.md](bridge-ui/README.md).

| File | Role |
|------|------|
| `docker-compose.yml` | Default: baked image, no bind mount |
| `docker-compose.dev.yml` | Bind-mount repo; build inside container ([docker/README.md](docker/README.md)) |

## Repository

| Path | Role |
|------|------|
| [`cfs/`](cfs/) | cFS bundle (nested submodules: `ci_lab`, `cfe`, …) |
| [`rust-bridge/`](rust-bridge/README.md) | Rust library and `bridge-server` |
| [`bridge-ui/`](bridge-ui/README.md) | Web UI |
| [`docker/`](docker/README.md) | Dockerfile and runtime entrypoint |
| [`scripts/`](scripts/ensure-github-forks.sh) | Helpers (e.g. GitHub forks for submodules) |

If you already cloned without submodules: `git submodule update --init --recursive`. Remotes point at [macaris64](https://github.com/macaris64) forks so commits are fetchable without NASA write access.

## License

Apache 2.0 for files in this repo that we authored; see [LICENSE](LICENSE). The `cfs/` submodule follows its own licenses ([cfs/LICENSE](cfs/LICENSE)).
