"""Render bounded native voxel observations as a standalone diagnostic viewer."""
import argparse
from datetime import datetime, timezone
import json
import math
from pathlib import Path

from investigate import Client

KINDS = ("air", "terrain", "building", "unknown")
MAX_BYTES = 2 * 1024 * 1024


def require(ok, message):
    if not ok:
        raise ValueError(message)


def vector(value, *, integer=False):
    return (isinstance(value, list) and len(value) == 3 and
            all(type(x) is int if integer else type(x) in (int, float) and -20480 <= x <= 20480 for x in value))


def scene(survey, pins=()):
    """Reject partial/contradictory captures; retain raw cells without mesh inference."""
    require(isinstance(survey, dict) and survey.get("status") == "read_only_observation", "invalid survey")
    require(isinstance(survey.get("grid"), dict) and survey["grid"].get("cell_size_m") == .5, "unsupported cell size")
    region, voxels = survey.get("region", {}), survey.get("voxels", {})
    require(isinstance(region, dict) and isinstance(voxels, dict), "invalid region/voxels")
    low, size, high = (region.get(k) for k in ("min_cells", "size_cells", "max_exclusive_cells"))
    require(all(vector(v, integer=True) for v in (low, size, high)), "invalid cell bounds")
    require(all(0 <= low[i] < high[i] <= (2048 if i == 1 else 20480) and
                1 <= size[i] <= 16 and high[i] == low[i] + size[i] for i in range(3)), "outside bounded region")
    require(vector(region.get("min_m")) and vector(region.get("max_exclusive_m")) and
            region.get("min_m") == [x / 2 for x in low] and
            region.get("max_exclusive_m") == [x / 2 for x in high], "metre/cell bounds disagree")
    cells = voxels.get("cells")
    require(isinstance(cells, list) and len(cells) == math.prod(size) <= 4096 and
            type(voxels.get("cell_count")) is int and voxels["cell_count"] == len(cells), "incomplete cells")
    seen, encoded, counts = set(), [], dict.fromkeys(KINDS, 0)
    for cell in cells:
        require(isinstance(cell, dict), "invalid cell")
        pos, material, fill = cell.get("min_cells"), cell.get("material", {}), cell.get("fill")
        require(vector(pos, integer=True) and all(low[i] <= pos[i] < high[i] for i in range(3)), "outside cell")
        key = tuple(pos)
        require(key not in seen, "duplicate cell")
        seen.add(key)
        require(isinstance(material, dict) and material.get("kind") in KINDS, "invalid material")
        kind, ident = material["kind"], material.get("id")
        require((ident is None and kind in ("air", "unknown")) or
                (kind != "air" and type(ident) is int and 0 <= ident <= 255), "invalid material ID")
        require(type(fill) is int and 0 <= fill <= 255, "invalid fill")
        counts[kind] += 1
        encoded.append([*pos, kind, ident, fill])
    reported = voxels.get("material_counts")
    require(isinstance(reported, dict) and all(type(v) is int for v in reported.values()) and reported == counts,
            "material counts disagree")
    require(isinstance(pins, (list, tuple)) and len(pins) <= 64, "at most 64 pins")
    normalized, labels = [], set()
    for pin in pins:
        require(isinstance(pin, dict), "invalid pin")
        label, position, kind = pin.get("label"), pin.get("position"), pin.get("kind", "reference")
        require(isinstance(label, str) and 1 <= len(label) <= 64 and all(c.isprintable() for c in label), "invalid pin label")
        require(label not in labels and vector(position), "duplicate label or invalid pin position")
        require(all(0 <= position[i] <= (1024 if i == 1 else 10240) for i in range(3)), "outside pin")
        require(kind in ("reference", "player", "prop"), "invalid pin kind")
        labels.add(label)
        normalized.append({"label": label, "position": position, "kind": kind,
                           "inside_capture": all(low[i] / 2 <= position[i] < high[i] / 2 for i in range(3))})
    return {"version": 1, "min_m": region["min_m"], "max_m": region["max_exclusive_m"],
            "cells": encoded, "pins": normalized, "material_counts": counts,
            "native_session": "unavailable", "atomic_snapshot": False,
            "rendered_surface_verified": False, "capture": "offline input; time/session unverified"}


def box_args(box):
    require(isinstance(box, (list, tuple)) and len(box) == 6 and
            all(type(x) in (int, float) and 0 <= x <= 10240 and float(x * 2).is_integer()
                for x in box), "box requires six half-metre grid values")
    low, sizes = box[:3], box[3:]
    require(all(0 <= low[i] and .5 <= sizes[i] <= 8 and
                low[i] + sizes[i] <= (1024 if i == 1 else 10240) for i in range(3)), "invalid box")
    return dict(zip(("x", "y", "z", "width", "height", "depth"), box))


