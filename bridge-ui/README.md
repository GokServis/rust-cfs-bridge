# bridge-ui

Web UI for the **rust-cfs-bridge** stack. It loads the command dictionary from **`bridge-server`** (`GET /api/commands`) and sends JSON bodies to **`POST /api/send`**. The server turns that JSON into CCSDS wire format and UDP to CI_LAB; see [rust-bridge/README.md](../rust-bridge/README.md).

## Which URL?

| How you run things | Open in the browser |
|--------------------|---------------------|
| **`docker compose up`** (UI baked into the image) | **`http://127.0.0.1:8080`** — `bridge-server` serves the built **`dist/`** and `/api`. There is **no** Vite process in the container, so **`http://localhost:5173` does not apply**. |
| **Local UI dev** (`npm run dev` on your machine) | **`http://localhost:5173`** (or whatever Vite prints). You must still run **`bridge-server`** (Docker or `cargo run --bin bridge-server`) so `/api` can be proxied to port **8080**. |

## Prerequisites

- **Node.js 20** (matches the Docker image NodeSource setup).

## Install

```bash
npm ci
```

## Development

With **`bridge-server`** running (default `http://127.0.0.1:8080`), start Vite; it proxies **`/api`** to that host (see `vite.config.ts`).

```bash
npm run dev
```

Open the URL Vite prints (usually `http://localhost:5173`).

## Production build

```bash
npm run build
```

Output is under **`dist/`**. Point **`bridge-server`** at it with **`BRIDGE_STATIC_DIR`** (Docker sets `BRIDGE_STATIC_DIR=/app/bridge-ui/dist`).

## Lint and test

```bash
npm run lint
npm run test
npm run test:coverage
```

Coverage is enforced at **≥80% lines** on `src/` (see `vite.config.ts` and CI).

## API contract

The UI follows the JSON shapes accepted by **`SpaceCommand::from_json`** in the Rust crate: named dictionary commands (`command`, `sequence_count`, optional hex `payload`) or legacy `apid` + hex `payload`. The browser never implements CCSDS framing; only **`bridge-server`** does.
