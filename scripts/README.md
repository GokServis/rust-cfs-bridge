# Scripts

Helpers for CI, end-to-end checks, local development, and golden-vector generation. In Docker images the repo root is `/app`, so paths look like `/app/scripts/<subdir>/...`.

## Index

| Path | Purpose | Typical use | CI ([`scripts-integration.yml`](../.github/workflows/scripts-integration.yml)) | Dependencies |
|------|---------|-------------|--------------------------------------------------------------------------------|----------------|
| [`ci/verify_uplink_dictionary.py`](ci/verify_uplink_dictionary.py) | POST every command from `/api/commands` via `/api/send` | With `bridge-server` up | Yes | stdlib only |
| [`e2e/full_loop_verify.sh`](e2e/full_loop_verify.sh) | Compose cFS profile, health wait, POST brain upload, log golden sequence | Full stack certification | No (heavy / Docker) | bash, docker, curl, Python |
| [`e2e/e2e_log_watcher.py`](e2e/e2e_log_watcher.py) | Tail `cfs` logs; match regex patterns within timeout | Called by `full_loop_verify.sh` | No | stdlib only |
| [`dev/mock_es_hk_udp.py`](dev/mock_es_hk_udp.py) | Emit synthetic ES HK UDP for telemetry UI | Local / Docker without cFS | No | stdlib only |
| [`dev/ws_tlm_tail.py`](dev/ws_tlm_tail.py) | Tail WebSocket `/api/tlm/ws` | Debugging downlink | No | `websockets` (PyPI) |
| [`lab/verify_live_telemetry_no_mock.py`](lab/verify_live_telemetry_no_mock.py) | Assert live ES HK / TO_LAB HK over WS without mock | Bridge + cFS stack | No | stdlib |
| [`golden/autograd_golden.py`](golden/autograd_golden.py) | C-style golden vectors for autograd UT | Regenerate parity with C | No | stdlib |
| [`golden/tensor_golden.py`](golden/tensor_golden.py) | Tensor goldens | Same | No | stdlib |
| [`golden/gpt_forward_golden.py`](golden/gpt_forward_golden.py) | GPT forward goldens | Same | No | stdlib |
| [`reference/microgpt.py`](reference/microgpt.py) | Standalone micro GPT reference (Karpathy-style) | Not for CI; may download `input.txt` | No | stdlib; network if no `input.txt` |
| [`maintenance/ensure-github-forks.sh`](maintenance/ensure-github-forks.sh) | Fork upstream cFS/cFE/ci_lab via GitHub API | Manual maintenance | No | `curl`, `GITHUB_TOKEN` |

## Subdirectories

- **`ci/`** — Fast checks intended for GitHub Actions (stdlib Python).
- **`e2e/`** — Docker + cFS full-loop automation and log watching.
- **`dev/`** — Developer utilities (mocks, WS tail).
- **`lab/`** — Live-stack acceptance scripts.
- **`golden/`** — Outputs aligned with [`cfs/apps/ai_app/unit-test/`](../cfs/apps/ai_app/unit-test/).
- **`reference/`** — Long-form reference code; not wired into CI by default.
- **`maintenance/`** — One-off repo/org maintenance (tokens required).

## Not in default CI

Full Docker E2E (`e2e/full_loop_verify.sh`), live telemetry lab checks, and `ws_tlm_tail.py` (extra dependency) are documented for manual or scheduled runs, not every PR.
