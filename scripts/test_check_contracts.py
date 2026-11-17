"""Contract drift checks never execute advertised gameplay tools."""

import copy
import io
import json
from contextlib import redirect_stdout
import unittest
from unittest.mock import patch

from check_contracts import ROOT, compare_live, main, validate_index

INDEX = json.loads((ROOT / "plans/capability-contracts.json").read_text())


class Endpoint:
    def __init__(self, scope):
        self.scope = scope
        self.calls = []
        self.tools = [
            {"name": row["name"], "description": row["description"], "inputSchema": copy.deepcopy(row["input_schema"])}
            for row in INDEX["tools"] if row["scope"] == scope
        ][::-1]
        profile = INDEX["profiles"]["product"]
        self.capabilities = {
            "mod_version" if scope == "server" else "version": profile[scope + "_version"],
            "compiled_framework_revision": profile["sdk_revision"],
            "compiled_framework_abi": profile["abi"],
            "runtime_framework": {"source":"published_host_api_immutable_header", "state":"observed",
                                  "abi_major":12, "abi_minor":1, "api_table_bytes":4096,
                                  "scope":"header metadata only; signatures, API-call fidelity and gameplay are not verified"},
        }
        self.error = False

    def call(self, method, params=None):
        self.calls.append((method, params))
        if method == "tools/list":
            return {"tools": self.tools}
        expected = "capabilities" if self.scope == "server" else "client_capabilities"
        if method != "tools/call" or params != {"name": expected, "arguments": {}}:
            raise AssertionError("only fixed metadata tools may execute")
        return {"isError": self.error, "content": [{"type": "text", "text": json.dumps(self.capabilities)}]}


