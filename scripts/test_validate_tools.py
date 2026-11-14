"""Fixed-probe behavior tests without game access."""

import json
import unittest
from unittest.mock import patch

from validate_tools import READS, SERVER_TOOLS, Validation


class Fixture:
    def __init__(self, broken=None):
        self.broken = broken
        self.calls = []

    def call(self, method, params=None):
        self.calls.append((method, params))
        if method == "initialize":
            return {"serverInfo": {"name": "enshrouded-mcp", "version": "0.16.0"}, "protocolVersion": "2025-06-18"}
        if method == "tools/list":
            return {"tools": [{"name": name} for name in SERVER_TOOLS | {"future_write"}]}
        name, args = params["name"], params["arguments"]
        error = False
        if "validation_unknown" in args:
            error, value = True, "unknown argument"
        elif name == "item_info":
            error = args["item"] != "Prop_Storage_24_T1_Box"
            value = "no vanilla item" if error else {"guid": "fixture", "metadata": {"item_id": 213939880},
                "placement": {"state": "observed", "template_guid": "000102030405060708090a0b0c0d0eff"}}
        elif name == "item_lookup":
            error = args["ids"] == [0]
            value = "ids must contain nonzero u32 values" if error else {
                "source": "vanilla_archive_item_registry", "effective_definition_verified": False,
                "results": [{"item_id": 213939880, "state": "single_match", "total_matches": 1,
                             "truncated": False, "matches": [{"guid": "fixture", "debug_name": "Prop_Storage_24_T1_Box"}]}]}
        elif name == "inventory_read":
            error = args.get("entity") != 4294967295
            if not error:
                value = {"slots": [], "total_slots": 0, "entity": "private-id", "world_key": "private-world"}
            elif not args:
                value = "provide exactly one of player or entity"
            elif args.get("entity") == 0:
                value = "argument 'entity' must be at least 1"
            else:
                value = "no connected player"
        elif name == "capabilities":
            value = {"mod_version": "0.16.0", "inventory_read": True, "item_info": True, "item_lookup": True,
                     "typed_network_health": True, "new_inspection_tools_live_verified": False}
        elif name == "entity_health":
            value = {"entities": [], "total_candidates": 0, "truncated": False}
        elif name == "knowledge_search":
            value = {"applicability_checked_against_current_world": False, "records": [
                {"applicability": {}, "status": "hypothesis", "next_test": "test", "evidence": [
                    {"source": "source", "checked": "date", "method": "method"}]}]}
        elif name == "recipe_plan":
            value = {"mode": "report_only", "provenance": "caller_supplied_unverified", "feasible_under_supplied_assumptions": True,
                     "steps": [{"batches": 2, "produced": 4}], "planned_surplus": [{"item": "boards", "count": 1}],
                     "stock_reserved": [{"item": "wood", "count": 6}], "missing_raw_items": []}
        elif name == "building_operation":
            error, value = True, "unknown construction operation"
        elif name == "building_survey":
            error = True
            value = "unknown argument 'confirm'" if "confirm" in args else "'x' must be a 0.5 m grid value from 0 to 32768"
        elif name == "building_plan":
            value = {"executable": False, "geometry": {"solid_cell_count": 16}, "cell_minima": [[x, 0, z] for x in range(4) for z in range(4)]}
        if name == self.broken and not error:
            if name == "recipe_plan":
                value["steps"][0]["batches"] = 1
            elif name == "building_plan":
                value["cell_minima"][-1] = value["cell_minima"][0]
            elif name == "item_lookup":
                value["results"][0]["matches"][0]["debug_name"] = "Wrong_Item"
            else:
                value = {"unexpected": "private-do-not-persist"}
        return {"isError": error, "content": [{"type": "text", "text": value if error else json.dumps(value)}]}


