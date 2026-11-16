"""Fixed read-only probes for native client companion 0.20.0; no actions."""

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import time

from investigate import Client, InvalidReply


READS = frozenset(("client_capabilities", "client_commands", "client_snapshot", "client_state", "client_events", "client_selection_events", "client_item", "client_recipes", "client_workshops"))
GUID = re.compile(r"[0-9a-fA-F]{8}(?:-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}\Z")


def integer(value):
    return type(value) is int and value >= 0


def require(condition):
    if not condition:
        raise InvalidReply("fixed client probe shape or value mismatch")


def capabilities(value):
    require(value.get("role") == "optional_client_companion" and value.get("version") == "0.20.0" and
            value.get("enshwick") == "1.0.0-alpha.17" and
            value.get("compiled_framework_revision") == "6a151a9550ed7a6ade99c23fa817a91e25b71bc9" and
            value.get("compiled_framework_abi") == "12.1" and value.get("live_verified") is False and
            value.get("screen_host_template_observation") is True and value.get("semantic_gameplay_actions") == (["client_command.look", "client_command.move"] if value.get("native_client_commands") else []) and
            type(value.get("native_map_marker_ui")) is bool and value.get("native_map_marker_rendered_verified") is False and value.get("native_command_live_verified") is False and
            value.get("semantic_ui_actions") == ((["client_map_markers"] if value["native_map_marker_ui"] else []) + (["client_command"] if value.get("native_client_commands") else [])) and
            all(type(value.get(key)) is bool for key in ("ui_observation", "local_input", "recipe_layout", "workshop_layout", "live_verified", "native_client_snapshot", "native_client_commands", "native_command_live_verified")))
    sources = value.get("client_event_sources")
    require(isinstance(sources, dict) and all(type(sources.get(key)) is bool for key in ("open_hud_screen", "station_recipe_selection")))
    return {"compiled_framework_revision": value["compiled_framework_revision"], "compiled_framework_abi": value["compiled_framework_abi"],
            "live_verified": False, "native_observation": value["ui_observation"], "recipe_layout_available": value["recipe_layout"],
            "client_event_sources": {key: sources[key] for key in ("open_hud_screen", "station_recipe_selection")}}



def command_inventory(value):
    require(value.get("commands_submitted") is False and value.get("native_identity_reconstructed") is False
            and value.get("automatic_retry") is False and type(value.get("max_records")) is int and value["max_records"] == 128)
    commands = value.get("commands")
    require(isinstance(commands, list) and len(commands) <= 128 and all(isinstance(row, dict) for row in commands))
    operations = [row.get("operation") for row in commands]
    require(all(type(operation) is str and re.fullmatch(r"[a-z0-9_-]{1,64}", operation) for operation in operations))
    require(operations == sorted(set(operations)))
    counts = {}
    stale_polls = unpolled = 0
    for row in commands:
        state = row.get("state")
        require(state in ("reserved", "queued", "finished_or_unknown", "refused_or_unknown", "prior_boot_unavailable")
                and row.get("automatic_retry") is False)
        if state == "prior_boot_unavailable":
            require(row.get("native_identity_reconstructed") is False)
        else:
            require(type(row.get("native_submitted")) is bool and row.get("rendered_verified") is False and row.get("arrival_verified") is False)
        observed = row.get("status_observation")
        require(isinstance(observed, dict) and observed.get("source") == "client_frame_status_poll"
                and observed.get("current_context_verified") is False)
        age = observed.get("age_ms")
        require((age is None or integer(age)) and observed.get("stale") is (age is None or age > 1000))
        if state == "prior_boot_unavailable":
            require(age is None and observed.get("polling_finished") is None)
        else:
            require(type(observed.get("polling_finished")) is bool)
        stale_polls += int(observed["stale"])
        unpolled += int(age is None)
        counts[state] = counts.get(state, 0) + 1
    # Caller operation identities, payloads and errors never enter the saved probe report.
    return {"records":len(commands), "states":counts, "stale_poll_observations":stale_polls, "unpolled_operations":unpolled,
            "commands_submitted":False, "native_identity_reconstructed":False}


