#!/usr/bin/env python3
"""
Send a synthetic CFE ES HK-sized UDP datagram to the bridge telemetry listener.

Matches Linux little-endian layout used by rust-bridge `tlm::es_hk` (12-byte headers + 168-byte payload).
Default target: 127.0.0.1:2234 (override with BRIDGE_TLM_BIND or argv).
"""
from __future__ import annotations

import os
import socket
import struct
import sys

# Must match rust-bridge `tlm::es_hk` (CFE_MISSION_ES_PERF_MAX_IDS = 128).
TL_HEADER = 12
PAYLOAD = 168
TOTAL = TL_HEADER + PAYLOAD


def build_packet() -> bytes:
    buf = bytearray(TOTAL)
    user_len = TOTAL - 6
    w2 = user_len - 1
    # CCSDS primary: TM, secondary header flag, APID 0; seq 0; data length
    struct.pack_into(">HHH", buf, 0, 0x0800, 0xC000, w2)
    # Payload starts at 12: command counters + checksum + versions (set magic bytes)
    buf[12] = 0xC0
    buf[13] = 0xFF
    buf[14] = 0xEE
    buf[15] = 0x01  # cFE major
    struct.pack_into("<I", buf, 48, 42)  # registered_core_apps
    struct.pack_into("<Q", buf, 144, 0x100000)  # heap_bytes_free
    return bytes(buf)


def main() -> None:
    target = os.environ.get("BRIDGE_TLM_BIND", "127.0.0.1:2234")
    if len(sys.argv) >= 2:
        target = sys.argv[1]
    host, port_s = target.rsplit(":", 1)
    port = int(port_s)
    data = build_packet()
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.sendto(data, (host, port))
    print(f"Sent {len(data)} bytes to {host}:{port}")


if __name__ == "__main__":
    main()
