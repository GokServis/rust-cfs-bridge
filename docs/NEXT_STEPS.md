# Next steps — roadmap

This document tracks **stability fixes**, **telemetry expansion**, **CI/CD**, and **documentation** after recent **E2E validation** (live TO_LAB → UDP → bridge → `/telemetry`). Mission inventory and topic IDs remain in [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md).

---

## E2E validation snapshot (current project state)

| Result | Detail |
|--------|--------|
| **Downlink path works** | `EnableOutput`, SCH/HK (or Docker patch), ES HK reaches `BRIDGE_TLM_BIND`; WebSocket shows `es_hk_v1` and live log updates. |
| **Noise pattern (resolved)** | Previously: alternating **`es_hk_v1`** and **`parse_error`** (~100 ms offset) from a **20-byte** UDP datagram with **CCSDS APID 128** (`0x080`) whose bytes **6–7** carry **MsgId LE `0x0F00`**. Now: Rust classification handles **`0x0F00`** and emits `to_lab_hk_v1` for this packet. |
| **Root cause (working hypothesis)** | On-wire **TO_LAB HK** (or passthrough) layout **does not match** the synthetic parser; treat **`parse_error` hex** as ground truth until the parser is aligned. |

**Mission inventory (cFS topic IDs, `to_lab_sub.c`, parser coverage):** [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md).

---

## Immediate fixes (stability)

- [x] **[Rust]** Align **`parse_to_lab_hk_datagram`** with the **actual 20-byte** on-wire layout observed in E2E (APID 128, length field → 20-byte total). Reconcile **MsgId at bytes 6–7** (`0x0F00` LE in capture) vs current constant **`TO_LAB_HK_TLM_MSGID_LE` (`0x0880`)** — confirm against mission **`default_to_lab_msgids.h`** / EDS and TO_LAB encode path ([`to_lab_encode` / passthru](../cfs/apps/to_lab/fsw/src/)).
- [x] **[Rust]** Add **unit tests** using **verified hex** from [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md) (ICD section) so regressions are caught in CI.
- [ ] **[TS]** Extend **Kind filter** on `/telemetry` with an explicit **“Hide `parse_error`”** / **“Show only parsed kinds”** control (in addition to existing kind/APID/search) to reduce operator noise when **`parse_error` is expected** during parser bring-up.
- [ ] **[TS]** Optional: persist filter preference in **session** or **localStorage** for repeat operators.

---

## Telemetry expansion (the parser gap)

**Tracking (onboard subscription scope):** extend **[`to_lab_sub.c`](../cfs/apps/to_lab/fsw/tables/to_lab_sub.c)** to add MsgIds for products you want on the wire. Link the **GitHub Issue** for `to_lab_sub.c` expansion here once filed: *(add URL)* — repository: [GokServis/rust-cfs-bridge](https://github.com/GokServis/rust-cfs-bridge/issues).

### `CFE_EVS_LONG_EVENT_MSG_MID` — parser → UI log component

Step-by-step (order matters for integration testing):

1. - [ ] **[C]** Confirm **`CFE_EVS_LONG_EVENT_MSG_MID`** (and short event if needed) in mission **`cfe-topicids.xml`** / generated MsgId headers; add row to **`to_lab_sub.c`** and reload table per cFS procedures.
2. - [ ] **[C]** Verify on **SB** / EVS that events are published when expected (ground or script trigger).
3. - [ ] **[Rust]** Capture **hex** from UDP (`parse_error` row is fine initially); define **`TlmEvent`** variant (e.g. `evs_long_event_v1`) and parser module under `rust-bridge/src/tlm/` (layout: CCSDS + cFE secondary + EVS payload per ICD).
4. - [ ] **[Rust]** Wire **`classify_datagram`** and add **serde** + tests (golden vectors from capture).
5. - [ ] **[TS]** Add **`telemetryTypes.ts`** / MobX types and a **log-friendly row** (message text, severity, app id — fields per real layout).
6. - [ ] **[TS]** Add **`EvsLongEventPanel`** or reuse **`TelemetryLogTable`** / **`ParseErrorPanel`** pattern for structured display; Vitest coverage ≥90% for new code paths.
7. - [ ] **[Docs]** Update [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md) **rust-cfs-bridge** table and ICD rows.

---

## CI/CD hygiene

**Status:** Implemented in [`.github/workflows/rust-bridge.yml`](../.github/workflows/rust-bridge.yml).

- [x] **[CI]** Path triggers include **`scripts/**`** so script edits run the workflow.
- [x] **[CI]** **`python3 -m py_compile scripts/*.py`** (syntax check for helper scripts).
- [x] **[CI]** **`cargo build --release --bin bridge-server`** + background **UDP sink on `127.0.0.1:1234`** + **`scripts/verify_uplink_dictionary.py`** (uplink dictionary contract); **`BRIDGE_HTTP_BASE=http://127.0.0.1:8080`** set for the script.

**Ongoing:**

- [ ] **[CI]** Keep **pre-commit** ([`.pre-commit-config.yaml`](../.pre-commit-config.yaml)) aligned with workflow when Rust/UI thresholds change.
- [ ] **[CI]** If **`verify_live_telemetry_no_mock.py`** is ever added, use a **separate workflow** or **nightly** job (Docker / cFS — too heavy for default PR CI).

---

## Documentation

- [ ] **[Docs]** Keep [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md) **ICD / verified captures** section updated when parsers change or new live sessions are recorded.

---

## Historical context (unchanged goals)

### Mock script vs live cFS telemetry

| Source | Role |
|--------|------|
| **[`scripts/mock_es_hk_udp.py`](../scripts/mock_es_hk_udp.py)** | Injects **ES HK–shaped UDP** directly to `BRIDGE_TLM_BIND`. Does **not** go through cFS. Use for **ground-stack** checks when TO_LAB is idle. |
| **TO_LAB → UDP** | **Primary path** for **real** onboard telemetry: SB → TO_LAB (subscription + **`EnableOutput`**) → UDP. |

See [TELEMETRY.md](TELEMETRY.md), [MESSAGE_FLOW.md](MESSAGE_FLOW.md), [ARCHITECTURE.md](ARCHITECTURE.md).

### Goal A — Reliably receive telemetry

1. Operational verification — `bridge-server` logs **`telemetry UDP listening on …`**; mock or live TO_LAB; watch `/telemetry`.
2. cFS / TO_LAB — `to_lab_sub.tbl`, **`EnableOutput`**, HK / SCH as needed.
3. Rust — additional **`TlmEvent`** kinds per product; unparsed → **`parse_error`** until implemented.
4. Backpressure — broadcast capacity; optional server-side history later.

### Goal B — Filtered observation in the UI

**Implemented:** bounded buffer, kind/APID/search filters, pagination, `TelemetryLogTable`. Roadmap items above add **parse_error visibility** controls and new kinds (EVS).

### Suggested order (remaining work)

1. **Stability** — TO_LAB HK parser alignment + UI parse_error filter (this document, top sections).
2. **cFS** — Subscription expansion (GitHub issue + `to_lab_sub.c`).
3. **Rust / TS** — EVS long event pipeline per checklist.
4. **Polish** — Time-range filter, virtualization, server history only if needed.

---

## Housekeeping

- Merge feature branches when CI is green.
- When **`targets.cmake`**, **`cfe-topicids.xml`**, **TO_LAB subscriptions**, or **Rust parsers** change, update [AVAILABLE_TELEMETRY.md](AVAILABLE_TELEMETRY.md).
- After server-side history (if ever), extend [TELEMETRY.md](TELEMETRY.md).