def native_snapshot(value):
    require(value.get("source") == "published_native_client_snapshot" and
            value.get("state") in ("never", "off", "stale", "fresh") and integer(value.get("session")) and
            (value.get("age_ms") is None or integer(value["age_ms"])) and
            value.get("entity_world") == "client" and value.get("commands_submitted") is False and
            value.get("rendered_verified") is False)
    reads = value.get("reads")
    require(isinstance(reads, dict) and all(type(reads.get(key)) is bool for key in ("screens", "menu_tab", "held", "player", "recipe")))
    for key, part, kind in (("hud_screen", "screens", str), ("return_screen", "screens", str),
                            ("menu_tab", "menu_tab", str), ("host", "screens", int),
                            ("player", "player", int), ("highlighted_recipe_id", "recipe", int),
                            ("held_actions", "held", list)):
        require(key in value)
        item = value[key]
        require(item is None or reads[part] and type(item) is kind)
        if kind is int and item is not None:
            require(item > 0 and (key != "highlighted_recipe_id" or item <= 0xFFFFFFFF))
        if kind is list and item is not None:
            require(all(type(action) is str for action in item))
    # Never save entity IDs, session identity, selection or held controls.
    return {"state":value["state"], "age_ms":value["age_ms"], "reads":reads,
            "rendered_verified":False, "commands_submitted":False}


def state(value):
    age = value.get("age_ms")
    require((age is None or integer(age)) and type(value.get("stale")) is bool and integer(value.get("sequence")))
    observation = value.get("observation")
    require(observation is None or isinstance(observation, dict))
    if observation is not None:
        require(isinstance(observation.get("screen"), str) and "host_template" in observation and
                (observation["host_template"] is None or isinstance(observation["host_template"], str) and GUID.fullmatch(observation["host_template"])))
        prompt = observation.get("prompt")
        if prompt is not None:
            require(isinstance(prompt, dict) and integer(prompt.get("verb")) and prompt["verb"] <= 0xFFFFFFFF and
                    isinstance(prompt.get("language"), str) and type(prompt.get("verb_text_truncated")) is bool and
                    prompt.get("verb_text_source") == "client_archive_localization_with_English_fallback")
            text = prompt.get("verb_text")
            require((prompt.get("verb_text_state") == "unavailable" and text is None and prompt["verb_text_truncated"] is False) or
                    (prompt.get("verb_text_state") == "observed" and isinstance(text, str) and len(text.encode("utf-8")) <= 256))
    # Neither prompts, held inputs nor host/session identities enter the report.
    return {"stale": value["stale"], "age_ms": age, "observation_available": observation is not None,
            "host_template_available": observation is not None and observation["host_template"] is not None}


def events(value):
    entries = value.get("events")
    require(isinstance(entries, list) and len(entries) <= 2 and
            all(integer(value.get(key)) for key in ("next_after", "latest_sequence", "dropped_total")) and
            type(value.get("cursor_gap")) is bool)
    sequences = [entry.get("sequence") for entry in entries]
    require(all(integer(seq) and seq > 0 for seq in sequences) and sequences == sorted(set(sequences)))
    require(value["next_after"] == (sequences[-1] if sequences else 0) and value["next_after"] <= value["latest_sequence"])
    return {"returned": len(entries), "cursor_gap": value["cursor_gap"], "dropped_count": value["dropped_total"]}


def selection_events(value):
    entries = value.get("events")
    require(isinstance(entries, list) and len(entries) <= 2 and
            all(integer(value.get(key)) for key in ("next_after", "latest_sequence", "dropped_history")) and
            type(value.get("cursor_gap")) is bool and
            all(key in value and value[key] is None for key in ("current_selected_tab", "current_selected_recipe")) and
            all(isinstance(value.get(key), str) and value[key] for key in ("state_status", "evidence_scope", "identity_scope")))
    sequences = [entry.get("sequence") for entry in entries]
    require(all(integer(seq) and seq > 0 for seq in sequences) and sequences == sorted(set(sequences)))
    require(value["next_after"] == (sequences[-1] if sequences else 0) and value["next_after"] <= value["latest_sequence"])
    for entry in entries:
        require(entry.get("scope") == "client_world" and isinstance(entry.get("event"), dict))
        event = entry["event"]
        if event.get("kind") == "open_hud_screen_event":
            require(isinstance(event.get("requested_screen"), str) and len(event["requested_screen"].encode()) <= 256 and
                    integer(event.get("host")) and event["host"] <= 4294967295)
        elif event.get("kind") == "station_recipe_selection_event":
            choices = event.get("category_choices")
            require(all(integer(event.get(key)) and event[key] <= 4294967295 for key in ("operator", "station", "recipe_id")) and
                    isinstance(choices, list) and len(choices) <= 4 and type(event.get("choices_truncated")) is bool and
                    all(integer(choice.get("category_id")) and choice["category_id"] <= 4294967295 and
                        integer(choice.get("selected_index")) and choice["selected_index"] <= 255 for choice in choices))
        else:
            require(False)
    return {"returned": len(entries), "cursor_gap": value["cursor_gap"], "dropped_count": value["dropped_history"],
            "current_selection_verified": False}


def item(value):
    require(value.get("item_id") == 213939880 and value.get("debug_name") == "Prop_Storage_24_T1_Box" and
            value.get("source") == "vanilla_archive_and_client_localization" and value.get("effective_definition_verified") is False)
    localized = value.get("localized_name")
    require(localized is None or isinstance(localized, str))
    return {"localized_name_available": bool(localized)}


