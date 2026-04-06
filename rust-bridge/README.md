# rust-bridge

Part of [rust-cfs-bridge](https://github.com/GokServis/rust-cfs-bridge).

Rust library and binaries that send CCSDS-style packets to cFS over UDP (`std::net::UdpSocket`), typically to `127.0.0.1` on the port CI_LAB listens on (for example **1234** in the sample mission). A **command dictionary** maps named commands (for example `CMD_HEARTBEAT`, `CMD_PING`) to on-wire CCSDS APID, matching Software Bus MsgId (used by CI_LAB after ingest), payload length, and optional hex payload overrides; the library builds headers, payload, and CRC to match **bridge_reader** on the cFS side.

### Migration / compatibility

- **`CMD_HEARTBEAT`** remains **APID `0x006`** on the wire and **SB MsgId `0x18F0`** after CI_LAB — unchanged for existing scripts and UI defaults.
- **`CMD_PING`** uses **APID `0x007`** and **MsgId `0x18F1`**. Numeric values are defined in both Rust (`bridge_*` constants in `lib.rs`) and [`cfs/apps/bridge_reader/fsw/inc/bridge_reader_mission_ids.h`](../cfs/apps/bridge_reader/fsw/inc/bridge_reader_mission_ids.h); keep them aligned when adding commands.
- **Legacy JSON** `{ "apid", "sequence_count", "payload" }` is unchanged. Only APIDs allowlisted in CI_LAB are accepted as bridge wire format; others follow the normal CI_LAB path.

## Binaries

| Binary | Role |
|--------|------|
| `rust-bridge` | One-shot: sends a sample dictionary packet (default heartbeat) then exits (useful for smoke tests). |
| `bridge-server` | Long-lived HTTP server: `GET /api/commands`, `POST /api/send`, `GET /api/health`, WebSocket **`GET /api/tlm/ws`** for telemetry JSON; optional static UI when `BRIDGE_STATIC_DIR` points at a built [bridge-ui](../bridge-ui) `dist/` tree (see [bridge-ui/README.md](../bridge-ui/README.md)). |

Environment variables for **`bridge-server`**:

| Variable | Default | Meaning |
|----------|---------|---------|
| `BRIDGE_HTTP_BIND` | `127.0.0.1:8080` | TCP listen address |
| `BRIDGE_UDP_TARGET` | `127.0.0.1:1234` | Connected UDP destination (CI_LAB) |
| `BRIDGE_TLM_BIND` | `127.0.0.1:5001` | UDP bind address for incoming telemetry (TO_LAB / mock) |
| `BRIDGE_STATIC_DIR` | (unset) | If set, serve this directory as static files (SPA fallback to `index.html`) |

Telemetry flow, mock script, and troubleshooting: [docs/TELEMETRY.md](../docs/TELEMETRY.md).

## Build

```bash
cargo build
cargo build --release
```

The Docker image builds this crate in release mode as part of the image build.

## Lint, test, and coverage

From this directory:

```bash
# One-shot: rustfmt, clippy (-D warnings), tests, line coverage (≥90% lines, all targets)
./check.sh
```

Individual steps:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Auto-fix (optional): `cargo fmt --all`, `cargo clippy --all-targets --all-features --fix -- -D warnings`.

**Line coverage (≥90% on all targets)** uses [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) (library, `bridge-server`, and the thin `rust-bridge` binary entrypoints are included in the aggregate gate).

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
./coverage.sh              # same as: cargo llvm-cov --all-targets --fail-under-lines 90
# Optional: cargo llvm-cov --all-targets --html --open
```

## Layout

- `src/lib.rs` — CCSDS packet + JSON + command dictionary metadata + unit tests.
- `src/udp.rs` — UDP sender (also covered by a loopback unit test).
- `src/server.rs` — Axum HTTP API + WebSocket telemetry (behind the `server` feature, on by default).
- `src/tlm/` — Telemetry UDP task, CCSDS primary + CFE ES HK parsing, `TlmEvent` JSON.
- `src/main.rs` — one-shot binary for smoke tests.
- `src/bin/bridge_server.rs` — `bridge-server` entrypoint.
- `Cargo.toml` — dependencies and Rust edition (`2021`).
- `check.sh` — **fmt**, **clippy**, **test**, and **coverage** (≥90% lines, all targets).
- `coverage.sh` — coverage only (`cargo llvm-cov --all-targets --fail-under-lines 90`).

## Running in Docker

The container entrypoint runs `bridge-server` after starting cFS `core-cpu1`, with `BRIDGE_STATIC_DIR=/app/bridge-ui/dist` so the web UI is available on port **8080** (host network). Paths inside the image assume `/app` as the project root.

## Local dev (UI + API)

1. Start cFS (for example Docker) so CI_LAB listens on UDP **1234**.
2. From `rust-bridge/`: `cargo run --release --bin bridge-server` (or `BRIDGE_UDP_TARGET=127.0.0.1:1234 BRIDGE_TLM_BIND=127.0.0.1:5001 cargo run --bin bridge-server`).
3. From `bridge-ui/`: `npm install && npm run dev` — Vite proxies `/api` to `http://127.0.0.1:8080` (WebSocket upgrade for telemetry); use the URL Vite prints (**`:5173`**). If you use **only** `docker compose up`, skip Vite and open **`http://127.0.0.1:8080`** instead. Details: [bridge-ui/README.md](../bridge-ui/README.md).

## Pre-commit

From the repo root, [pre-commit](https://pre-commit.com/) can run the same Rust and `bridge-ui` checks as CI (see [`.pre-commit-config.yaml`](../.pre-commit-config.yaml)):

```bash
pip install pre-commit
pre-commit install
```
