# Mission certification report

**Date:** 2026-04-07  
**Scope:** Full-loop Docker E2E (`scripts/e2e/full_loop_verify.sh`), Rust QA, C `ai_app` unit tests, optional Rust coverage.

## 1. End-to-end (automated full loop)

| Item | Value |
|------|--------|
| **Command** | `./scripts/e2e/full_loop_verify.sh` (repo root) |
| **Compose** | `docker compose --profile cfs up --build -d` |
| **Bridge API** | `http://127.0.0.1:8080` (nginx → `bridge-server`) |
| **Result** | **PASS** (exit code 0) |

**Observed golden log markers (excerpt):**

- `UDP_CFDP_INGEST: listening`
- `CF R…: successfully retained file as …/cf/(tmp/)?ai_app_weights\.tbl`
- `AI_APP weights table validation success` (or `BRAIN_UPLOAD_E2E: AI_APP_VALIDATE_OK`)
- `BRAIN_UPLOAD_E2E: CFE_TBL_LOAD_EVS_OK`

## 2. Tests

| Suite | Command | Result |
|--------|---------|--------|
| **Rust** | `cargo test --all` in `rust-bridge/` | **PASS** — 67 lib + 2 command_verification + 2 persistence + 2 tlm_ws = **73** tests |
| **ai_app C UT** | `make ai-app-ut` | **PASS** — autograd, tensor, gpt, sb, tbl |

## 3. Lint / format

| Check | Command | Result |
|--------|---------|--------|
| **rustfmt** | `cargo fmt --check` | **PASS** |
| **clippy** | `cargo clippy --all-targets -- -D warnings` | **PASS** |

## 4. Coverage (Rust)

| Command | `cargo llvm-cov test --workspace --summary-only` (in `rust-bridge/`) |
|--------|--------------------------------------------------------------------------|
| **Workspace lines (TOTAL)** | **75.77%** |

**Notable file line coverage (llvm-cov):**

- `ai_app/table_image.rs` — ~91.5%
- `ai_app/cfe_tbl_file.rs` — ~89.5%
- `brain_upload.rs` — 0% in this run (not executed under unit/integration tests; covered by E2E above)

## 5. Phase 1 alignment checklist (integrity)

| Item | Status |
|------|--------|
| **116-byte cFE wrapper** (`CFE_FS` + `CFE_TBL` file headers before raw image) | Aligned — see `rust-bridge/src/ai_app/cfe_tbl_file.rs`, tests |
| **CRC32 field offset** (`Hdr.Crc32` at byte 12) | Aligned with `offsetof(..., Crc32)` |
| **`AI_APP_WeightsTblHdr_t` size** | **104 bytes** on gcc/x86_64 — 4 bytes tail padding after `MissionVersion[64]` before `double` data; Rust builder pads to `AI_APP_WEIGHTS_TBL_HDR_LAYOUT_BYTES` (see `table_image.rs`) |
| **Full raw image size** | **`sizeof(AI_APP_WeightsTable_t)` = 104808** bytes for default mission dims; CRC is over the full struct image skipping only the 4 CRC bytes (includes header tail padding and trailing struct padding) |
| **Registry / file name** | `CFE_TBL_Register` uses base name `WEIGHTS` → qualified `AI_APP.WEIGHTS`; matches load/activate and bridge config |
| **Mission dims** | `AiAppDims` defaults match `default_ai_app_mission_cfg.h` |

### Fix applied for CRC mismatch (Rust ↔ C)

The flight CRC did not match the bridge because the serialized table omitted **4 bytes** of padding that exist in the C header (`sizeof(AI_APP_WeightsTblHdr_t) == 104` vs 100 bytes of logical fields). Weights must start at **offset 104**, and the total byte length must equal **`sizeof(AI_APP_WeightsTable_t)`** so the CRC over `sizeof(*Tbl)` matches.

---

*Artifact produced per “Full-loop mission certification” plan; the plan file itself was not modified.*
