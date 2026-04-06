# bridge-ui

Part of [rust-cfs-bridge](https://github.com/GokServis/rust-cfs-bridge).

Web UI for the **rust-cfs-bridge** stack: **MobX** stores, **React Router** (`/` commands, `/telemetry` downlink), and **pure CSS** under `src/components/` (layout, command/telemetry screens, shared `ui/` primitives). It loads the command dictionary from **`bridge-server`** (`GET /api/commands`) and sends JSON to **`POST /api/send`**. Telemetry uses WebSocket **`/api/tlm/ws`** with a **bounded buffer**, **filters**, and **pagination** (`TelemetryStore`, `TelemetryLogTable`); see [docs/TELEMETRY.md](../docs/TELEMETRY.md). **Theme:** `data-theme` on `<html>` (dark default, toggle in header; persisted in `localStorage`).

## Which URL?

| How you run things | Open in the browser |
|--------------------|---------------------|
| **`docker compose up`** | **`http://127.0.0.1:8080`** — the **bridge-ui** container (nginx) serves **`dist/`** and proxies **`/api`** to **bridge-server** on **127.0.0.1:8081**. There is **no** Vite process in the stack, so **`http://localhost:5173` does not apply** unless you run Vite locally. |
| **Local UI dev** (`npm run dev`) | **`http://localhost:5173`**. Vite proxies **`/api`** to **`127.0.0.1:8081`** (same as Docker nginx). Run **`bridge-server`** with **`BRIDGE_HTTP_BIND=127.0.0.1:8081`** (Compose default) or adjust the proxy in `vite.config.ts`. |

## Prerequisites

- **Node.js 20** (matches the Docker image NodeSource setup).

## Install

```bash
npm ci
```

## Development

With **`bridge-server`** listening on **`127.0.0.1:8081`** (same as Docker Compose), start Vite; it proxies **`/api`** to that host with **WebSocket upgrade** for telemetry (see `vite.config.ts`).

```bash
npm run dev
```

Open the URL Vite prints (usually `http://localhost:5173`).

## Production build

```bash
npm run build
```

Output is under **`dist/`**. In the **split Compose** stack, nginx serves **`dist/`** from the **bridge-ui** image; the legacy **`entrypoint.sh`** path still sets **`BRIDGE_STATIC_DIR=/app/bridge-ui/dist`** for a monolithic `docker run`.

## Lint and test

```bash
npm run lint
npm run lint:fix
npm run test
npm run test:coverage
```

Coverage is enforced at **≥90% lines** on `src/` (see `vite.config.ts` and CI).

## API contract

The UI follows the JSON shapes accepted by **`SpaceCommand::from_json`** in the Rust crate: named dictionary commands (`command`, `sequence_count`, optional hex `payload`) or legacy `apid` + hex `payload`. The browser never implements CCSDS framing; only **`bridge-server`** does.
