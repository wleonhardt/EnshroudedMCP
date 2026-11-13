"""Companion probe behavior without sockets or game data."""

import json
import unittest

from client_probe import Probe, READS


class Fixture:
    def __init__(self, broken=None, stale=False, localized=True, selection_error=None):
        self.broken, self.stale, self.localized = broken, stale, localized
        self.calls = []
        self.selection_error = selection_error

    def call(self, method, params=None):
        self.calls.append((method, params))
        if method == "initialize":
            return {"serverInfo": {"name": "enshrouded-mcp", "version": "0.2.0"}, "protocolVersion": "2025-06-18"}
        if method == "tools/list":
            return {"tools": [{"name": name} for name in READS | {"client_press", "future_write", "give_item"}]}
        name, args = params["name"], params["arguments"]
        error = False
        if "probe_unknown" in args:
            error, value = True, "unknown argument"
        elif name == "client_recipes" and args.get("limit") == 9:
            error, value = True, "argument 'limit' must be at most 8"
        elif name == "client_capabilities":
            value = {"role": "optional_client_companion", "version": "0.2.0", "enshwick": "1.0.0-alpha.14",
                     "ui_observation": True, "local_input": True, "recipe_layout": True, "live_verified": False,
                     "client_event_sources": {"open_hud_screen": True, "station_recipe_selection": True},
                     "semantic_gameplay_actions": []}
        elif name == "client_state":
            value = {"age_ms": 2000 if self.stale else 0, "stale": self.stale, "sequence": 1,
                     "observation": {"screen": "private-screen", "prompt": {"title": "private-prompt"}, "host": "private-session"}}
        elif name == "client_events":
            value = {"events": [{"sequence": 1, "host": "private-session", "screen": "private-screen"}],
                     "next_after": 1, "latest_sequence": 1, "dropped_total": 0, "cursor_gap": False}
        elif name == "client_selection_events":
            value = {"events": [{"sequence": 1, "scope": "client_world", "event": {
                "kind": "open_hud_screen_event", "requested_screen": "private-tab", "host": 123}},
                {"sequence": 2, "scope": "client_world", "event": {
                    "kind": "station_recipe_selection_event", "operator": 123, "station": 456, "recipe_id": 789,
                    "category_choices": [{"category_id": 99, "selected_index": 1}], "choices_truncated": False}}],
                "next_after": 2, "latest_sequence": 2, "dropped_history": 0, "cursor_gap": False,
                "current_selected_tab": None, "current_selected_recipe": None,
                "state_status": "not observed", "evidence_scope": "events only", "identity_scope": "client only"}
            if self.selection_error == "tab":
                value["current_selected_tab"] = "Crafting"
            elif self.selection_error == "recipe":
                value["current_selected_recipe"] = 789
            elif self.selection_error == "scope":
                value["events"][0]["scope"] = "server_world"
        elif name == "client_item":
            value = {"debug_name": "Prop_Storage_24_T1_Box", "item_id": 213939880,
                     "localized_name": "private-localized" if self.localized else None,
                     "source": "vanilla_archive_and_client_localization", "effective_definition_verified": False}
        elif name == "client_recipes":
            guid = "11111111-1111-1111-1111-111111111111"
            value = {"mode": "report_only", "source": "vanilla_archive_recipe_registry", "effective_definition_verified": False,
                     "selected_output_guid": guid, "total_matches": 1, "truncated": False, "unverified": ["fixture"],
                     "recipes": [{"recipe_guid": guid, "workshop_guid": guid, "debug_name": "private-recipe",
                                  "inputs": [{"item_guid": "00000000-0000-0000-0000-000000000000", "count": 0}],
                                  "outputs": [{"item_guid": guid, "count": 1}], "ingredient_category_semantics": None,
                                  "catalyst_semantics": None, "required_props": None, "unlock": None, "feasible": None}]}
        if self.broken == name:
            if error:
                value = "tool unavailable"
            elif name == "client_events":
                value["next_after"] = 0
            elif name == "client_selection_events":
                value["events"][0]["scope"] = "server_world"
            elif name == "client_recipes":
                value["recipes"][0]["feasible"] = True
            elif name == "client_state":
                value["stale"] = "false"
            elif name == "client_item":
                value["item_id"] = 1
            else:
                value["semantic_gameplay_actions"] = ["press"]
        return {"isError": error, "content": [{"type": "text", "text": value if error else json.dumps(value)}]}


class ClientProbeTests(unittest.TestCase):
    def test_fixed_reads_only_and_no_sensitive_values_saved(self):
        client = Fixture()
        report = Probe(client, pause=lambda _: None).run()
        self.assertEqual(report["verdict"], "incomplete")
        self.assertEqual(len(report["checks"]), 9)
        self.assertTrue(all(check["outcome"] == "pass" for check in report["checks"]))
        serialized = json.dumps(report)
        self.assertNotIn("private-", serialized)
        self.assertNotIn("11111111-1111", serialized)
        self.assertEqual({params["name"] for method, params in client.calls if method == "tools/call"}, READS)

    def test_stale_and_missing_localization_remain_explicitly_incomplete(self):
        report = Probe(Fixture(stale=True, localized=False), pause=lambda _: None).run()
        self.assertEqual(report["verdict"], "incomplete")
        state = next(check for check in report["checks"] if check["name"] == "state_shape")
        item = next(check for check in report["checks"] if check["name"] == "storage_box_localization")
        self.assertTrue(state["observation"]["stale"])
        self.assertFalse(item["observation"]["localized_name_available"])

    def test_nonempty_client_events_remain_unverified_and_sanitized(self):
        report = Probe(Fixture(), pause=lambda _: None).run()
        selection = next(check for check in report["checks"] if check["name"] == "selection_event_evidence")
        self.assertEqual(selection["outcome"], "pass")
        self.assertEqual(selection["observation"]["returned"], 2)
        self.assertFalse(selection["observation"]["current_selection_verified"])
        serialized = json.dumps(selection)
        for private in ("private-tab", "recipe_id", "category_id", "requested_screen", "789"):
            self.assertNotIn(private, serialized)

    def test_events_cannot_claim_current_selection_or_server_scope(self):
        for error in ("tab", "recipe", "scope"):
            with self.subTest(error=error):
                report = Probe(Fixture(selection_error=error), pause=lambda _: None).run()
                selection = next(check for check in report["checks"] if check["name"] == "selection_event_evidence")
                self.assertEqual(selection["outcome"], "fail")

    def test_shape_cursor_identity_and_feasibility_errors_fail(self):
        for name in READS:
            with self.subTest(name=name):
                report = Probe(Fixture(broken=name), pause=lambda _: None).run()
                self.assertEqual(report["verdict"], "failed")

    def test_unrelated_errors_do_not_prove_specific_refusals(self):
        for name, label in (("client_state", "unknown_argument_refused"), ("client_recipes", "invalid_recipe_limit_refused")):
            report = Probe(Fixture(broken=name), pause=lambda _: None).run()
            self.assertEqual(next(check for check in report["checks"] if check["name"] == label)["outcome"], "fail")

    def test_actions_are_refused_before_call(self):
        client = Fixture()
        with self.assertRaises(ValueError):
            Probe(client).tool("action", "client_press", {"dry": True})
        self.assertEqual(client.calls, [])


if __name__ == "__main__":
    unittest.main()
