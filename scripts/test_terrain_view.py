"""Independent asymmetric geometry and read-only capture contracts."""
import copy
from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from terrain_view import capture, scene, html, main


def survey():
    cells = [dict(min_cells=[x, y, z], material={"kind": "air", "id": None}, fill=0)
             for z in range(60, 64) for y in range(40, 43) for x in range(20, 22)]
    cells[0].update(material={"kind": "terrain", "id": 2}, fill=128)
    cells[-1].update(material={"kind": "building", "id": 157}, fill=7)
    return {"status": "read_only_observation", "grid": {"cell_size_m": .5},
            "region": {"min_cells": [20, 40, 60], "size_cells": [2, 3, 4],
                       "max_exclusive_cells": [22, 43, 64], "min_m": [10, 20, 30],
                       "max_exclusive_m": [11, 21.5, 32]},
            "voxels": {"cell_count": 24, "cells": cells,
                       "material_counts": {"air": 22, "terrain": 1, "building": 1, "unknown": 0}}}


class TerrainViewTests(unittest.TestCase):
    def test_asymmetric_axes_raw_fill_and_precise_pin_survive(self):
        value = scene(survey(), [{"label": "Corner", "position": [10.25, 21.125, 31.75]}])
        self.assertEqual(value["min_m"], [10, 20, 30])
        self.assertEqual(value["max_m"], [11, 21.5, 32])
        self.assertEqual(value["cells"][0], [20, 40, 60, "terrain", 2, 128])
        self.assertEqual(value["cells"][-1], [21, 42, 63, "building", 157, 7])
        self.assertEqual(value["pins"][0]["position"], [10.25, 21.125, 31.75])
        self.assertFalse(value["rendered_surface_verified"])
        self.assertFalse(value["atomic_snapshot"])

    def test_incomplete_duplicate_outside_and_false_counts_refuse(self):
        for change in [lambda s: s["voxels"]["cells"].pop(),
                       lambda s: s["voxels"]["cells"].__setitem__(1, copy.deepcopy(s["voxels"]["cells"][0])),
                       lambda s: s["voxels"]["cells"][0].update(min_cells=[22, 40, 60]),
                       lambda s: s["voxels"].update(cell_count=23),
                       lambda s: s["voxels"]["material_counts"].update(terrain=2),
                       lambda s: s["voxels"]["material_counts"].update(terrain=True),
                       lambda s: s["region"].update(max_exclusive_m=[11, 22, 32])]:
            with self.subTest(change=change):
                value = survey(); change(value)
                with self.assertRaises(ValueError): scene(value)

    def test_invalid_values_and_oversized_regions_refuse(self):
        for change in [lambda s: s["voxels"]["cells"][0].update(fill=True),
                       lambda s: s["voxels"]["cells"][0].update(fill=256),
                       lambda s: s["voxels"]["cells"][0].update(material={"kind": "lava", "id": 3}),
                       lambda s: s["region"].update(size_cells=[17, 3, 4]),
                       lambda s: s["region"].update(min_cells=[True, 40, 60])]:
            with self.subTest(change=change):
                value = survey(); change(value)
                with self.assertRaises(ValueError): scene(value)
        for pins in [[{"label": "bad", "position": [10, float("nan"), 30]}],
                     [{"label": "bad\n", "position": [10, 20, 30]}],
                     [{"label": "same", "position": [10, 20, 30]}]*2]:
            with self.assertRaises(ValueError): scene(survey(), pins)

    def test_html_keeps_untrusted_labels_as_data_and_has_no_remote_dependencies(self):
        value = scene(survey(), [{"label": "</script><img src=x>", "position": [10, 20, 30]}])
        page = html(value)
        self.assertNotIn("</script><img", page)
        self.assertNotIn("fetch(", page)
        self.assertNotIn("https://", page)
        self.assertIn("\\u003c/script", page)
        self.assertIn("terrain-data", page)

    def test_capture_calls_only_bounded_reads_and_discards_player_identity(self):
        calls = []
        def read(name, args):
            calls.append((name, args))
            if name == "building_survey": return survey()
            if name == "players": return [{"key": "private", "name": "private", "position": [10, 21, 31]}]
            if name == "props_in": return {"total": 0, "props": [], "truncated": False}
            self.fail(name)
        result = capture(read, [10, 20, 30, 1, 1.5, 2])
        self.assertEqual([x[0] for x in calls], ["building_survey", "players", "props_in"])
        self.assertEqual(calls[0][1], dict(x=10, y=20, z=30, width=1, height=1.5, depth=2))
        self.assertNotIn("private", json.dumps(result))
        self.assertFalse(result["atomic_snapshot"])
        self.assertEqual(result["native_session"], "unavailable")
        self.assertEqual(result["pins"][0]["kind"], "player")

    def test_prop_origin_coverage_and_missing_player_pose_remain_explicit(self):
        def read(name, args):
            return {"building_survey": survey(), "players": [{"position": None}],
                    "props_in": {"total": 2, "props": [{"position": [10, 21, 31]}], "truncated": False}}[name]
        value = capture(read, [10, 20, 30, 1, 1.5, 2])
        self.assertEqual(value["missing_player_positions"], 1)
        self.assertEqual(value["prop_origins"], {"total": 2, "shown": 1, "truncated": True})
        self.assertEqual(value["pins"][0]["kind"], "prop")

    def test_failed_or_mismatched_capture_never_becomes_empty_geometry(self):
        calls = []
        def read(name, args):
            calls.append(name)
            raise ValueError("chunk not loaded")
        with self.assertRaisesRegex(ValueError, "chunk not loaded"):
            capture(read, [10, 20, 30, 1, 1.5, 2])
        self.assertEqual(calls, ["building_survey"])
        with self.assertRaises(ValueError): capture(read, [10.25, 20, 30, 1, 1.5, 2])
        def wrong(name, args): return survey()
        with self.assertRaises(ValueError): capture(wrong, [10, 20, 30, 1, 1, 2])

    def test_offline_cli_is_exclusive_and_labels_cannot_replace_template(self):
        with tempfile.TemporaryDirectory() as folder:
            source, output = Path(folder)/"capture.json", Path(folder)/"view.html"
            source.write_text(json.dumps({"survey": survey(), "pins": [
                {"label": "__TERRAIN_SCRIPT__", "position": [10, 20, 30]}]}))
            args = ["--input", str(source), "--output", str(output)]
            with redirect_stdout(io.StringIO()): main(args)
            before = output.read_bytes()
            with self.assertRaisesRegex(ValueError, "output already exists"): main(args)
            self.assertEqual(before, output.read_bytes())
            page = before.decode()
            payload = page.split('<script id="terrain-data" type="application/json">')[1].split("</script>")[0]
            self.assertEqual(json.loads(payload)["pins"][0]["label"], "__TERRAIN_SCRIPT__")

    @unittest.skipUnless(shutil.which("node"), "Node is optional; required for renderer geometry checks")
    def test_renderer_coordinates_sections_front_faces_and_top_selection(self):
        script = r'''
const assert = require('node:assert/strict');
require(process.argv[1]);
const g = globalThis.TerrainGeometry;
assert.deepEqual(g.project([3,4,5], 'top'), {x:3,y:-5,depth:4});
assert.deepEqual(g.project([3,4,5], 'xy'), {x:3,y:-4,depth:5});
assert.deepEqual(g.project([3,4,5], 'zy'), {x:5,y:-4,depth:3});
const p=g.project([3,4,5], 'orbit', Math.PI/2, 0);
assert.ok(Math.abs(p.x-5)<1e-10 && p.y===-4 && Math.abs(p.depth+3)<1e-10);
const q=g.project([3,4,5], 'orbit', 0, Math.PI/2);
assert.ok(q.x===3 && Math.abs(q.y-5)<1e-10 && Math.abs(q.depth-4)<1e-10);
const cells=[[20,40,60,'terrain',2,128],[20,41,60,'air',null,0],[21,42,63,'building',157,7]];
assert.deepEqual(g.section(cells,'xy',60),cells.slice(0,2));
assert.deepEqual(g.section(cells,'zy',21),[cells[2]]);
assert.deepEqual(g.topCells(cells),[cells[0],cells[2]]);
const faces=g.exposedFaces([[20,40,60,'building',157,7]],-.7,.6);
assert.equal(faces.length,3);
assert.deepEqual(new Set(faces.map(f=>f.face)),new Set([1,3,5]));
assert.ok(faces.every(f=>f.points.every(p=>p[0]>=10 && p[0]<=10.5 && p[1]>=20 && p[1]<=20.5 && p[2]>=30 && p[2]<=30.5)));
const pair=g.exposedFaces([[20,40,60,'building',157,7],[21,40,60,'terrain',2,255]],-.7,.6);
assert.equal(pair.length,5); // shared internal face is absent
assert.equal(g.exposedFaces([[20,40,60,'air',null,255]],-.7,.6).length,0);
'''
        subprocess.run([shutil.which("node"), "-e", script,
                        str(Path(__file__).with_name("terrain_view.js"))], check=True, capture_output=True)

    @unittest.skipUnless(shutil.which("node"), "Node is optional; required for renderer event checks")
    def test_standard_select_change_redraws_sections_and_range_input_updates_slice(self):
        # Standard EventTarget; canvas is a drawing sink, not a second renderer.
        script = r'''
const assert = require('node:assert/strict');
class Element extends EventTarget {
  constructor(tagName='DIV'){super();this.tagName=tagName;this.value='';this.hidden=false;this.dataset={};}
  append(){}
}
const ids=['terrain-data','terrain','view','yaw','pitch','zoom','slice','selection','measure','pins',
           'provenance','slice-control','yaw-control','pitch-control','slice-value','export'];
const nodes=Object.fromEntries(ids.map(id=>[id,new Element()]));
nodes['terrain-data'].textContent=process.argv[2];
nodes.view.tagName='SELECT';nodes.view.value='orbit';nodes.yaw.value='320';
nodes.pitch.value='32';nodes.zoom.value='100';
nodes.terrain.clientWidth=800;nodes.terrain.clientHeight=520;
nodes.terrain.getContext=()=>new Proxy({}, {get:(target,key)=>target[key]??(()=>{})});
globalThis.document={getElementById:id=>nodes[id],createElement:tag=>new Element(tag.toUpperCase())};
globalThis.window={devicePixelRatio:1,addEventListener:()=>{}};
require(process.argv[1]);
assert.equal(nodes.terrain.dataset.view,'orbit');
assert.equal(nodes['slice-control'].hidden,true);
nodes.view.value='xy';nodes.view.dispatchEvent(new Event('change'));
assert.equal(nodes.terrain.dataset.view,'xy');
assert.equal(nodes.terrain.dataset.renderedFaces,'6');
assert.equal(nodes['slice-control'].hidden,false);
assert.equal(nodes['yaw-control'].hidden,true);
nodes.view.value='zy';nodes.view.dispatchEvent(new Event('change'));
assert.equal(nodes.terrain.dataset.renderedFaces,'12');
nodes.slice.value='21';nodes.slice.dispatchEvent(new Event('input'));
assert.equal(nodes['slice-value'].textContent,'X [10.5, 11) m');
nodes.view.value='top';nodes.view.dispatchEvent(new Event('change'));
assert.equal(nodes.terrain.dataset.view,'top');
assert.equal(nodes.terrain.dataset.renderedFaces,'8');
assert.equal(nodes['slice-control'].hidden,true);
'''
        subprocess.run([shutil.which("node"), "-e", script,
                        str(Path(__file__).with_name("terrain_view.js")),
                        json.dumps(scene(survey()))], check=True, capture_output=True)


if __name__ == "__main__": unittest.main()
