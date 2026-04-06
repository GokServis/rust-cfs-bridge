# rust-bridge

Rust library and binary that sends CCSDS-style packets to cFS over UDP (`std::net::UdpSocket`), typically to `127.0.0.1` on the port CI_LAB listens on (for example **1234** in the sample mission). A small **command dictionary** maps named commands (such as `CMD_HEARTBEAT`) to wire APID, payload length, and optional hex payload overrides; the library builds headers, payload, and CRC to match **bridge_reader** on the cFS side.

## Build

```bash
cargo build
cargo build --release
```

The Docker image builds this crate in release mode as part of the image build.

## Lint, test, and coverage

From this directory:

```bash
# One-shot: rustfmt, clippy (-D warnings), tests, library coverage (≥90% lines)
./check.sh
```

Individual steps:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

**Line coverage (goal: 90%+ on the library)** uses [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov). The thin `main.rs` binary is not exercised by unit tests, so coverage is measured for the **library only** (`--lib`), which includes CCSDS logic and `UdpSender`.

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
./coverage.sh              # same as: cargo llvm-cov --lib --fail-under-lines 90
# Optional: cargo llvm-cov --lib --html --open
```

## Layout

- `src/lib.rs` — CCSDS packet + JSON + unit tests.
- `src/udp.rs` — UDP sender (also covered by a loopback unit test).
- `src/main.rs` — binary that sends a dictionary-resolved packet (default heartbeat) to `127.0.0.1:1234`, then exits after a short delay so cFS can process it when run under Docker.
- `Cargo.toml` — dependencies and Rust edition (`2021`).
- `check.sh` — **fmt**, **clippy**, **test**, and **coverage** (library ≥90% lines).
- `coverage.sh` — coverage only (`cargo llvm-cov --lib --fail-under-lines 90`).

## Running in Docker

The container entrypoint runs `rust-bridge/target/release/rust-bridge` after starting cFS `core-cpu1`. Paths inside the image assume `/app` as the project root.
