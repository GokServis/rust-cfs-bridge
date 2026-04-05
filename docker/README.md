# Docker image

Build context is the **repository root** (so `cfs/` and `rust-bridge/` are available). The compose files reference `docker/Dockerfile` from that context.

## Dockerfile

- **Base:** `ubuntu:22.04`
- **Packages:** `build-essential`, `cmake`, `git`, `python3`, `curl`
- **Rust:** installed with `rustup` (stable), on `PATH` as `/root/.cargo/bin`
- **cFS:** in `/app/cfs`, copies `cfe/cmake/Makefile.sample` → `Makefile` and `sample_defs`, then:
  - `make BUILDTYPE=release prep`
  - `make` and `make install`
- **Environment for cFS:** `SIMULATION=native` (host 64-bit GCC on amd64), `OMIT_DEPRECATED=true`
- **Rust:** `cargo build --release` in `/app/rust-bridge`

## Entrypoint

[entrypoint.sh](entrypoint.sh) runs:

1. `cd /app/cfs/build/exe/cpu1`
2. `./core-cpu1` in the background
3. `exec /app/rust-bridge/target/release/rust-bridge` in the foreground

cFS expects to be started from `build/exe/cpu1` so it can find its startup script and loadable modules next to the executable.

## Compose

| File | Use |
|------|-----|
| `../docker-compose.yml` | Default: host network, no volume over `/app`; runs pre-built image contents. |
| `../docker-compose.dev.yml` | Bind mount `.` → `/app`; build cFS and Rust inside the container before `up`. |

Both use `network_mode: host` for straightforward UDP between the bridge and cFS lab apps on the host loopback.

## Manual build

From the repository root:

```bash
docker build -f docker/Dockerfile -t rust-cfs-bridge:local .
```
