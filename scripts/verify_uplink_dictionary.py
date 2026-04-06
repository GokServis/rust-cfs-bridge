#!/usr/bin/env python3
"""POST each command from GET /api/send (dictionary) and report HTTP status."""
from __future__ import annotations

import json
import os
import sys
import urllib.error
import urllib.request


def main() -> None:
    base = os.environ.get("BRIDGE_HTTP_BASE", "http://127.0.0.1:8080").rstrip("/")
    with urllib.request.urlopen(f"{base}/api/commands") as r:
        cmds = json.load(r)
    names = [c["name"] for c in cmds]
    print(f"Checking {len(names)} commands against {base}", file=sys.stderr)
    for name in names:
        body = json.dumps({"command": name, "sequence_count": 0}).encode()
        req = urllib.request.Request(
            f"{base}/api/send",
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urllib.request.urlopen(req) as resp:
                if resp.status != 200:
                    print(f"FAIL {name} HTTP {resp.status}", file=sys.stderr)
                    sys.exit(1)
        except urllib.error.HTTPError as e:
            print(f"FAIL {name} HTTP {e.code}: {e.read().decode()}", file=sys.stderr)
            sys.exit(1)
        print(f"OK {name}")
    print("All dictionary commands accepted.", file=sys.stderr)


if __name__ == "__main__":
    main()