class ContractTests(unittest.TestCase):
    def test_runtime_header_remains_separate_from_compiled_provenance_and_trust(self):
        endpoint = Endpoint("server")
        endpoint.capabilities["runtime_framework"].update(abi_major=42, abi_minor=99, api_table_bytes=8192)
        observed = compare_live(INDEX, "server", endpoint)
        self.assertEqual((observed["abi_major"], observed["abi_minor"], observed["api_table_bytes"]), (42, 99, 8192))
        endpoint.capabilities["runtime_framework"].update(state="unavailable", abi_major=None, abi_minor=None, api_table_bytes=None)
        self.assertEqual(compare_live(INDEX, "server", endpoint)["state"], "unavailable")

    def test_missing_malformed_or_contradictory_runtime_header_refuses_without_echo(self):
        cases = [None, {}, {"state":"unavailable"}]
        valid = Endpoint("client").capabilities["runtime_framework"]
        for key, wrong in (("source", "untrusted marker"), ("state", "unknown"), ("abi_major", True),
                           ("abi_minor", -1), ("abi_minor", 65536), ("api_table_bytes", 4294967296),
                           ("scope", "verified"), ("api_table_bytes", None)):
            cases.append({**valid, key:wrong})
        cases.append({**valid, "state":"unavailable"})
        for value in cases:
            with self.subTest(value=value):
                endpoint = Endpoint("client")
                endpoint.capabilities["runtime_framework"] = value
                with self.assertRaisesRegex(ValueError, "runtime framework header") as error:
                    compare_live(INDEX, "client", endpoint)
                self.assertNotIn("untrusted marker", str(error.exception))

    def test_current_index_profiles_references_and_human_table_are_consistent(self):
        validate_index(INDEX, ROOT)

    def test_duplicate_missing_scope_count_schema_and_next_check_refuse(self):
        cases = [
            (lambda d: d["tools"].append(copy.deepcopy(d["tools"][0])), "duplicate"),
            (lambda d: d["tools"][0].update(scope="other"), "scope"),
            (lambda d: d["counts"].update(server=0), "count"),
            (lambda d: d["tools"][0].update(input_schema=None), "schema"),
            (lambda d: d["tools"][0].update(next_missing_check=""), "next check"),
            (lambda d: d["tools"][0].update(evidence_references=["plans/missing-contract-evidence.md"]), "reference"),
        ]
        for mutate, reason in cases:
            with self.subTest(reason=reason):
                data = copy.deepcopy(INDEX)
                mutate(data)
                with self.assertRaisesRegex(ValueError, reason):
                    validate_index(data, ROOT)

    def test_source_version_pin_and_toolchain_drift_refuse(self):
        for key in ["server_version", "client_version", "sdk_revision", "rust"]:
            with self.subTest(key=key):
                data = copy.deepcopy(INDEX)
                data["profiles"]["product"][key] = "different"
                with self.assertRaisesRegex(ValueError, key):
                    validate_index(data, ROOT)

    def test_discovery_order_is_irrelevant_and_only_metadata_executes(self):
        for scope in ["server", "client"]:
            endpoint = Endpoint(scope)
            compare_live(INDEX, scope, endpoint)
            self.assertEqual(endpoint.calls, [
                ("tools/list", None),
                ("tools/call", {"name": "capabilities" if scope == "server" else "client_capabilities", "arguments": {}}),
            ])

    def test_missing_unexpected_duplicate_description_and_schema_drift_refuse(self):
        cases = [
            (lambda e: e.tools.pop(), "tool names"),
            (lambda e: e.tools.append({"name": "future_write"}), "tool names"),
            (lambda e: e.tools.append(copy.deepcopy(e.tools[0])), "duplicate"),
            (lambda e: e.tools[0].update(description="untrusted response marker"), "description"),
            (lambda e: e.tools[0]["inputSchema"].update(additionalProperties=True), "schema"),
        ]
        for mutate, reason in cases:
            with self.subTest(reason=reason):
                endpoint = Endpoint("server")
                mutate(endpoint)
                with self.assertRaisesRegex(ValueError, reason) as failure:
                    compare_live(INDEX, "server", endpoint)
                self.assertNotIn("untrusted response marker", str(failure.exception))
                self.assertEqual(endpoint.calls, [("tools/list", None)])

    def test_compiled_version_revision_abi_and_capability_error_refuse(self):
        for scope in ["server", "client"]:
            for key in ["mod_version" if scope == "server" else "version", "compiled_framework_revision", "compiled_framework_abi"]:
                with self.subTest(scope=scope, key=key):
                    endpoint = Endpoint(scope)
                    endpoint.capabilities[key] = "different"
                    with self.assertRaisesRegex(ValueError, key):
                        compare_live(INDEX, scope, endpoint)
        endpoint = Endpoint("client")
        endpoint.error = True
        with self.assertRaisesRegex(ValueError, "capabilities response"):
            compare_live(INDEX, "client", endpoint)

    def test_offline_cli_never_constructs_a_network_client_or_claims_native_acceptance(self):
        output = io.StringIO()
        with patch("check_contracts.Client", side_effect=AssertionError("offline network access")), redirect_stdout(output):
            self.assertEqual(main([]), 0)
        report = json.loads(output.getvalue())
        self.assertEqual(report["live_contracts_checked"], [])
        self.assertFalse(report["native_behavior_verified"])
        self.assertFalse(report["runtime_provider_verified"])

    def test_live_cli_checks_only_requested_endpoints(self):
        endpoints = {12345: Endpoint("server"), 12346: Endpoint("client")}
        output = io.StringIO()
        with patch("check_contracts.Client", side_effect=endpoints.__getitem__), redirect_stdout(output):
            self.assertEqual(main(["--server-port", "12345", "--client-port", "12346"]), 0)
        self.assertEqual(json.loads(output.getvalue())["live_contracts_checked"], ["server", "client"])
        self.assertEqual(sum(len(e.calls) for e in endpoints.values()), 4)

    def test_unavailable_endpoint_does_not_echo_response_payload(self):
        output = io.StringIO()
        with patch("check_contracts.Client", side_effect=ConnectionError("untrusted response marker")), redirect_stdout(output):
            self.assertEqual(main(["--server-port", "12345"]), 2)
        self.assertNotIn("untrusted response marker", output.getvalue())
        self.assertEqual(json.loads(output.getvalue())["verdict"], "unavailable")


if __name__ == "__main__":
    unittest.main()
