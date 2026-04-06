#!/usr/bin/env python3
"""
Live no-mock acceptance: WebSocket /api/tlm/ws must emit es_hk_v1 and/or to_lab_hk_v1
without mock_es_hk_udp.py. Optionally grep Docker / cFS log for TO_LAB EnableOutput EVS text.

Requires: bridge-server + cFS (e.g. docker compose up), BRIDGE_TLM_BIND aligned with TO_LAB
(default 127.0.0.1:2234). Sends CMD_TO_LAB_ENABLE_OUTPUT via POST /api/send.

Usage:
  python3 scripts/verify_live_telemetry_no_mock.py
  BRIDGE_HTTP_BASE=http://127.0.0.1:8080 python3 scripts/verify_live_telemetry_no_mock.py
  python3 scripts/verify_live_telemetry_no_mock.py --check-docker-log
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import secrets
import socket
import struct
import subprocess
import sys
import time
import urllib.error
import urllib.request
from urllib.parse import urlparse

TARGET_KINDS = frozenset({"es_hk_v1", "to_lab_hk_v1"})
TO_LAB_GREP_NEEDLE = "telemetry output enabled"


def _http_base_to_ws_and_api(base: str) -> tuple[str, str, str, int]:
    u = urlparse(base)
    if u.scheme not in ("http", "https"):
        raise SystemExit(f"unsupported scheme in BRIDGE_HTTP_BASE: {base!r}")
    host = u.hostname or "127.0.0.1"
    port = u.port or (443 if u.scheme == "https" else 80)
    ws_scheme = "wss" if u.scheme == "https" else "ws"
    ws_url = f"{ws_scheme}://{host}:{port}/api/tlm/ws"
    api_root = f"{u.scheme}://{host}:{port}"
    return ws_url, api_root, host, port


def _ws_handshake(sock: socket.socket, host: str, port: int, path: str) -> bytes:
    """HTTP Upgrade; returns any bytes already read after the response headers (first WS frames)."""
    key = base64.b64encode(secrets.token_bytes(16)).decode("ascii")
    req = (
        f"GET {path} HTTP/1.1\r\n"
        f"Host: {host}:{port}\r\n"
        "Upgrade: websocket\r\n"
        "Connection: Upgrade\r\n"
        f"Sec-WebSocket-Key: {key}\r\n"
        "Sec-WebSocket-Version: 13\r\n"
        "\r\n"
    )
    sock.sendall(req.encode())
    buf = b""
    while b"\r\n\r\n" not in buf:
        chunk = sock.recv(4096)
        if not chunk:
            raise ConnectionError("incomplete WebSocket handshake response")
        buf += chunk
    head, _, tail = buf.partition(b"\r\n\r\n")
    status_line = head.split(b"\r\n", 1)[0]
    if b" 101 " not in status_line:
        raise ConnectionError(f"expected HTTP 101, got: {status_line!r}")
    return tail


def _ws_read_text_frame(sock: socket.socket, buf: bytearray) -> str:
    while True:
        while len(buf) < 2:
            buf += sock.recv(65536)
        b0 = buf[0]
        b1 = buf[1]
        masked = (b1 & 0x80) != 0
        ln = b1 & 0x7F
        off = 2
        if ln == 126:
            while len(buf) < 4:
                buf += sock.recv(65536)
            ln = struct.unpack("!H", buf[2:4])[0]
            off = 4
        elif ln == 127:
            while len(buf) < 10:
                buf += sock.recv(65536)
            ln = struct.unpack("!Q", buf[2:10])[0]
            off = 10
        mask_len = 4 if masked else 0
        hdr_end = off + mask_len
        while len(buf) < hdr_end + ln:
            buf += sock.recv(65536)
        raw = buf[hdr_end : hdr_end + ln]
        if masked:
            mask = buf[off : off + 4]
            out = bytes(b ^ mask[i % 4] for i, b in enumerate(raw))
        else:
            out = bytes(raw)
        del buf[: hdr_end + ln]
        opcode = b0 & 0x0F
        if opcode == 0x1:
            return out.decode("utf-8")
        if opcode == 0x8:
            raise ConnectionError("WebSocket close frame")
        if opcode == 0x9:
            # ping — minimal pong (masked from client)
            continue
        if opcode == 0x0:
            continue


def _post_send(api_root: str, body: str) -> None:
    url = f"{api_root.rstrip('/')}/api/send"
    req = urllib.request.Request(
        url,
        data=body.encode(),
        method="POST",
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=15) as r:
            r.read()
    except urllib.error.HTTPError as e:
        err = e.read().decode(errors="replace")
        raise SystemExit(f"POST {url} failed: {e.code} {err}") from e
    except urllib.error.URLError as e:
        raise SystemExit(f"POST {url} failed: {e}") from e


def _docker_cfs_log_container_names() -> list[str]:
    """Compose v2 uses `rust-cfs-bridge-cfs`; legacy single container was `rust-cfs-bridge`."""
    raw = os.environ.get("DOCKER_CFS_LOG_CONTAINER", "").strip()
    if raw:
        return [x.strip() for x in raw.split(",") if x.strip()]
    return ["rust-cfs-bridge-cfs", "rust-cfs-bridge"]


def _docker_log_has_enable_output() -> bool:
    for container in _docker_cfs_log_container_names():
        try:
            p = subprocess.run(
                ["docker", "logs", container],
                capture_output=True,
                text=True,
                timeout=30,
            )
        except (FileNotFoundError, subprocess.TimeoutExpired) as e:
            print(f"verify_live_telemetry_no_mock: docker logs {container} skipped: {e}", file=sys.stderr)
            continue
        text = (p.stdout or "") + (p.stderr or "")
        if TO_LAB_GREP_NEEDLE.lower() in text.lower():
            return True
    return False


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--base-url",
        default=None,
        help="HTTP base (default: env BRIDGE_HTTP_BASE or http://127.0.0.1:8080)",
    )
    ap.add_argument(
        "--timeout",
        type=float,
        default=90.0,
        help="Seconds to wait for target telemetry kinds on WebSocket (default: 90)",
    )
    ap.add_argument(
        "--check-docker-log",
        action="store_true",
        help=f"After send, require docker logs (cfs container, see DOCKER_CFS_LOG_CONTAINER) to contain TO_LAB EVS text ({TO_LAB_GREP_NEEDLE!r})",
    )
    ap.add_argument(
        "--require-both",
        action="store_true",
        help="Require both es_hk_v1 and to_lab_hk_v1 (default: either kind is enough)",
    )
    args = ap.parse_args()

    base = args.base_url or os.environ.get("BRIDGE_HTTP_BASE", "http://127.0.0.1:8080")
    ws_url, api_root, host, port = _http_base_to_ws_and_api(base)
    path = "/api/tlm/ws"

    parsed = urlparse(ws_url)
    if parsed.scheme != "ws":
        raise SystemExit("this script only supports ws:// (not wss) for WebSocket")

    sock = socket.create_connection((host, port), timeout=15)
    try:
        sock.settimeout(min(30.0, args.timeout))
        leftover = _ws_handshake(sock, host, port, path)
    except OSError as e:
        raise SystemExit(f"WebSocket connect failed: {e}") from e

    buf = bytearray(leftover)
    send_body = json.dumps(
        {"command": "CMD_TO_LAB_ENABLE_OUTPUT", "sequence_count": 0},
        separators=(",", ":"),
    )
    print("verify_live_telemetry_no_mock: POST CMD_TO_LAB_ENABLE_OUTPUT …", file=sys.stderr)
    _post_send(api_root, send_body)

    if args.check_docker_log:
        time.sleep(1.5)
        if not _docker_log_has_enable_output():
            raise SystemExit(
                f"docker logs ({_docker_cfs_log_container_names()}) did not contain {TO_LAB_GREP_NEEDLE!r} "
                "(TO_LAB EVS after EnableOutput). Try: docker compose --profile cfs logs cfs | grep -i telemetry"
            )
        print(
            f"verify_live_telemetry_no_mock: docker log contains TO_LAB EVS ({TO_LAB_GREP_NEEDLE!r})",
            file=sys.stderr,
        )

    deadline = time.monotonic() + args.timeout
    seen: set[str] = set()
    want = "both kinds" if args.require_both else "es_hk_v1 or to_lab_hk_v1"
    print(
        f"verify_live_telemetry_no_mock: waiting up to {args.timeout}s for {want} on WebSocket …",
        file=sys.stderr,
    )
    while time.monotonic() < deadline:
        sock.settimeout(max(0.5, min(5.0, deadline - time.monotonic())))
        try:
            text = _ws_read_text_frame(sock, buf)
        except socket.timeout:
            continue
        except ConnectionError as e:
            raise SystemExit(f"WebSocket closed: {e}") from e
        try:
            obj = json.loads(text)
        except json.JSONDecodeError:
            continue
        kind = obj.get("kind")
        if isinstance(kind, str) and kind in TARGET_KINDS:
            seen.add(kind)
            print(f"verify_live_telemetry_no_mock: saw kind={kind!r}", file=sys.stderr)
            if args.require_both and seen >= TARGET_KINDS:
                print("verify_live_telemetry_no_mock: OK (both kinds seen)", file=sys.stderr)
                return
            if not args.require_both:
                print(
                    f"verify_live_telemetry_no_mock: OK (saw {kind!r}; use --require-both for both)",
                    file=sys.stderr,
                )
                return
    if seen:
        print(
            f"verify_live_telemetry_no_mock: partial: saw {sorted(seen)}; "
            f"timed out before {sorted(TARGET_KINDS - seen)}",
            file=sys.stderr,
        )
        raise SystemExit(1)
    raise SystemExit(
        "timeout: no es_hk_v1 or to_lab_hk_v1 on WebSocket (ensure SCH/HK drives subscribed TLM, "
        "TO_LAB EnableOutput, and BRIDGE_TLM_BIND matches TO_LAB UDP port)"
    )


if __name__ == "__main__":
    main()
