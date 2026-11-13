"""Calls every read tool on a running endpoint and prints the answers.

Usage: python scripts/smoke.py [port]. Needs the game or the dedicated server
running with the mod and a world loaded. Exits 1 when a call fails.
"""

import json
import os
import sys
import time
import urllib.request

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8751
TOKEN = os.environ.get("ENSHROUDED_MCP_TOKEN", "")
URL = f"http://127.0.0.1:{PORT}/mcp"


def rpc(method, params=None, ident=1):
    body = {"jsonrpc": "2.0", "id": ident, "method": method}
    if params is not None:
        body["params"] = params
    req = urllib.request.Request(URL, data=json.dumps(body).encode(), headers={"Content-Type": "application/json"})
    if TOKEN:
        req.add_header("Authorization", "Bearer " + TOKEN)
    with urllib.request.urlopen(req, timeout=15) as r:
        return json.loads(r.read())


def tool(name, **args):
    for _ in range(5):  # a read of an unloaded area starts loading it; enshwick says retry after about a second
        result = rpc("tools/call", {"name": name, "arguments": args})["result"]
        text = result["content"][0]["text"]
        if "not loaded" not in text:
            break
        time.sleep(1.5)
    print(f"--- {name} {args or ''}{' (error)' if result['isError'] else ''}\n{text}")
    return result["isError"], text


def main():
    init = rpc("initialize", {"protocolVersion": "2025-06-18"})["result"]
    print("server:", init["serverInfo"], "protocol:", init["protocolVersion"])
    print("tools:", [t["name"] for t in rpc("tools/list")["result"]["tools"]])
    failed = False
    err, text = tool("status")
    failed |= err
    if err or not json.loads(text).get("hosting"):
        print("no world runs; skipping the world reads")
        return 1 if failed else 0
    err, text = tool("players")
    failed |= err
    players = json.loads(text) if not err else []
    centre = next((p["position"] for p in players if p.get("position")), None)
    if centre is None:
        print("no player position; reading near the map centre instead")
        centre = [5120.0, 100.0, 5120.0]
    x, y, z = centre
    failed |= tool("voxel_sample", x=x, y=y - 0.5, z=z)[0]
    failed |= tool("props_in", x1=x - 16, y1=y - 8, z1=z - 16, x2=x + 16, y2=y + 8, z2=z + 16)[0]
    refused, _ = tool("voxel_sample", x=-5, y=0, z=0)
    if not refused:
        print("FAIL: a point outside the world was not refused")
        failed = True
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
