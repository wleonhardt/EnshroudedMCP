"""Read-only 0.23.2 transport/shape probes; never establishes fixture semantics."""

import argparse
from datetime import datetime, timezone
import json
import math
import os
from pathlib import Path
import time

from investigate import Client, InvalidReply


READS = frozenset(("status", "capabilities", "inventory_read", "item_info", "entity_health",
                   "knowledge_search", "recipe_plan", "building_plan", "building_survey", "building_operation", "item_lookup"))
# Discovery checks the known surface, but only READS may ever be invoked.
SERVER_TOOLS = READS | frozenset(("building_access", "player_restore_health", "enemy_pacification", "status", "players", "voxel_sample", "props_in", "entities_near",
                                  "entity_components", "give_item", "building_place_cell", "building_apply_plan",
                                  "building_accept", "building_undo_cell", "player_reposition"))
RECIPE = {"catalog_label": "synthetic-validation", "goals": [{"item": "boards", "count": 3}],
          "stock": [{"item": "wood", "count": 6}], "stations": [], "unlocks": [],
          "recipes": [{"id": "boards", "item": "boards", "count": 2,
                       "ingredients": [{"item": "wood", "count": 3}], "stations": [], "unlocks": []}]}
FLOOR = {"kind": "floor", "x": 100, "y": 100, "z": 100, "width": 2,
         "height": 0.5, "depth": 2, "material": "unverified-validation-material"}


def require(condition):
    if not condition:
        raise InvalidReply("response did not match this fixed probe")


def observed_template(value):
    placement = value.get("placement")
    if not isinstance(placement, dict):
        return False
    template = placement.get("template_guid")
    return (placement.get("state") == "observed" and isinstance(template, str)
            and len(template) == 32 and set(template) <= set("0123456789abcdef")
            and template != "0" * 32)


def clock_observation(value):
    clock = value.get("clock")
    if not isinstance(clock, dict) or clock.get("source") != "published_server_clock" or clock.get("rendered_lighting_verified") is not False:
        return False
    if clock.get("state") == "unavailable":
        return "observation" in clock and clock["observation"] is None
    now = clock.get("observation")
    return (clock.get("state") == "observed" and isinstance(now, dict) and type(now.get("is_day")) is bool and
            all(type(now.get(key)) in (int, float) and math.isfinite(now[key]) for key in
                ("hours", "day_starts_hours", "night_starts_hours", "day_duration_seconds", "night_duration_seconds", "speed")))


