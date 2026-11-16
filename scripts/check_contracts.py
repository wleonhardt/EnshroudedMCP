"""Check the capability index and optional loopback metadata discovery; never execute gameplay tools."""

import argparse
import json
from pathlib import Path
import tomllib

from investigate import Client

ROOT = Path(__file__).resolve().parents[1]
SCOPES = ("server", "client")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate_index(data, root):
    tools = data["tools"]
    require(isinstance(tools, list), "invalid contract tools")
    seen = set()
    table = (root / "plans/capability-matrix.md").read_text(encoding="utf-8")
    for tool in tools:
        scope, name = tool["scope"], tool["name"]
        require(scope in SCOPES, "invalid contract scope")
        require(isinstance(name, str) and name, "invalid contract name")
        require((scope, name) not in seen, "duplicate contract tool")
        seen.add((scope, name))
        require(isinstance(tool["input_schema"], dict), "invalid contract schema")
        require(isinstance(tool["description"], str), "invalid contract description")
        require(isinstance(tool.get("next_missing_check"), str) and tool["next_missing_check"], "missing next check")
        for key in ("source_definitions", "admission_sources", "evidence_references"):
            require(isinstance(tool[key], list) and tool[key], "missing contract reference")
            for rel in tool[key] + [tool["dispatch_source"]]:
                path = Path(rel)
                require(not path.is_absolute() and ".." not in path.parts and (root / path).is_file(), "invalid contract reference")
        row = f"| {scope} / `{name}` | {tool['thread']} | {tool['evidence_summary']} Next: {tool['next_missing_check']} |"
        require(row in table, f"human table drift: {scope}/{name}")
    for scope in SCOPES:
        require(sum(s == scope for s, _ in seen) == data["counts"][scope], f"{scope} contract count mismatch")
    profile = data["profiles"]["product"]
    actual = {
        "server_version": tomllib.loads((root / "crates/mcp/Cargo.toml").read_text())["package"]["version"],
        "client_version": tomllib.loads((root / "crates/client/Cargo.toml").read_text())["package"]["version"],
        "sdk_revision": tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["dependencies"]["enshwick"]["rev"],
        "rust": tomllib.loads((root / "rust-toolchain.toml").read_text())["toolchain"]["channel"],
    }
    for key, value in actual.items():
        require(profile[key] == value, f"source profile {key} mismatch")


def compare_live(data, scope, client):
    require(scope in SCOPES, "invalid endpoint scope")
    expected = {t["name"]: t for t in data["tools"] if t["scope"] == scope}
    reply = client.call("tools/list")
    require(isinstance(reply, dict) and isinstance(reply.get("tools"), list), "invalid tools/list response")
    actual = {}
    for tool in reply["tools"]:
        require(isinstance(tool, dict) and isinstance(tool.get("name"), str), "invalid tools/list name")
        name = tool["name"]
        require(name not in actual, f"{scope} duplicate advertised tool")
        actual[name] = tool
    require(actual.keys() == expected.keys(), f"{scope} tool names drift")
    for name, tool in expected.items():
        require(actual[name].get("inputSchema") == tool["input_schema"], f"{scope}/{name} schema drift")
        require(actual[name].get("description") == tool["description"], f"{scope}/{name} description drift")
    # Fixed allowlist: the index can never select an executable gameplay tool.
    capability = "capabilities" if scope == "server" else "client_capabilities"
    reply = client.call("tools/call", {"name": capability, "arguments": {}})
    require(isinstance(reply, dict) and reply.get("isError") is False, "invalid capabilities response")
    content = reply.get("content")
    require(isinstance(content, list) and len(content) == 1 and isinstance(content[0], dict)
            and content[0].get("type") == "text" and isinstance(content[0].get("text"), str), "invalid capabilities response")
    observed = json.loads(content[0]["text"])
    require(isinstance(observed, dict), "invalid capabilities response")
    profile = data["profiles"]["product"]
    fields = {
        "mod_version" if scope == "server" else "version": profile[scope + "_version"],
        "compiled_framework_revision": profile["sdk_revision"],
        "compiled_framework_abi": profile["abi"],
    }
    for key, expected_value in fields.items():
        require(observed.get(key) == expected_value, f"{scope} {key} mismatch")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server-port", type=int)
    parser.add_argument("--client-port", type=int)
    args = parser.parse_args(argv)
    ports = [("server", args.server_port), ("client", args.client_port)]
    if any(port is not None and not 1 <= port <= 65535 for _, port in ports):
        parser.error("ports must be 1..65535")
    checked = []
    try:
        data = json.loads((ROOT / "plans/capability-contracts.json").read_text(encoding="utf-8"))
        validate_index(data, ROOT)
        for scope, port in ports:
            if port is not None:
                compare_live(data, scope, Client(port))
                checked.append(scope)
    except (ValueError, KeyError, TypeError) as error:
        print(json.dumps({"verdict": "drift", "error": str(error), "live_contracts_checked": checked}))
        return 1
    except OSError as error:
        print(json.dumps({"verdict": "unavailable", "error": type(error).__name__, "live_contracts_checked": checked}))
        return 2
    print(json.dumps({"verdict": "consistent", "index_consistent": True, "source_profile_checked": True,
                      "live_contracts_checked": checked, "native_behavior_verified": False,
                      "runtime_provider_verified": False}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
