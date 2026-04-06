# rust-cfs-bridge

Bridge between NASA [cFS](https://github.com/nasa/cFS) (core Flight System) and a Rust application. This repo tracks a **fork** of the cFS bundle under `cfs/` as a git submodule (see below); the Rust side lives in `rust-bridge/`. The goal is to exchange CCSDS-style traffic with the Software Bus (for example via lab apps such as CI_LAB, **bridge_reader**, and TO_LAB) over UDP on the host network.

## Set up the project from scratch

You only need **Git** and **Docker** (with Compose) on the host. You do **not** need Rust or a cFS toolchain on the host if you run everything in the container using the steps in [How to run (Docker)](#how-to-run-docker).

1. **Clone this repository and every submodule** (see [Git submodules in this repo](#git-submodules-in-this-repo)):

   ```bash
   git clone --recurse-submodules https://github.com/macaris64/rust-cfs-bridge.git
   cd rust-cfs-bridge
   ```

   If you cloned without `--recurse-submodules`, fix it before continuing:

   ```bash
   git submodule update --init --recursive
   ```

2. **Run with Docker** (builds cFS and the Rust bridge inside the image—see [How to run (Docker)](#how-to-run-docker)):

   ```bash
   docker compose build
   docker compose up
   ```

3. **Confirm** you see cFS start, **BRIDGE_READER** / **bridge_reader** subscribe to the bridge MsgId, and the Rust process send UDP to CI_LAB; you should see a received packet line in the logs. See the Docker section for details.

For iterative development with a bind-mounted tree (rebuild inside the container), use [docker-compose.dev.yml](docker-compose.dev.yml) and the notes in [docker/README.md](docker/README.md).

## Git submodules in this repo

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

## How to run (Docker)

**Prerequisites:** [Docker](https://docs.docker.com/get-docker/) and Docker Compose (`docker compose`). **Linux** is the expected host: [docker-compose.yml](docker-compose.yml) uses `network_mode: host`, which behaves differently on Docker Desktop for Mac/Windows; use a Linux machine or VM for the path described here.

From the **repository root** (after a [recursive clone](#set-up-the-project-from-scratch)):

```bash
docker compose build
docker compose up
```

**What this does:** The image builds cFS (`make prep`, `make`, `make install`), the **bridge-ui** static bundle (`npm ci && npm run build`), and **`rust-bridge`** (`cargo build --release`) from the tree baked into the image—see [docker/Dockerfile](docker/Dockerfile) and [docker/README.md](docker/README.md). At runtime, [docker/entrypoint.sh](docker/entrypoint.sh) raises POSIX **mqueue** limits (needs **`privileged: true`** in Compose), starts **`core-cpu1`** from `cfs/build/exe/cpu1/`, streams its log to `/app/cfs-cpu1.log`, waits **2 seconds** for apps to register, then runs **`bridge-server`**, which listens on **http://127.0.0.1:8080** (API under `/api`, UI from `/app/bridge-ui/dist`) and sends CCSDS UDP datagrams to **127.0.0.1:1234** (CI_LAB’s default port) when you use the UI or API.

**What to look for in the logs:** cFS should pass Software Bus init without **`CFE_SB_CreatePipe`** / mqueue errors. You should see **bridge_reader** / **BRIDGE_READER** load and subscribe (for example MsgId **0x18F0**), then **`bridge-server: listening on http://…`**. Use **Send** in the browser or `POST /api/send` to trigger traffic; you should see **`Bridge Reader: Received valid packet`** (or similar) in the cFS log. The container keeps running until you stop it (**Ctrl+C**); **`bridge-server`** shuts down on SIGINT.

**Optional:** Open **http://127.0.0.1:8080** on the host (with `network_mode: host`) to use the web UI.

**Optional image tag:**

```bash
docker build -f docker/Dockerfile -t rust-cfs-bridge:local .
```

For a **bind-mounted** working tree and rebuilding inside the container, use **`docker compose -f docker-compose.dev.yml`** and follow [docker-compose.dev.yml](docker-compose.dev.yml) and [docker/README.md](docker/README.md).

## License

Original files in this repository (for example Docker assets, Compose files, `rust-bridge/`, `bridge-ui/`, and documentation) are licensed under the [Apache License 2.0](LICENSE).

The `cfs/` git submodule is third-party software; see [cfs/LICENSE](cfs/LICENSE) and the license files in submodule components for their terms.
