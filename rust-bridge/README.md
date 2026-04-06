# rust-bridge

Rust library and binaries that send CCSDS-style packets to cFS over UDP (`std::net::UdpSocket`), typically to `127.0.0.1` on the port CI_LAB listens on (for example **1234** in the sample mission). A small **command dictionary** maps named commands (such as `CMD_HEARTBEAT`) to wire APID, payload length, and optional hex payload overrides; the library builds headers, payload, and CRC to match **bridge_reader** on the cFS side.

## Binaries

| Binary | Role |
|--------|------|
| `rust-bridge` | One-shot: sends a sample dictionary packet (default heartbeat) then exits (useful for smoke tests). |
| `bridge-server` | Long-lived HTTP server: `GET /api/commands`, `POST /api/send`, `GET /api/health`; optional static UI when `BRIDGE_STATIC_DIR` points at a built [bridge-ui](../bridge-ui) `dist/` tree. |

Environment variables for **`bridge-server`**:

| Variable | Default | Meaning |
|----------|---------|---------|
| `BRIDGE_HTTP_BIND` | `127.0.0.1:8080` | TCP listen address |
| `BRIDGE_UDP_TARGET` | `127.0.0.1:1234` | Connected UDP destination (CI_LAB) |
| `BRIDGE_STATIC_DIR` | (unset) | If set, serve this directory as static files (SPA fallback to `index.html`) |

## Build

```bash
cargo build
cargo build --release
```

The Docker image builds this crate in release mode as part of the image build.

## Lint, test, and coverage

From this directory:

```bash
# One-shot: rustfmt, clippy (-D warnings), tests, line coverage (≥80% lines, all targets)
./check.sh
```

Individual steps:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

**Line coverage (≥80% on all targets)** uses [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) (library, `bridge-server`, and the thin `rust-bridge` binary entrypoints are included in the aggregate gate).

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
./coverage.sh              # same as: cargo llvm-cov --all-targets --fail-under-lines 80
# Optional: cargo llvm-cov --all-targets --html --open
```

## Layout

- `src/lib.rs` — CCSDS packet + JSON + command dictionary metadata + unit tests.
- `src/udp.rs` — UDP sender (also covered by a loopback unit test).
- `src/server.rs` — Axum HTTP API (behind the `server` feature, on by default).
- `src/main.rs` — one-shot binary for smoke tests.
- `src/bin/bridge_server.rs` — `bridge-server` entrypoint.
- `Cargo.toml` — dependencies and Rust edition (`2021`).
- `check.sh` — **fmt**, **clippy**, **test**, and **coverage** (≥80% lines, all targets).
- `coverage.sh` — coverage only (`cargo llvm-cov --all-targets --fail-under-lines 80`).

## Running in Docker

The container entrypoint runs `bridge-server` after starting cFS `core-cpu1`, with `BRIDGE_STATIC_DIR=/app/bridge-ui/dist` so the web UI is available on port **8080** (host network). Paths inside the image assume `/app` as the project root.

## Local dev (UI + API)

1. Start cFS (for example Docker) so CI_LAB listens on UDP **1234**.
2. From `rust-bridge/`: `cargo run --release --bin bridge-server` (or `BRIDGE_UDP_TARGET=127.0.0.1:1234 cargo run --bin bridge-server`).
3. From `bridge-ui/`: `npm install && npm run dev` — Vite proxies `/api` to `http://127.0.0.1:8080`.

## Pre-commit

From the repo root, [pre-commit](https://pre-commit.com/) can run the same Rust and `bridge-ui` checks as CI (see [`.pre-commit-config.yaml`](../.pre-commit-config.yaml)):

```bash
pip install pre-commit
pre-commit install
```
