# rust-bridge

Small Rust binary that will eventually talk to cFS over the network (for example `std::net::UdpSocket` on `127.0.0.1`, with ports aligned to CI_LAB / TO_LAB in your mission `sample_defs`).

## Build

```bash
cargo build
cargo build --release
```

The Docker image builds this crate in release mode as part of the image build.

## Layout

- `src/main.rs` — entry point (currently a minimal hello-world and notes for the next UDP/CCSDS step).
- `Cargo.toml` — package metadata and edition (Rust 2024 if supported by your toolchain).

## Running in Docker

The container entrypoint runs `rust-bridge/target/release/rust-bridge` after starting cFS `core-cpu1`. Paths inside the image assume `/app` as the project root.
