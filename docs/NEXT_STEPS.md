# Roadmap

## Phase 4: Orchestration and Developer Experience

- [x] Compose profiles: default **bridge-server** + **bridge-ui**; `docker compose --profile cfs up` for **core-cpu1**
- [x] Split Docker entrypoints (`entrypoint-bridge.sh`, `entrypoint-cfs.sh`); **privileged** only on **cfs** service; legacy **`entrypoint.sh`** for monolithic `docker run`
- [x] **nginx** image (`docker/Dockerfile.bridge-ui`) + **`docker/nginx-bridge-ui.conf`** proxy for `/api` and WebSocket
- [x] Rust: supervised UDP listener (bind backoff, `recv_from` error handling)
- [x] UI: **Bridge (API)** vs **Downlink** connection status on telemetry overview
- [x] Docs and **Makefile** workflow (ports **8080** nginx / **8081** bridge-server)

### Follow-up (optional)

- [ ] Slim Docker image or build stage without cFS for faster UI/API-only iteration
