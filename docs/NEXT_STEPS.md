# Next steps

This document summarizes where the **rust-cfs-bridge** repo stands and concrete follow-ups aligned with **receiving telemetry** and **observing it in the UI with filtering and pagination**.

**Mission inventory (cFS topic IDs, default cpu1 apps, TO_LAB table, parser coverage):** [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md) — use it as the source of truth when choosing which packets to enable and which Rust parsers to add.

## Mock script vs live cFS telemetry

| Source | Role |
|--------|------|
| **[`scripts/mock_es_hk_udp.py`](../scripts/mock_es_hk_udp.py)** | Injects **ES HK–shaped UDP** directly to `BRIDGE_TLM_BIND`. Does **not** go through cFS. Use for **ground-stack** checks (Rust parse, WebSocket, UI) when TO_LAB is idle or timing is inconvenient. **Keep it** for developer smoke tests and CI-style validation; **deleting it is not required** for a “complete” mission. |
| **TO_LAB → UDP** | **Primary path** for **real** onboard telemetry: Software Bus → TO_LAB (subscription table + **`EnableOutput`**) → UDP to the bridge. This is what you should treat as **production-like** acceptance once configured. |

**Recommended milestone:** On the cFS side, complete whatever is still needed so that **HK (and any other subscribed products)** actually flow through TO_LAB to `BRIDGE_TLM_BIND`, then **verify** in bridge logs, WebSocket JSON (`kind`), and the `/telemetry` UI — **without** relying on the Python script for that acceptance run. Continue to use the mock when you want a **deterministic** test that does not depend on SCH/SEND_HK/TO_LAB timing.

See also: [TELEMETRY.md](TELEMETRY.md), [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md) (three-layer flow, `to_lab_sub.c`).

---

## Current state (snapshot)

| Area | What exists today |
|------|-------------------|
| **Uplink** | `POST /api/send` → CCSDS + CRC → UDP → CI_LAB; dictionary in `rust-bridge/src/lib.rs`; UI on `/`. |
| **Downlink** | UDP on `BRIDGE_TLM_BIND` (default `127.0.0.1:5001`); each datagram → `classify_datagram` → `TlmEvent` JSON; `tokio::sync::broadcast` to WebSocket clients at **`GET /api/tlm/ws`**. |
| **Parsing** | ES HK v1 (`es_hk_v1`) and `parse_error` in `rust-bridge/src/tlm/`; other SB products (EVS, SB stats, TO_LAB HK, CI_LAB HK, …) arrive as **`parse_error`** until new `TlmEvent` arms exist — see [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md) (“rust-cfs-bridge — what is parsed today”). |
| **UI** | MobX `TelemetryStore` opens **`/api/tlm/ws`**, keeps a **bounded buffer**, **filters**, **pagination**, and **`TelemetryLogTable`** on `/telemetry` (`TelemetryOverview`, `EsHkPanel`, `ParseErrorPanel`). |
| **cFS (this repo)** | Default cpu1 apps: `sample_app`, `ci_lab`, `bridge_reader`, `to_lab`, `sch_lab` (`cfs/cfe/cmake/sample_defs/targets.cmake`). Optional apps (FM, LC, DS, …) are **not** built unless you extend `APPLIST`. Full telemetry **topic** list: `cfs/cfe/cmake/sample_defs/eds/cfe-topicids.xml` — [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md). |
| **TO_LAB → UDP** | [`to_lab_sub.c`](../cfs/apps/to_lab/fsw/tables/to_lab_sub.c) in this repo enables **`CFE_ES_HK_TLM_MID`** and **`TO_LAB_HK_TLM_MID`**; you still need **`EnableOutput`** to the ground IP and HK scheduling — see [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md) (“TO_LAB subscription table”). |
| **CI** | [`.github/workflows/rust-bridge.yml`](../.github/workflows/rust-bridge.yml): `rust-bridge` (fmt, clippy, test, llvm-cov ≥90%) and `bridge-ui` (lint, build, coverage ≥90%). |

