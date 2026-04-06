# rust-cfs-bridge

Bridge between NASA [cFS](https://github.com/nasa/cFS) (core Flight System) and a Rust application. This repo tracks a **fork** of the cFS bundle under `cfs/` as a git submodule (see [Git submodules](#git-submodules-in-this-repo)); the Rust side lives in `rust-bridge/`. The goal is to exchange CCSDS-style traffic with the Software Bus (for example via lab apps such as CI_LAB, **bridge_reader**, and TO_LAB) over UDP on the host network.

## Quick start

You need the repository checked out **with submodules** (see below). On **Linux**, from the **repository root**, this is enough:

```bash
docker compose build
docker compose up
```

[docker-compose.yml](docker-compose.yml) uses `network_mode: host`; use a Linux host or VM (not Docker Desktop on Mac/Windows for this path). The image builds cFS, **bridge-ui**, and **rust-bridge**; at runtime you get **core-cpu1** and **bridge-server**. Open the web UI at **`http://127.0.0.1:8080`** (not port **5173** — that is only for a Vite dev server on the host; see [bridge-ui/README.md](bridge-ui/README.md)). Details: [docker/Dockerfile](docker/Dockerfile), [docker/README.md](docker/README.md).

For a bind-mounted dev tree, use [docker-compose.dev.yml](docker-compose.dev.yml) and [docker/README.md](docker/README.md).

## Git submodules in this repo

```bash
git clone --recurse-submodules https://github.com/macaris64/rust-cfs-bridge.git
cd rust-cfs-bridge
```

If you cloned without submodules: `git submodule update --init --recursive`.

A **git submodule** pins another repository at an **exact commit** inside a folder. Cloning only the parent repo leaves those folders empty until you initialize submodules. This project nests submodules: **rust-cfs-bridge** → **`cfs/`** (cFS bundle) → inside **`cfs/`**, further submodules such as **ci_lab** and **cfe**. You must use **`--recurse-submodules`** (or `git submodule update --init --recursive`) so Git checks out **all** of them; otherwise `cfs/` or its dependencies will be incomplete and builds will fail.

Submodule URLs in this project point at **forks** under [macaris64](https://github.com/macaris64) so anyone can **`git fetch`** the recorded commits without NASA write access:

| Submodule path | Remote (fetch) |
|----------------|----------------|
| `cfs/` | [macaris64/cFS](https://github.com/macaris64/cFS) |
| `cfs/apps/ci_lab` | [macaris64/ci_lab](https://github.com/macaris64/ci_lab) |
| `cfs/cfe` | [macaris64/cFE](https://github.com/macaris64/cFE) |

Upstream NASA repositories remain the conceptual baseline; these forks carry **bridge_reader** and related wiring. To create your own forks (for example after a clean reset), use the “Fork” button on [nasa/cFS](https://github.com/nasa/cFS), [nasa/ci_lab](https://github.com/nasa/ci_lab), and [nasa/cFE](https://github.com/nasa/cFE), or run [scripts/ensure-github-forks.sh](scripts/ensure-github-forks.sh) with `GITHUB_TOKEN` / `GH_TOKEN` set.

## Repository layout

| Path | Purpose |
|------|---------|
| `cfs/` | cFS bundle (submodule). Points at [macaris64/cFS](https://github.com/macaris64/cFS); nested submodules include **ci_lab** and **cfe** forks with bridge-related commits. Build with CMake/Make per cFS docs. |
| `rust-bridge/` | Rust library and `bridge-server`; see [rust-bridge/README.md](rust-bridge/README.md). |
| `bridge-ui/` | Web UI (Vite + React); see [bridge-ui/README.md](bridge-ui/README.md). |
| `docker/` | Container image and entrypoint; see [docker/README.md](docker/README.md). |
| `scripts/` | Helper scripts (for example [scripts/ensure-github-forks.sh](scripts/ensure-github-forks.sh) to fork upstream NASA repos via the GitHub API when `GITHUB_TOKEN` is set). |
| `docker-compose.yml` | Production-style run: no bind mount, uses binaries baked into the image. |
| `docker-compose.dev.yml` | Development: bind-mounts the repo; you build inside the container (or sync artifacts yourself). |

## cFS build note (64-bit native simulation)

Sample `targets.cmake` may list `i686-linux-gnu`, but a **native simulation** build uses `SIMULATION=native` (see NASA cFS README and CI). That selects the host compiler on Ubuntu 22.04 amd64 (x86_64) without an i686 cross toolchain. The Docker image sets `SIMULATION=native` for `make prep`.

## Optional: manual image build

```bash
docker build -f docker/Dockerfile -t rust-cfs-bridge:local .
```

## License

Original files in this repository (for example Docker assets, Compose files, `rust-bridge/`, `bridge-ui/`, and documentation) are licensed under the [Apache License 2.0](LICENSE).

The `cfs/` git submodule is third-party software; see [cfs/LICENSE](cfs/LICENSE) and the license files in submodule components for their terms.
