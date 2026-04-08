#!/usr/bin/env python3
"""
E2E log watcher for "brain upload" (weights uplink) flow.

This is intentionally simple and lab-oriented:
- streams `docker compose --profile cfs logs -f --no-color cfs`
- waits for a set of regex patterns within a timeout

Usage:
  scripts/e2e/e2e_log_watcher.py --timeout 180 \
    --pattern "UDP_CFDP_INGEST: listening" \
    --pattern "CF R\\d+\\(\\d+:\\d+\\): successfully retained file as /cf/ai_app_weights\\.tbl" \
    --pattern "AI_APP initialized"
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import time


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--timeout", type=float, default=180.0)
    ap.add_argument(
        "--pattern",
        action="append",
        default=[],
        help="Regex pattern to require (may repeat).",
    )
    ap.add_argument(
        "--compose-service",
        default=None,
        help="Deprecated: single service. Prefer --services.",
    )
    ap.add_argument(
        "--services",
        nargs="*",
        default=None,
        help="docker compose service names to merge (default: cfs bridge-server).",
    )
    return ap.parse_args()


def main() -> int:
    args = parse_args()
    patterns = [re.compile(p) for p in (args.pattern or [])]

    if not patterns:
        # Full-loop: cfs (ingest + CF retain + optional AI EVS) + bridge-server (orchestrator markers).
        # CFE_TBL long-form strings often do not appear verbatim in abbreviated docker logs; the
        # rust-bridge brain upload prints BRAIN_UPLOAD_E2E lines to stderr after EVS matches.
        patterns = [
            re.compile(r"UDP_CFDP_INGEST: listening"),
            re.compile(
                r"CF R\d+\(\d+:\d+\): successfully retained file as (/)?cf/(tmp/)?ai_app_weights\.tbl"
            ),
            re.compile(r"BRAIN_UPLOAD_E2E: CFE_TBL_LOAD_EVS_OK"),
            re.compile(
                r"(BRAIN_UPLOAD_E2E: AI_APP_VALIDATE_OK|AI_APP weights table validation success)"
            ),
        ]

    found = [False] * len(patterns)
    start = time.time()

    # Hard-fail patterns: if any of these appear, abort immediately.
    fail_patterns = [
        re.compile(r"AI_APP weights table validation failed"),
        re.compile(r"CF R\d+\(\d+:\d+\): cannot move file to (/)?cf/(tmp/)?ai_app_weights\.tbl"),
        re.compile(r"CF R\d+\(\d+:\d+\): CRC mismatch"),
        re.compile(r"brain_upload: FAILED:"),
    ]

    if args.compose_service:
        services = [args.compose_service]
    elif args.services:
        services = list(args.services)
    else:
        services = ["cfs", "bridge-server"]

    cmd = ["docker", "compose", "--profile", "cfs", "logs", "-f", "--no-color", *services]

    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )

    try:
        assert proc.stdout is not None
        for line in proc.stdout:
            now = time.time()
            if now - start > args.timeout:
                break

            for fp in fail_patterns:
                if fp.search(line):
                    print(f"[FAIL] {fp.pattern}", file=sys.stderr)
                    return 3

            for i, pat in enumerate(patterns):
                if not found[i] and pat.search(line):
                    found[i] = True
                    print(f"[FOUND] {pat.pattern}", file=sys.stderr)

            if all(found):
                return 0
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.kill()

    missing = [patterns[i].pattern for i, ok in enumerate(found) if not ok]
    print(f"[TIMEOUT] Missing patterns: {missing}", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())