class ValidationTests(unittest.TestCase):
    def test_storage_mapping_must_be_observed_nonzero_file_order_hex(self):
        for state, template in [("unavailable", None), ("known_no_template", None),
                                ("observed", None), ("observed", "0" * 32),
                                ("observed", "z" * 32), ("observed", "A" * 32),
                                ("observed", "1" * 31)]:
            client = Fixture()
            original = client.call

            def reply(method, params=None):
                result = original(method, params)
                if method == "tools/call" and params["name"] == "item_info" and not result["isError"]:
                    value = json.loads(result["content"][0]["text"])
                    value["placement"] = {"state": state, "template_guid": template}
                    result["content"][0]["text"] = json.dumps(value)
                return result

            with self.subTest(state=state, template=template), patch.object(client, "call", side_effect=reply):
                report = Validation(client, pause=lambda _: None).run()
                check = next(c for c in report["checks"] if c["name"] == "known_static_item")
                self.assertEqual(check["outcome"], "fail")

    def test_unrelated_errors_do_not_prove_specific_refusals(self):
        for label in ("invalid_item_refused", "missing_inventory_selector", "invalid_inventory_selector", "invalid_lookup_id_refused",
                      "survey_off_grid_refused", "survey_write_argument_refused", "unknown_construction_operation_refused"):
            client = Fixture()
            original = client.call

            def reply(method, params=None):
                if method == "tools/call":
                    name, args = params["name"], params["arguments"]
                    target = ((label == "invalid_item_refused" and name == "item_info" and args["item"].startswith("Not_")) or
                              (label == "missing_inventory_selector" and name == "inventory_read" and not args) or
                              (label == "invalid_inventory_selector" and name == "inventory_read" and args.get("entity") == 0) or
                              (label == "invalid_lookup_id_refused" and name == "item_lookup" and args["ids"] == [0]) or
                              (label == "survey_off_grid_refused" and name == "building_survey" and args.get("x") == 100.25) or
                              (label == "survey_write_argument_refused" and name == "building_survey" and "confirm" in args) or
                              (label == "unknown_construction_operation_refused" and name == "building_operation"))
                    if target:
                        return {"isError": True, "content": [{"type": "text", "text": "tool unavailable"}]}
                return original(method, params)

            with self.subTest(label=label), patch.object(client, "call", side_effect=reply):
                report = Validation(client, pause=lambda _: None).run()
                self.assertEqual(next(c for c in report["checks"] if c["name"] == label)["outcome"], "fail")

    def test_fixed_reads_only_and_sanitized_incomplete_report(self):
        client = Fixture()
        report = Validation(client, pause=lambda _: None).run()
        self.assertEqual(report["verdict"], "incomplete")
        self.assertEqual(len(report["checks"]), 18)
        self.assertTrue(all(c["outcome"] == "pass" for c in report["checks"]))
        self.assertTrue(report["incomplete"])
        self.assertNotIn("private-", json.dumps(report))
        called = {params["name"] for method, params in client.calls if method == "tools/call"}
        self.assertEqual(called, READS)
        self.assertTrue({"building_place_cell", "building_apply_plan", "building_accept", "building_undo_cell"}.isdisjoint(called))
        self.assertTrue(any("do not establish live placement" in note for note in report["incomplete"]))
        self.assertFalse(any("dry" in params.get("arguments", {}) for method, params in client.calls if method == "tools/call"))

    def test_shape_arithmetic_provenance_and_geometry_mismatches_fail(self):
        for tool in ("capabilities", "item_info", "item_lookup", "inventory_read", "entity_health", "knowledge_search", "recipe_plan", "building_plan"):
            with self.subTest(tool=tool):
                report = Validation(Fixture(tool), pause=lambda _: None).run()
                self.assertEqual(report["verdict"], "failed")
                self.assertNotIn("private-", json.dumps(report))

    def test_survey_refusals_do_not_establish_geometry_acceptance(self):
        client = Fixture()
        report = Validation(client, pause=lambda _: None).run()
        surveys = [params["arguments"] for method, params in client.calls
                   if method == "tools/call" and params["name"] == "building_survey"]
        self.assertEqual(len(surveys), 2)
        self.assertTrue(all(args["x"] == 100.25 or "confirm" in args for args in surveys))
        self.assertEqual(report["verdict"], "incomplete")
        self.assertTrue(any("no valid survey read is validated" in note for note in report["incomplete"]))
        self.assertTrue(any("persistence acceptance remain incomplete" in note for note in report["incomplete"]))

    def test_unknown_operation_probe_retries_only_bounded_inactive_session(self):
        for state, inactive_count, expected, calls in [
            ("inactive_session_unknown", 1, "pass", 2),
            ("inactive_session_unknown", 10, "fail", 3),
            ("stale_session_unknown", 1, "fail", 1),
            ("failed_unknown", 1, "fail", 1),
        ]:
            client = Fixture()
            original = client.call
            attempts = []

            def reply(method, params=None):
                if method == "tools/call" and params["name"] == "building_operation":
                    attempts.append(1)
                    if len(attempts) <= inactive_count:
                        return {"isError": False, "content": [{"type": "text", "text": json.dumps({"state": state})}]}
                return original(method, params)

            with self.subTest(state=state, count=inactive_count), patch.object(client, "call", side_effect=reply):
                report = Validation(client, pause=lambda _: None).run()
                check = next(c for c in report["checks"] if c["name"] == "unknown_construction_operation_refused")
                self.assertEqual(check["outcome"], expected)
                self.assertEqual(len(attempts), calls)

    def test_advertised_writes_never_authorize_a_write_probe(self):
        client = Fixture()
        original = client.call
        advertised = READS | {"status", "players", "voxel_sample", "props_in", "entities_near",
                              "entity_components", "give_item", "building_place_cell", "building_apply_plan",
                              "building_accept", "building_undo_cell", "player_reposition", "future_write"}

        def reply(method, params=None):
            if method == "initialize":
                return {"serverInfo": {"name": "enshrouded-mcp", "version": "0.16.0"}, "protocolVersion": "2025-06-18"}
            if method == "tools/list":
                return {"tools": [{"name": name} for name in advertised]}
            return original(method, params)

        with patch.object(client, "call", side_effect=reply):
            report = Validation(client, pause=lambda _: None).run()
        self.assertEqual(report["verdict"], "incomplete")
        self.assertEqual({params["name"] for method, params in client.calls if method == "tools/call"}, READS)
        self.assertEqual(len(SERVER_TOOLS), 22)
        self.assertIn("building_accept", SERVER_TOOLS)
        self.assertNotIn("building_accept", READS)
        self.assertNotIn("player_reposition", READS)
        self.assertTrue(all(check["outcome"] == "pass" for check in report["checks"]))

    def test_missing_accept_discovery_fails_without_invoking_any_tool(self):
        client = Fixture()
        original = client.call

        def reply(method, params=None):
            if method == "tools/list":
                return {"tools": [{"name": name} for name in SERVER_TOOLS - {"building_accept"}]}
            return original(method, params)

        with patch.object(client, "call", side_effect=reply):
            report = Validation(client, pause=lambda _: None).run()
        self.assertEqual(report["verdict"], "failed")
        self.assertFalse(any(method == "tools/call" for method, _ in client.calls))

    def test_write_rejected_before_client_call(self):
        client = Fixture()
        for name in ("give_item", "building_place_cell", "building_apply_plan", "building_accept", "building_undo_cell"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                Validation(client).probe("write", name, {"dry": True})
        self.assertEqual(client.calls, [])


if __name__ == "__main__":
    unittest.main()
