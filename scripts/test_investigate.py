"""Test probe decisions without a server, sockets, or copied game data."""

import io
import json
import unittest
from unittest.mock import Mock

from investigate import Client, Investigation, InvalidReply, READS, valid_center


class FakeClient:
    def __init__(self):
        self.calls = []
        self.tools = list(READS) + ["give_item", "future_write"]
        self.players = [{"name": "private-name", "key": "private-key", "position": [100, 10, 100]}]
        self.loading = 0
        self.fail_voxel = False
        self.hosting = True
        self.wrong_refusal = False

    def call(self, method, params=None):
        self.calls.append((method, params))
        if method == "initialize":
            return {"serverInfo": {"name": "enshrouded-mcp", "version": "fixture"}, "protocolVersion": "2025-06-18"}
        if method == "tools/list":
            return {"tools": [{"name": name, "inputSchema": {}} for name in self.tools]}
        name, args = params["name"], params["arguments"]
        message, error = None, False
        if "research_unknown" in args:
            message, error = "unknown argument 'research_unknown'", True
        elif args.get("x", 0) < 0:
            message, error = ("world unavailable" if self.wrong_refusal else "point is outside the world"), True
        elif name == "status":
            value = {"hosting": self.hosting, "world_key": "private-save-path", "players": len(self.players)}
        elif name == "players":
            value = self.players
        elif name == "voxel_sample":
            if self.loading:
                self.loading -= 1
                message, error = "voxel read failed: not loaded", True
            elif self.fail_voxel:
                message, error = "some other failure", True
            else:
                value = {"material": "Air", "fill": 0}
        elif name == "props_in":
            value = {"total": 0, "props": []}
        else:
            value = {"total": 1, "known_types": 1, "entities": [{"entity": "private-session-id", "components": ["Fixture"]}]}
        return {"isError": error, "content": [{"type": "text", "text": message if message is not None else json.dumps(value)}]}


class InvestigationTests(unittest.TestCase):
    def run_probe(self, client, center=None):
        return Investigation(client, pause=lambda _: None).run(center)

    def test_covers_reads_without_discovering_writes_into_execution(self):
        client = FakeClient()
        report = self.run_probe(client)
        self.assertEqual(report["verdict"], "passed")
        called = {args["name"] for method, args in client.calls if method == "tools/call"}
        self.assertEqual(called, set(READS))
        self.assertEqual(len(report["checks"]), 11)
        self.assertEqual(report["unprobed_tools"], ["future_write", "give_item"])
        self.assertNotIn("private-", json.dumps(report))
        props = next(c for c in report["checks"] if c["name"] == "props_in")
        self.assertIn("known-object", props["observation"]["coverage"])

    def test_not_loaded_retries_are_bounded_and_other_errors_are_not_retried(self):
        for loading, failing, attempts, verdict in [(2, False, 3, "passed"), (4, False, 3, "failed"), (0, True, 1, "failed")]:
            with self.subTest(loading=loading, failing=failing):
                client = FakeClient()
                client.loading, client.fail_voxel = loading, failing
                report = self.run_probe(client)
                check = next(c for c in report["checks"] if c["name"] == "voxel_sample")
                self.assertEqual(len(check["attempts_ms"]), attempts)
                self.assertEqual(report["verdict"], verdict)

    def test_does_not_guess_a_location_with_zero_or_multiple_players(self):
        for players in [[], [{"position": [1, 2, 3]}, {"position": [4, 5, 6]}]]:
            client = FakeClient()
            client.players = players
            self.assertEqual(self.run_probe(client)["verdict"], "incomplete")
            self.assertFalse(any(args.get("name") == "props_in" for method, args in client.calls if method == "tools/call"))

    def test_explicit_boundary_center_produces_in_bounds_box(self):
        for center in ([0, 0, 0], [10240, 1024, 10240]):
            report = self.run_probe(FakeClient(), center)
            self.assertEqual(report["verdict"], "passed")
            box = next(c["arguments"] for c in report["checks"] if c["name"] == "props_in")
            for suffix in ("1", "2"):
                self.assertTrue(valid_center([box[a + suffix] for a in "xyz"]))

    def test_missing_tool_and_no_world_are_incomplete(self):
        client = FakeClient()
        client.tools.remove("entity_components")
        self.assertEqual(self.run_probe(client)["verdict"], "incomplete")
        client = FakeClient()
        client.hosting = False
        self.assertEqual(self.run_probe(client)["verdict"], "incomplete")

    def test_unrelated_error_does_not_pass_expected_refusal(self):
        client = FakeClient()
        client.wrong_refusal = True
        self.assertEqual(self.run_probe(client)["verdict"], "failed")

    def test_transport_failure_is_blocked(self):
        client = Mock()
        client.call.side_effect = PermissionError("fixture socket restriction")
        self.assertEqual(self.run_probe(client)["verdict"], "blocked")

    def test_bad_center_is_refused_before_any_call(self):
        client = FakeClient()
        for center in ([float("nan"), 0, 0], [-1, 0, 0], [True, 1, 2]):
            with self.assertRaises(ValueError):
                self.run_probe(client, center)
        self.assertEqual(client.calls, [])

    def test_client_rejects_mismatched_rpc_ids(self):
        client = Client(8752)
        client.opener = Mock()
        client.opener.open.return_value = io.BytesIO(b'{"jsonrpc":"2.0","id":99,"result":{}}')
        with self.assertRaises(InvalidReply):
            client.call("initialize")


if __name__ == "__main__":
    unittest.main()
