"""Run the current MCP's fixed read probes and save a summarized evidence report.

Requires an existing loopback endpoint or SSH tunnel. Never starts/restarts the
server, calls write tools, or enables writes. Derived reports stay in gamedata/.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import time
import urllib.error
import urllib.request


READS = ("status", "players", "voxel_sample", "props_in", "entities_near", "entity_components")
MAX_RESPONSE = 2 * 1024 * 1024


class InvalidReply(ValueError):
    pass


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class Client:
    def __init__(self, port, token=""):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self.token = token
        self.ident = 0
        self.opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())

    def call(self, method, params=None):
        self.ident += 1
        body = {"jsonrpc": "2.0", "id": self.ident, "method": method}
        if params is not None:
            body["params"] = params
        headers = {"Content-Type": "application/json"}
        if self.token:
            headers["Authorization"] = "Bearer " + self.token
        request = urllib.request.Request(self.url, data=json.dumps(body).encode(), headers=headers)
        with self.opener.open(request, timeout=10) as response:
            raw = response.read(MAX_RESPONSE + 1)
        if len(raw) > MAX_RESPONSE:
            raise InvalidReply("response exceeds the runner's 2 MiB limit")
        answer = json.loads(raw)
        if not isinstance(answer, dict) or answer.get("jsonrpc") != "2.0" or answer.get("id") != self.ident:
            raise InvalidReply("invalid JSON-RPC envelope or response id")
        if "error" in answer or "result" not in answer:
            raise InvalidReply("JSON-RPC error or missing result")
        return answer["result"]


def valid_center(point):
    return (
        isinstance(point, (list, tuple)) and len(point) == 3
        and all(type(v) in (int, float) and math.isfinite(v) for v in point)
        and all(0 <= v <= edge for v, edge in zip(point, (10240, 1024, 10240)))
    )


def summarize(name, value):
    """Keep observations, not player names, account keys or session entity ids."""
    if name == "players":
        if not isinstance(value, list) or not all(isinstance(p, dict) for p in value):
            raise InvalidReply("players must be a list of objects")
        return {"count": len(value), "positioned": sum(valid_center(p.get("position")) for p in value)}
    if not isinstance(value, dict):
        raise InvalidReply(f"{name} must return an object")
    if name == "status":
        if type(value.get("hosting")) is not bool:
            raise InvalidReply("status lacks a Boolean hosting field")
        return {"hosting": value["hosting"], "player_count": value.get("players")}
    if name == "voxel_sample":
        if type(value.get("fill")) is not int or not 0 <= value["fill"] <= 255 or not isinstance(value.get("material"), str):
            raise InvalidReply("invalid voxel material/fill")
        return {"material": value["material"], "fill": value["fill"]}
    key = "props" if name == "props_in" else "entities"
    entries = value.get(key)
    total = value.get("total")
    if not isinstance(entries, list) or not all(isinstance(e, dict) for e in entries) or type(total) is not int or total < len(entries):
        raise InvalidReply("invalid entity/prop counts")
    summary = {"total": total, "listed": len(entries), "truncated": total > len(entries),
               "coverage": "nonempty" if entries else "empty, known-object verification still needed"}
    if name == "entity_components":
        if type(value.get("known_types")) is not int or value["known_types"] <= 0:
            raise InvalidReply("no positive component-type count")
        if not all(isinstance(e.get("components"), list) and all(isinstance(c, str) for c in e["components"]) for e in entries):
            raise InvalidReply("invalid component names")
        summary.update(known_types=value["known_types"], components=sorted({c for e in entries for c in e["components"]}))
    return summary


class Investigation:
    def __init__(self, client, pause=time.sleep):
        self.client = client
        self.pause = pause
        self.available = set()
        self.report = {"started_utc": datetime.now(timezone.utc).isoformat(), "checks": [],
                       "scope": "Fixed read probes. Passing means protocol/shape checks passed, not all game semantics verified.",
                       "game_build": "not reported by current MCP; correlate with the loader log"}

    def tool(self, name, arguments, *, expected=None, label=None):
        if name not in READS:
            raise ValueError("not a research read")
        check = {"name": label or name, "tool": name, "arguments": arguments, "attempts_ms": []}
        self.report["checks"].append(check)
        if name not in self.available:
            check.update(outcome="unsupported", reason="not advertised by this endpoint")
            return None
        started = time.perf_counter()
        try:
            for attempt in range(3):
                tick = time.perf_counter()
                result = self.client.call("tools/call", {"name": name, "arguments": arguments})
                check["attempts_ms"].append(round((time.perf_counter() - tick) * 1000, 2))
                if not isinstance(result, dict) or type(result.get("isError")) is not bool:
                    raise InvalidReply("invalid tool-result envelope")
                content = result.get("content")
                if not isinstance(content, list) or len(content) != 1 or not isinstance(content[0], dict) or content[0].get("type") != "text" or not isinstance(content[0].get("text"), str):
                    raise InvalidReply("expected one text result from this MCP")
                message = content[0]["text"]
                if result["isError"] and "not loaded" in message.casefold() and attempt < 2 and expected is None:
                    self.pause(1.5)
                    continue
                if expected is not None:
                    ok = result["isError"] and expected in message.casefold()
                    check.update(outcome="pass" if ok else "fail", expected_refusal=expected)
                    return None
                if result["isError"]:
                    # Keep the error category, not arbitrary server text that could include identities.
                    reason = "area not loaded after retries" if "not loaded" in message.casefold() else "tool returned an error; inspect the mod log"
                    check.update(outcome="fail", reason=reason)
                    return None
                value = json.loads(message)
                check.update(outcome="pass", observation=summarize(name, value))
                return value
        except (ValueError, TypeError, KeyError) as error:
            check.update(outcome="fail", reason=str(error)[:180])
            return None
        except OSError:
            check.update(outcome="blocked", reason="transport failed")
            raise
        finally:
            check["elapsed_ms"] = round((time.perf_counter() - started) * 1000, 2)
            self.pause(0.25)

    def run(self, center=None):
        if center is not None and not valid_center(center):
            raise ValueError("center must be finite and inside current project bounds")
        try:
            init = self.client.call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                                   "clientInfo": {"name": "enshrouded-research", "version": "1"}})
            if not isinstance(init, dict) or not isinstance(init.get("serverInfo"), dict) or init["serverInfo"].get("name") != "enshrouded-mcp":
                raise InvalidReply("endpoint is not the expected Enshrouded MCP")
            self.report["server"] = init["serverInfo"]
            self.report["protocol"] = init.get("protocolVersion")
            listing = self.client.call("tools/list")
            advertised = listing.get("tools") if isinstance(listing, dict) else None
            if not isinstance(advertised, list) or not all(isinstance(t, dict) and isinstance(t.get("name"), str) and isinstance(t.get("inputSchema"), dict) for t in advertised):
                raise InvalidReply("invalid tool discovery result")
            self.available = {t["name"] for t in advertised}
            self.report["advertised_tools"] = sorted(self.available)
            schema = sorted(({"name": t["name"], "schema": t["inputSchema"]} for t in advertised), key=lambda t: t["name"])
            self.report["schema_sha256"] = hashlib.sha256(json.dumps(schema, sort_keys=True).encode()).hexdigest()
            self.report["unprobed_tools"] = sorted(self.available.difference(READS))
            status = self.tool("status", {})
            self.tool("status", {"research_unknown": True}, expected="unknown argument", label="unknown_argument_refused")
            self.tool("voxel_sample", {"x": -1, "y": 0, "z": 0}, expected="outside the world", label="outside_world_refused")
            if status is None or not status["hosting"]:
                self.report["incomplete"] = "hosting not confirmed; world reads not attempted"
                return self.finish()
            players = self.tool("players", {})
            if center is None:
                if players is not None and len(players) == 1 and valid_center(players[0].get("position")):
                    center = players[0]["position"]
                    self.report["center_source"] = "sole connected player"
                else:
                    self.report["incomplete"] = "provide --center X Y Z, or run with exactly one positioned player"
                    return self.finish()
            else:
                self.report["center_source"] = "explicit coordinates"
            self.report["center_metres"] = list(center)
            x, y, z = center
            self.tool("voxel_sample", {"x": x, "y": max(0, y - 0.5), "z": z})
            box = {}
            for axis, value, edge in zip("xyz", center, (10240, 1024, 10240)):
                box[axis + "1"] = max(0, value - 2)
                box[axis + "2"] = min(edge, value + 2)
            self.tool("props_in", box)
            for kind in ("placed", "used_item", "static_used_item", "containers"):
                self.tool("entities_near", {"x": x, "y": y, "z": z, "radius": 3, "kind": kind}, label="entities_near_" + kind)
            self.tool("entity_components", {"x": x, "y": y, "z": z, "radius": 3, "limit": 5})
        except (OSError, urllib.error.URLError) as error:
            self.report["blocked"] = f"transport unavailable ({type(error).__name__}); check tunnel and session network permissions"
        except (ValueError, TypeError, KeyError) as error:
            self.report["failure"] = str(error)[:180]
        return self.finish()

    def finish(self):
        outcomes = {c["outcome"] for c in self.report["checks"]}
        if "failure" in self.report or "fail" in outcomes:
            verdict = "failed"
        elif "blocked" in self.report:
            verdict = "blocked"
        elif "incomplete" in self.report or "unsupported" in outcomes:
            verdict = "incomplete"
        else:
            verdict = "passed"
        self.report["verdict"] = verdict
        return self.report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8752, help="existing local tunnel port, default 8752")
    parser.add_argument("--center", nargs=3, type=float, metavar=("X", "Y", "Z"))
    args = parser.parse_args()
    if not 1 <= args.port <= 65535:
        parser.error("port must be 1..65535")
    if args.center is not None and not valid_center(args.center):
        parser.error("center must be finite and inside current project bounds")
    investigation = Investigation(Client(args.port, os.environ.get("ENSHROUDED_MCP_TOKEN", "")))
    report = investigation.run(args.center)
    directory = Path(__file__).resolve().parents[1] / "gamedata" / "investigations"
    directory.mkdir(parents=True, exist_ok=True)
    output = directory / (datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ") + ".json")
    output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"{report['verdict']}: {len(report['checks'])} checks; report: {output}")
    return {"passed": 0, "failed": 1, "blocked": 2, "incomplete": 2}[report["verdict"]]


if __name__ == "__main__":
    raise SystemExit(main())
