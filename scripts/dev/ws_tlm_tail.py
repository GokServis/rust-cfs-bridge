#!/usr/bin/env python3
"""
Tail bridge-server telemetry websocket and print selected events.

Usage:
  scripts/dev/ws_tlm_tail.py --url ws://127.0.0.1:8080/api/tlm/ws --timeout 180
"""

from __future__ import annotations

import argparse
import asyncio
import json
import time

import websockets


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", default="ws://127.0.0.1:8080/api/tlm/ws")
    ap.add_argument("--timeout", type=float, default=180.0)
    return ap.parse_args()


async def main_async() -> int:
    args = parse_args()
    end = time.time() + args.timeout
    while time.time() < end:
        try:
            # Disable client keepalive pings; we only need best-effort tailing.
            async with websockets.connect(args.url, ping_interval=None) as ws:
                while time.time() < end:
                    try:
                        msg = await asyncio.wait_for(ws.recv(), timeout=1.0)
                    except asyncio.TimeoutError:
                        continue

                    try:
                        ev = json.loads(msg)
                    except json.JSONDecodeError:
                        print(msg)
                        continue

                    k = ev.get("kind")
                    if k in ("brain_upload_progress", "command_ack", "cf_eot_v1"):
                        print(json.dumps(ev, sort_keys=True))
                    elif k == "evs_long_event_v1":
                        m = (ev.get("evs_long_event") or {}).get("message", "")
                        if (
                            "successfully retained file as" in m
                            and "ai_app_weights.tbl" in m
                            or "AI_APP weights table validation" in m
                        ):
                            print(json.dumps(ev, sort_keys=True))
        except Exception:
            # Reconnect on any transient websocket/proxy error.
            await asyncio.sleep(0.5)
    return 0


def main() -> int:
    return asyncio.run(main_async())


if __name__ == "__main__":
    raise SystemExit(main())