def recipes(value):
    entries = value.get("recipes")
    require(value.get("mode") == "report_only" and value.get("source") == "vanilla_archive_recipe_registry" and
            value.get("workshop_reference_layout_checked") is True and
            value.get("effective_definition_verified") is False and isinstance(value.get("unverified"), list) and bool(value["unverified"]) and
            isinstance(entries, list) and len(entries) <= 2 and integer(value.get("total_matches")) and
            value["total_matches"] >= len(entries) and value.get("truncated") is (value["total_matches"] > len(entries)))
    require(isinstance(value.get("selected_output_guid"), str) and GUID.fullmatch(value["selected_output_guid"]))
    for entry in entries:
        require(integer(entry.get("recipe_id")) and entry["recipe_id"] <= 0xFFFFFFFF)
        require(integer(entry.get("workshop_id")) and entry["workshop_id"] <= 0xFFFFFFFF)
        require(all(isinstance(entry.get(key), str) and GUID.fullmatch(entry[key]) for key in ("recipe_guid", "workshop_guid")))
        require(isinstance(entry.get("debug_name"), str) and len(entry["debug_name"].encode()) <= 128)
        require(all(key in entry and entry[key] is None for key in
                    ("ingredient_category_semantics", "catalyst_semantics", "required_props", "unlock", "feasible")))
        for key, cap in (("inputs", 32), ("outputs", 8)):
            stacks = entry.get(key)
            require(isinstance(stacks, list) and len(stacks) <= cap and all(
                isinstance(stack.get("item_guid"), str) and GUID.fullmatch(stack["item_guid"]) and
                integer(stack.get("item_id")) and stack["item_id"] <= 0xFFFFFFFF and
                integer(stack.get("count")) for stack in stacks))
        require(any(stack["item_guid"] == value["selected_output_guid"] for stack in entry["outputs"]))
        requirements = entry.get("requirement_observations")
        require(isinstance(requirements, dict) and requirements.get("state") == "observed" and
                requirements.get("source") == "vanilla_archive_recipe_requirements" and
                requirements.get("full_requirements_verified") is False and requirements.get("eligibility_verified") is False and
                type(requirements.get("requires_shelter")) is bool and isinstance(requirements.get("required_prop_ids"), list) and
                len(requirements["required_prop_ids"]) <= 32 and all(integer(item) and item <= 0xFFFFFFFF for item in requirements["required_prop_ids"]))
        observations = entry.get("input_observations")
        require(isinstance(observations, list) and len(observations) == len(entry["inputs"]))
        for observation in observations:
            membership = observation.get("category_membership")
            require(isinstance(membership, dict))
            if membership.get("state") == "observed":
                members = membership.get("members")
                total = membership.get("total_members")
                require(membership.get("source") == "vanilla_archive_recipe_input_categories" and
                        membership.get("scaling_semantics_verified") is False and
                        integer(total) and total <= 1024 and isinstance(members, list) and
                        len(members) == min(total, 32) and membership.get("truncated") is (total > 32))
                for member in members:
                    require(integer(member.get("item_id")) and member["item_id"] <= 0xFFFFFFFF and
                            integer(member.get("scaling_bits")) and member["scaling_bits"] <= 0xFFFFFFFF and
                            type(member.get("scaling_finite")) is bool and
                            (type(member.get("scaling")) in (int, float) if member["scaling_finite"] else member.get("scaling") is None))
            else:
                require(membership.get("state") in ("not_found", "no_category_id") and membership.get("members") is None)
    return {"returned": len(entries), "total_matches": value["total_matches"], "truncated": value["truncated"],
            "raw_stack_semantics": "unverified; zero GUID/count remains unresolved"}


def workshops(value):
    rows=value.get("matches")
    require(value.get("mode")=="report_only" and value.get("source")=="vanilla_archive_workshop_registry"
            and value.get("identity_scope")=="recipe_workshop_definition" and value.get("effective_definition_verified") is False
            and value.get("current_availability_verified") is False and value.get("current_station_available") is None)
    require(type(value.get("workshop_id")) is int and value["workshop_id"]==0
            and integer(value.get("registry_entries")) and value["registry_entries"]<=4096
            and integer(value.get("total_matches")) and value["total_matches"]<=value["registry_entries"]
            and isinstance(rows,list) and len(rows)==min(2,value["total_matches"])
            and value.get("truncated") is (value["total_matches"]>len(rows)))
    require(value.get("state")==("not_found" if value["total_matches"]==0 else "single_match" if value["total_matches"]==1 else "multiple_matches"))
    for row in rows:
        require(type(row.get("workshop_id")) is int and row["workshop_id"]==0
                and all(integer(row.get(k)) and row[k]<=0xFFFFFFFF for k in ("name_tag","station_item_id"))
                and all(isinstance(row.get(k),str) and GUID.fullmatch(row[k]) for k in ("workshop_guid","station_item_guid"))
                and type(row.get("npc_workshop")) is bool)
    return {"returned":len(rows),"total_matches":value["total_matches"],"current_availability_verified":False}