class Validation:
    def __init__(self, client, pause=time.sleep):
        self.client, self.pause = client, pause
        self.checks = []

    def probe(self, label, name, args, predicate=None, refusal=None):
        if name not in READS:
            raise ValueError("only fixed read tools are allowed")
        check = {"name": label, "outcome": "fail"}
        self.checks.append(check)
        try:
            for attempt in range(3):
                reply = self.client.call("tools/call", {"name": name, "arguments": args})
                require(isinstance(reply, dict) and type(reply.get("isError")) is bool)
                content = reply.get("content")
                require(isinstance(content, list) and len(content) == 1 and content[0].get("type") == "text")
                message = content[0]["text"]
                require(isinstance(message, str))
                # A restarted world may not have emitted its loaded callback yet.
                # Retry only this fixed status probe and this exact readiness state.
                if (label == "unknown_construction_operation_refused" and name == "building_operation" and
                        not reply["isError"] and json.loads(message).get("state") == "inactive_session_unknown" and attempt < 2):
                    self.pause(0.5)
                    continue
                break
            if refusal is not None:
                require(reply["isError"] and refusal in message.casefold())
            else:
                require(not reply["isError"])
                value = json.loads(message)
                require(isinstance(value, dict) and predicate(value))
            check["outcome"] = "pass"
        except OSError:
            check["outcome"] = "blocked"
        except (ValueError, TypeError, KeyError, AttributeError):
            pass  # Do not persist arbitrary server text or identities.
        finally:
            self.pause(0.25)

    def run(self):
        try:
            init = self.client.call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                                    "clientInfo": {"name": "enshrouded-validation", "version": "1"}})
            require(init["serverInfo"] == {"name": "enshrouded-mcp", "version": "0.23.2"})
            require(init.get("protocolVersion") == "2025-06-18")
            listing = self.client.call("tools/list")
            require(SERVER_TOOLS.issubset({t["name"] for t in listing["tools"]}))
            self.checks.append({"name": "discovery_0.23.2", "outcome": "pass"})
        except (OSError, ValueError, TypeError, KeyError):
            return self.report("failed", "discovery failed; no probes attempted")
        self.probe("capability_shapes", "capabilities", {}, lambda v: v.get("mod_version") == "0.23.2" and
                   v.get("compiled_framework_revision") == "acd5550c607a38c983433f9807cc95564244fe0b" and
                   v.get("compiled_framework_abi") == "11.30" and v.get("enshwick") == "1.0.0-alpha.15" and
                   all(type(v.get(k)) is bool for k in ("inventory_read", "item_info", "item_lookup", "typed_network_health", "new_inspection_tools_live_verified")))
        self.probe("world_clock_shape", "status", {}, clock_observation)
        self.probe("known_item_lookup", "item_lookup", {"ids": [213939880]}, lambda v:
                   v.get("source") == "vanilla_archive_item_registry" and v.get("effective_definition_verified") is False and
                   len(v["results"]) == 1 and v["results"][0]["item_id"] == 213939880 and
                   v["results"][0]["state"] == "single_match" and v["results"][0]["total_matches"] == 1 and
                   v["results"][0]["truncated"] is False and len(v["results"][0]["matches"]) == 1 and
                   v["results"][0]["matches"][0]["debug_name"] == "Prop_Storage_24_T1_Box")
        self.probe("invalid_lookup_id_refused", "item_lookup", {"ids": [0]}, refusal="ids must contain nonzero u32 values")
        self.probe("known_static_item", "item_info", {"item": "Prop_Storage_24_T1_Box"},
                   lambda v: v["metadata"]["item_id"] == 213939880 and isinstance(v.get("guid"), str)
                   and observed_template(v))
        self.probe("invalid_item_refused", "item_info", {"item": "Not_A_Vanilla_Item_Validation"}, refusal="no vanilla item")
        self.probe("missing_inventory_selector", "inventory_read", {}, refusal="provide exactly one of player or entity")
        self.probe("invalid_inventory_selector", "inventory_read", {"entity": 0}, refusal="argument 'entity' must be at least 1")
        self.probe("unmatched_player", "inventory_read", {"player": "__mcp_validation_no_such_player__"}, refusal="no connected player")
        self.probe("arbitrary_entity_empty_inventory", "inventory_read", {"entity": 4294967295, "limit": 1},
                   lambda v: v.get("slots") == [] and v.get("total_slots") == 0)
        self.probe("empty_health_shape", "entity_health", {"x": 5120, "y": 100, "z": 5120, "radius": 1, "limit": 1},
                   lambda v: v.get("entities") == [] and v.get("total_candidates") == 0 and v.get("truncated") is False)
        self.probe("knowledge_provenance", "knowledge_search", {}, lambda v:
                   v.get("applicability_checked_against_current_world") is False and bool(v["records"]) and
                   all(isinstance(r.get("applicability"), dict) and r.get("status") and r.get("next_test") and
                       r.get("evidence") and all(all(s.get(k) for k in ("source", "checked", "method")) for s in r["evidence"])
                       for r in v["records"]))
        self.probe("synthetic_recipe_arithmetic", "recipe_plan", RECIPE, lambda v:
                   v.get("mode") == "report_only" and v.get("provenance") == "caller_supplied_unverified" and
                   v.get("feasible_under_supplied_assumptions") is True and len(v["steps"]) == 1 and
                   v["steps"][0]["batches"] == 2 and v["steps"][0]["produced"] == 4 and
                   v["planned_surplus"] == [{"item": "boards", "count": 1}] and
                   v["stock_reserved"] == [{"item": "wood", "count": 6}] and v["missing_raw_items"] == [])
        self.probe("floor_geometry", "building_plan", FLOOR, lambda v:
                   v.get("executable") is False and v["geometry"]["solid_cell_count"] == 16 and
                   len(v["cell_minima"]) == 16 and len({tuple(p) for p in v["cell_minima"]}) == 16)
        # Admission-only probes: neither request reaches game-thread survey reads.
        survey = {"x": 100, "y": 100, "z": 100, "width": 0.5, "height": 0.5, "depth": 0.5}
        self.probe("survey_off_grid_refused", "building_survey", {**survey, "x": 100.25},
                   refusal="'x' must be a 0.5 m grid value")
        self.probe("survey_write_argument_refused", "building_survey", {**survey, "confirm": "server-write"},
                   refusal="unknown argument 'confirm'")
        self.probe("unknown_construction_operation_refused", "building_operation",
                   {"operation": "validation-no-such-operation"}, refusal="unknown construction operation")
        self.probe("unknown_argument_refused", "capabilities", {"validation_unknown": True}, refusal="unknown argument")
        outcomes = {c["outcome"] for c in self.checks}
        return self.report("failed" if "fail" in outcomes else "blocked" if "blocked" in outcomes else "incomplete")

    def report(self, verdict, reason=None):
        result = {"started_utc": datetime.now(timezone.utc).isoformat(), "verdict": verdict, "checks": self.checks,
                  "incomplete": ["Known stock and inventory eligibility versus UI remain unverified.",
                                 "Known healthy/damaged fixture versus UI remains unverified.",
                                 "Synthetic planning and empty reads do not establish live game semantics.",
                                 "Survey probes cover off-grid and unknown write-argument refusal only; no valid survey read is validated.",
                                 "Live building geometry comparison and persistence acceptance remain incomplete.",
                                 "Construction status refusal and shape checks do not establish live placement, read-back, undo or reload acceptance."]}
        if reason:
            result["reason"] = reason
        return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8753)
    args = parser.parse_args()
    if not 1 <= args.port <= 65535:
        parser.error("port must be 1..65535")
    report = Validation(Client(args.port, os.environ.get("ENSHROUDED_MCP_TOKEN", ""))).run()
    directory = Path(__file__).resolve().parents[1] / "gamedata" / "validations"
    directory.mkdir(parents=True, exist_ok=True)
    output = directory / (datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ") + ".json")
    output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"{report['verdict']}: {len(report['checks'])} checks; report: {output}")
    return 1 if report["verdict"] == "failed" else 2


if __name__ == "__main__":
    raise SystemExit(main())