def capture(read, box, pins=()):
    """Exactly three read tools; failures propagate and never imply empty geometry."""
    args = box_args(box)
    times = []
    def observe(name, arguments):
        start = datetime.now(timezone.utc).isoformat()
        value = read(name, arguments)
        times.append({"tool": name, "started_utc": start, "completed_utc": datetime.now(timezone.utc).isoformat()})
        return value
    survey = observe("building_survey", args)
    result = scene(survey, pins)
    require(result["min_m"] == list(box[:3]) and
            result["max_m"] == [box[i] + box[i + 3] for i in range(3)], "requested/captured box differs")
    players = observe("players", {})
    require(isinstance(players, list) and len(players) <= 16, "invalid player roster")
    observations = list(pins)
    missing_players = 0
    for index, player in enumerate(players):
        require(isinstance(player, dict), "invalid player")
        position = player.get("position")
        if position is None:
            missing_players += 1
        else:
            observations.append({"label": f"Player {index + 1}", "position": position, "kind": "player"})
    bounds = dict(zip(("x1", "y1", "z1", "x2", "y2", "z2"), result["min_m"] + result["max_m"]))
    props = observe("props_in", bounds)
    require(isinstance(props, dict) and isinstance(props.get("props"), list) and
            type(props.get("total")) is int and 0 <= len(props["props"]) <= props["total"] and
            type(props.get("truncated")) is bool, "invalid props")
    for index, prop in enumerate(props["props"][:32]):
        require(isinstance(prop, dict), "invalid prop")
        observations.append({"label": f"Prop {index + 1}", "position": prop.get("position"), "kind": "prop"})
    result = scene(survey, observations)
    result["capture"] = times
    result["missing_player_positions"] = missing_players
    result["prop_origins"] = {"total": props["total"], "shown": min(32, len(props["props"])),
                              "truncated": props["truncated"] or props["total"] > min(32, len(props["props"]))}
    return result


def html(value):
    folder = Path(__file__).resolve().parent
    payload = json.dumps(value, allow_nan=False).replace("<", "\\u003c").replace("\u2028", "\\u2028").replace("\u2029", "\\u2029")
    return ((folder / "terrain_view.html").read_text(encoding="utf-8")
            .replace("__TERRAIN_SCRIPT__", (folder / "terrain_view.js").read_text(encoding="utf-8"))
            .replace("__TERRAIN_DATA__", payload))


def plan_cells(preview):
    require(isinstance(preview, dict) and preview.get("status") == "geometry_preview_only" and
            preview.get("executable") is False, "plan must be a geometry-only preview")
    require(isinstance(preview.get("grid"), dict) and preview["grid"].get("cell_size_m") == .5,
            "unsupported plan grid")
    cells, geometry = preview.get("cell_minima"), preview.get("geometry", {})
    require(isinstance(cells, list) and 1 <= len(cells) <= 256 and isinstance(geometry, dict) and
            type(geometry.get("solid_cell_count")) is int and geometry["solid_cell_count"] == len(cells),
            "plan requires 1 to 256 counted cells")
    seen = set()
    for cell in cells:
        require(vector(cell, integer=True) and
                all(0 <= cell[i] < (2048 if i == 1 else 20480) for i in range(3)), "outside plan cell")
        key = tuple(cell)
        require(key not in seen, "duplicate plan cell")
        seen.add(key)
    return cells


def with_plan(value, preview):
    """Compare proposed cells with samples; neither operand grants execution authority."""
    lookup = {tuple(c[:3]): c for c in value["cells"]}
    counts = dict.fromkeys(["sampled_" + k for k in KINDS] + ["unobserved"], 0)
    targets = []
    for position in plan_cells(preview):
        cell = lookup.get(tuple(position))
        state = "sampled_" + cell[3] if cell else "unobserved"
        counts[state] += 1
        targets.append({"min_cells": position, "state": state,
                        "observed": {"kind": cell[3], "id": cell[4], "fill": cell[5]} if cell else None})
    return {**value, "plan": {"targets": targets, "counts": counts, "executable": False,
                             "clearance_verified": False, "source": "provided geometry preview; not an authenticated native receipt"}}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--input", type=Path, help="Offline JSON containing survey and optional pins")
    source.add_argument("--port", type=int, help="Existing loopback server endpoint; no credentials read")
    parser.add_argument("--box", type=float, nargs=6, metavar=("X", "Y", "Z", "W", "H", "D"))
    parser.add_argument("--pins", type=Path, help="Optional JSON list of at most 64 labelled metre positions")
    parser.add_argument("--plan", type=Path, help="Optional geometry-only building_plan result; at most 256 cells")
    parser.add_argument("--output", required=True, type=Path, help="New standalone HTML path; never overwrites")
    args = parser.parse_args(argv)
    require(not args.output.exists(), "output already exists")
    def load(path):
        require(path.is_file() and path.stat().st_size <= MAX_BYTES, "input missing or too large")
        return json.loads(path.read_text(encoding="utf-8"))
    pins = load(args.pins) if args.pins else []
    plan = load(args.plan) if args.plan else None
    if args.plan:
        plan_cells(plan)  # Reject malformed overlays before any live capture reads.
    if args.input:
        require(args.box is None, "--box requires --port")
        data = load(args.input)
        require(not (args.plan and "plan" in data), "plan supplied twice")
        plan = data.get("plan", plan)
        value = scene(data["survey"], data.get("pins", []) + pins)
    else:
        require(1 <= args.port <= 65535 and args.box is not None, "--port requires --box")
        client = Client(args.port, "")
        def read(name, arguments):
            reply = client.call("tools/call", {"name": name, "arguments": arguments})
            require(not reply.get("isError"), f"{name} refused; no view generated")
            return json.loads(reply["content"][0]["text"])
        value = capture(read, args.box, pins)
    if plan is not None:
        value = with_plan(value, plan)
    with args.output.open("x", encoding="utf-8") as stream:
        stream.write(html(value))
    print(json.dumps({"output": str(args.output), "cells": len(value["cells"]), "pins": len(value["pins"]),
                      "proposed_cells": len(value.get("plan", {}).get("targets", [])),
                      "rendered_surface_verified": False, "atomic_snapshot": False}))


if __name__ == "__main__":
    main()