class Probe:
    def __init__(self, client, pause=time.sleep):
        self.client, self.pause = client, pause
        self.checks = []

    def tool(self, label, name, args, summarize=None, refusal=None):
        if name not in READS:
            raise ValueError("only fixed companion reads are allowed")
        check = {"name": label, "outcome": "fail"}
        self.checks.append(check)
        try:
            reply = self.client.call("tools/call", {"name": name, "arguments": args})
            require(isinstance(reply, dict) and type(reply.get("isError")) is bool)
            content = reply.get("content")
            require(isinstance(content, list) and len(content) == 1 and content[0].get("type") == "text" and isinstance(content[0].get("text"), str))
            if refusal is not None:
                require(reply["isError"] and refusal in content[0]["text"].casefold())
            else:
                require(not reply["isError"])
                value = json.loads(content[0]["text"])
                require(isinstance(value, dict))
                check["observation"] = summarize(value)
            check["outcome"] = "pass"
        except OSError:
            check["outcome"] = "blocked"
        except (ValueError, TypeError, KeyError, AttributeError):
            pass
        finally:
            self.pause(0.25)

    def run(self):
        try:
            init = self.client.call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                                    "clientInfo": {"name": "native-client-probe", "version": "1"}})
            require(init["serverInfo"] == {"name": "enshrouded-mcp", "version": "0.20.0"} and
                    init.get("protocolVersion") == "2025-06-18")
            listing = self.client.call("tools/list")
            require((READS | {"client_map_markers"}).issubset({entry["name"] for entry in listing["tools"]}))
            self.checks.append({"name": "discovery_0.20.0", "outcome": "pass"})
        except OSError:
            return self.report("blocked")
        except (ValueError, TypeError, KeyError):
            return self.report("failed")
        self.tool("capabilities", "client_capabilities", {}, capabilities)
        self.tool("command_inventory", "client_commands", {}, command_inventory)
        self.tool("command_inventory_unknown_argument", "client_commands", {"probe_unknown": True}, refusal="unknown argument")
        self.tool("native_snapshot_shape", "client_snapshot", {}, native_snapshot)
        self.tool("state_shape", "client_state", {}, state)
        self.tool("event_cursor", "client_events", {"after": 0, "limit": 2}, events)
        self.tool("selection_event_evidence", "client_selection_events", {"after": 0, "limit": 2}, selection_events)
        self.tool("storage_box_localization", "client_item", {"item": "Prop_Storage_24_T1_Box"}, item)
        self.tool("workbench_raw_recipes", "client_recipes", {"item": "Craftingstation_Workbench", "limit": 2}, recipes)
        self.tool("workshop_definition_shape", "client_workshops", {"workshop_id":0,"limit":2}, workshops)
        self.tool("unknown_argument_refused", "client_state", {"probe_unknown": True}, refusal="unknown argument")
        self.tool("invalid_recipe_limit_refused", "client_recipes", {"item": "Craftingstation_Workbench", "limit": 9}, refusal="argument 'limit' must be at most 8")
        outcomes = {check["outcome"] for check in self.checks}
        return self.report("failed" if "fail" in outcomes else "blocked" if "blocked" in outcomes else "incomplete")

    def report(self, verdict):
        return {"checked_utc": datetime.now(timezone.utc).isoformat(), "verdict": verdict, "checks": self.checks,
                "incomplete": ["No independent in-game UI fixture is established by these fixed probes.",
                               "Stale observations do not establish current UI state; inspect the age/stale observation.",
                               "Missing localization remains unavailable, not verified.",
                               "Observed client events do not establish a current tab or passively selected recipe.",
                               "Raw recipe stacks do not decode category/catalyst/required-prop/unlock semantics or establish feasibility."]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8754)
    args = parser.parse_args()
    if not 1 <= args.port <= 65535:
        parser.error("port must be 1..65535")
    report = Probe(Client(args.port, os.environ.get("ENSHROUDED_MCP_CLIENT_TOKEN", ""))).run()
    directory = Path(__file__).resolve().parents[1] / "gamedata" / "client-probes"
    directory.mkdir(parents=True, exist_ok=True)
    output = directory / (datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ") + ".json")
    output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"{report['verdict']}: {len(report['checks'])} checks; report: {output}")
    return 1 if report["verdict"] == "failed" else 2


if __name__ == "__main__":
    raise SystemExit(main())