See also: [TELEMETRY.md](TELEMETRY.md), [MESSAGE_FLOW.md](MESSAGE_FLOW.md), [ARCHITECTURE.md](ARCHITECTURE.md), [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md).

---

## Goal A — Reliably receive telemetry

These steps improve **end-to-end receipt** (lab / spacecraft → bridge → UI). Align onboard work with the **three layers** in [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md) (Software Bus → TO_LAB UDP → Rust parse).

1. **Operational verification** — Confirm `bridge-server` logs **`telemetry UDP listening on …`**, then either run **[`mock_es_hk_udp.py`](../scripts/mock_es_hk_udp.py)** (ground-only) or **live TO_LAB** once the path below is active. Watch `/telemetry` or a WebSocket client. Adjust `BRIDGE_TLM_BIND` / firewall / Docker host networking as in [TELEMETRY.md](TELEMETRY.md).

2. **cFS / TO_LAB (live downlink)** — Unblock **real** UDP from cFS: ensure **`to_lab_sub.tbl`** includes the MsgIds you need (extend beyond ES HK / TO_LAB HK if required), reload per cFS procedures, run **`EnableOutput`** toward the bridge host/port, and drive **HK** via **`SEND_HK`** / **SCH_LAB** as appropriate. Pick streams from the **mission inventory** in [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md). **Acceptance:** packets arrive at the bridge **without** the Python mock for that test.

3. **Rust: additional packet kinds** — For each product the mission emits on the wire, add parsing + tests under `rust-bridge/src/tlm/`, extend `TlmEvent`, and mirror types in `telemetryTypes.ts`. Prioritize products you enable in TO_LAB; unparsed packets remain **`parse_error`** with `hex_preview` until implemented.

4. **Backpressure and capacity** — The broadcast channel has a fixed capacity; lagging WebSocket clients drop (`Lagged`). For **history** or **slow consumers**, decide whether to (a) keep “live only” and document limits, or (b) add a **bounded server-side ring** and optional **HTTP history** endpoint (see Goal B).

---

## Goal B — Filtered, paginated observation in the UI

**Implemented:** `TelemetryStore` retains a bounded ring buffer, exposes **filters** (kind, APID, search), **pagination**, and **`TelemetryLogTable`** on `/telemetry`. Remaining items are **optional enhancements**.

### Optional follow-ups

1. **Data model** — Tune buffer cap, session banners, or derive “latest” hero metrics only from the buffer.
2. **Filtering** — Add **time-range** filter or debounced search if operators need it; extend **`kind`** filters as new `TlmEvent` variants appear ([AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md)).
3. **Virtualization** — If buffer caps grow large, consider `@tanstack/react-virtual` for the log table.
4. **Server-side history** — Optional **GET `/api/tlm/recent?limit=&before=`** (or similar) if refresh-safe backlog is required ([TELEMETRY.md](TELEMETRY.md) housekeeping).

**Tests:** Keep Vitest coverage ≥90% per CI for store helpers and components.

---

## Suggested order (remaining work)

1. **cFS / TO_LAB** — Finish onboard steps so **live** HK (and other subscribed telemetry) reaches `BRIDGE_TLM_BIND`; document the exact commands / table loads your mission uses.
2. **Acceptance run** — Validate WebSocket `kind` / fields and `/telemetry` log against **live** UDP; use **`mock_es_hk_udp.py`** only as a fallback or regression check, not as the only proof of cFS health.
3. **Rust parsers** — Add `TlmEvent` variants for additional products as they appear on the wire; update [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md).
4. **UI polish** — Time-range filter, virtualization, or server history only if needed.

---

## Housekeeping

- Merge feature branches when CI is green.
- When **`targets.cmake`**, **`cfe-topicids.xml`**, **TO_LAB subscriptions**, or **Rust parsers** change, update [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md) per its maintenance note.
- After adding server-side history (if ever), extend [TELEMETRY.md](TELEMETRY.md) (buffer semantics, no history unless implemented).
