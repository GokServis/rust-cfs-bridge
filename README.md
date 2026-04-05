# rust-cfs-bridge

Bridge between NASA [cFS](https://github.com/nasa/cFS) (core Flight System) and a Rust application. cFS lives under `cfs/` as a git submodule; the Rust side lives in `rust-bridge/`. The goal is to exchange CCSDS-style traffic with the Software Bus (for example via lab apps such as CI_LAB and TO_LAB) over UDP on the host network.

## Repository layout

| Path | Purpose |
|------|---------|
| `cfs/` | cFS bundle (submodule). Build with CMake/Make from this tree per upstream docs. |
| `rust-bridge/` | Rust binary; see [rust-bridge/README.md](rust-bridge/README.md). |
| `docker/` | Container image and entrypoint; see [docker/README.md](docker/README.md). |
| `docker-compose.yml` | Production-style run: no bind mount, uses binaries baked into the image. |
| `docker-compose.dev.yml` | Development: bind-mounts the repo; you build inside the container (or sync artifacts yourself). |

## Clone with submodules

```bash
git clone --recurse-submodules <repo-url>
# or after clone:
git submodule update --init --recursive
```

## cFS build note (64-bit native simulation)

Sample `targets.cmake` may list `i686-linux-gnu`, but a **native simulation** build uses `SIMULATION=native` (see NASA cFS README and CI). That selects the host compiler on Ubuntu 22.04 amd64 (x86_64) without an i686 cross toolchain. The Docker image sets `SIMULATION=native` for `make prep`.

## Docker quick start

From the repository root:

```bash
docker compose build
docker compose up
```

This uses `network_mode: host` so UDP to `127.0.0.1` matches cFS lab defaults. The container starts `core-cpu1` from `cfs/build/exe/cpu1/` (correct working directory for shared objects and startup files), then runs the Rust bridge in the foreground.

For live-mounted source trees, use [docker-compose.dev.yml](docker-compose.dev.yml) and follow the comments in that file to run `make prep`, `make`, `make install`, and `cargo build` inside the container first.

## License

See [LICENSE](LICENSE) if present; cFS and submodule licenses apply to their respective trees.
