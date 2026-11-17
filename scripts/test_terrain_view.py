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
from unittest.mock import patch

from terrain_view import capture, scene, html, main, with_plan


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
    def test_plan_classifies_sampled_materials_and_unobserved_without_changing_capture(self):
        raw = survey()
        row = next(c for c in raw["voxels"]["cells"] if c["min_cells"] == [21,40,61])
        row["material"] = {"kind":"unknown","id":254}
        raw["voxels"]["material_counts"].update(air=21,unknown=1)
        base = scene(raw); original = copy.deepcopy(base)
        cells = [[20,40,60],[21,42,63],[20,41,60],[21,40,61],[22,40,60]]
        value = with_plan(base, preview(cells))
        self.assertEqual(base, original)
        self.assertEqual(value["cells"], original["cells"])
        self.assertEqual([r["state"] for r in value["plan"]["targets"]],
                         ["sampled_terrain","sampled_building","sampled_air","sampled_unknown","unobserved"])
        self.assertEqual(value["plan"]["counts"], dict.fromkeys(
            ["sampled_air","sampled_terrain","sampled_building","sampled_unknown","unobserved"],1))
        self.assertIsNone(value["plan"]["targets"][-1]["observed"])
        self.assertEqual(value["plan"]["targets"][0]["observed"], {"kind":"terrain","id":2,"fill":128})
        self.assertFalse(value["plan"]["executable"])
        self.assertFalse(value["plan"]["clearance_verified"])

    def test_invalid_or_excessive_geometry_preview_refuses(self):
        valid=preview([[20,40,60]])
        cases=[({**valid,'status':'placed'},'geometry-only'),({**valid,'executable':True},'geometry-only'),
               (preview([]),'1 to 256'),(preview([[True,40,60]]),'outside plan'),
               (preview([[20,40,60]]*2),'duplicate plan'),(preview([[20480,40,60]]),'outside plan'),
               (preview([[20+i,40,60] for i in range(257)]),'1 to 256'),
               ({**valid,'geometry':{'solid_cell_count':2}},'counted cells'),
               ({**valid,'grid':{'cell_size_m':1}},'plan grid')]
        for value,fragment in cases:
            with self.subTest(fragment=fragment):
                with self.assertRaisesRegex(ValueError,fragment):with_plan(scene(survey()),value)

    def test_bad_plan_is_refused_before_live_client_or_output_exists(self):
        with tempfile.TemporaryDirectory() as folder:
            folder=Path(folder);plan=folder/'plan.json';output=folder/'view.html'
            for invalid,fragment in [(preview([[20,40,60]]*2),'duplicate plan'),(None,'geometry-only')]:
                plan.write_text(json.dumps(invalid))
                with patch('terrain_view.Client') as client:
                    with self.assertRaisesRegex(ValueError,fragment):
                        main(['--port','8751','--box','10','20','30','1','1.5','2',
                              '--plan',str(plan),'--output',str(output)])
                    client.assert_not_called()
            self.assertFalse(output.exists())

    def test_offline_cli_adds_geometry_file_without_altering_observations(self):
        with tempfile.TemporaryDirectory() as folder:
            folder=Path(folder);source=folder/'survey.json';plan=folder/'plan.json';output=folder/'view.html'
            source.write_text(json.dumps({'survey':survey()}))
            plan.write_text(json.dumps(preview([[20,41,60],[22,40,60]])))
            with redirect_stdout(io.StringIO()):
                main(['--input',str(source),'--plan',str(plan),'--output',str(output)])
            payload=output.read_text().split('<script id="terrain-data" type="application/json">')[1].split('</script>')[0]
            value=json.loads(payload)
            self.assertEqual(value['cells'],scene(survey())['cells'])
            self.assertEqual(value['plan']['counts']['sampled_air'],1)
            self.assertEqual(value['plan']['counts']['unobserved'],1)

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
    def test_selection_sections_use_half_open_bounds_without_clamping_or_mutation(self):
        script = r'''
const assert = require('node:assert/strict');
require(process.argv[1]);
const sectionIndex = globalThis.TerrainGeometry.sectionIndex;
const min = Object.freeze([10,20,30]), max = Object.freeze([11,21.5,32]);
const point = Object.freeze([10.75,20.5,31.75]);
assert.equal(sectionIndex(point,'xy',min,max),63);
assert.equal(sectionIndex(point,'zy',min,max),21);
assert.equal(sectionIndex(min,'xy',min,max),60);
assert.equal(sectionIndex(min,'zy',min,max),20);
assert.equal(sectionIndex([.25,.25,.25],'xy',[0,0,0],[.5,.5,.5]),0);
for (const p of [[11,20,30],[10,21.5,30],[10,20,32],[9.999,20,30],[10,19.999,30],[10,20,29.999],[NaN,20,30]]) {
  assert.equal(sectionIndex(p,'xy',min,max),null);
  assert.equal(sectionIndex(p,'zy',min,max),null);
}
assert.equal(sectionIndex(point,'top',min,max),null);
'''
        result = subprocess.run([shutil.which("node"), "-e", script,
                                 str(Path(__file__).with_name("terrain_view.js"))], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    @unittest.skipUnless(shutil.which("node"), "Node is optional; required for renderer geometry checks")
    def test_column_profile_preserves_all_raw_bands_and_half_open_coordinates(self):
        script = r'''
const assert = require('node:assert/strict');
require(process.argv[1]);
const cells = [[20,40,60,'terrain',2,128],[21,40,60,'building',157,7],
               [20,42,60,'unknown',255,0],[20,41,60,'air',null,0],
               [20,40,61,'air',null,255]];
const before = JSON.stringify(cells), column = globalThis.TerrainGeometry.column;
assert.deepEqual(column(cells,[10.25,200,30.25]),[cells[2],cells[3],cells[0]]);
assert.deepEqual(column(cells,[10,20,30]),[cells[2],cells[3],cells[0]]);
assert.deepEqual(column(cells,[10.5,20,30]),[cells[1]]);
assert.deepEqual(column(cells,[10,20,30.5]),[cells[4]]);
assert.deepEqual(column(cells,[9.999,20,30]),[]);
assert.deepEqual(column(cells,[10,20,29.999]),[]);
assert.deepEqual(column(cells,[11,20,30]),[]);
assert.equal(JSON.stringify(cells),before);
'''
        result = subprocess.run([shutil.which("node"), "-e", script,
                                 str(Path(__file__).with_name("terrain_view.js"))],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

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
        result=subprocess.run([shutil.which("node"), "-e", script,
                               str(Path(__file__).with_name("terrain_view.js"))], capture_output=True,text=True)
        self.assertEqual(result.returncode,0,result.stderr)

    @unittest.skipUnless(shutil.which("node"), "Node is optional; required for renderer event checks")
    def test_standard_select_change_redraws_sections_and_range_input_updates_slice(self):
        # Standard EventTarget; canvas is a drawing sink, not a second renderer.
        script = r'''
const assert = require('node:assert/strict');
class Element extends EventTarget {
  constructor(tagName='DIV'){super();this.tagName=tagName;this.value='';this.hidden=false;this.dataset={};this.children=[];}
  append(...children){this.children.push(...children);}
  replaceChildren(...children){this.children=[...children];}
}
const ids=['terrain-data','terrain','view','yaw','pitch','zoom','slice','selection','measure','pins',
           'provenance','slice-control','yaw-control','pitch-control','slice-value','export',
           'show-plan','plan-target','plan-controls','plan-summary',
           'column-profile','column-title','column-summary','column-rows','section-xy','section-zy'];
const nodes=Object.fromEntries(ids.map(id=>[id,new Element()]));
nodes['terrain-data'].textContent=process.argv[2];
nodes.view.tagName='SELECT';nodes.view.value='orbit';nodes.yaw.value='320';
nodes.pitch.value='32';nodes.zoom.value='100';
nodes['show-plan'].checked=true;
nodes['column-profile'].hidden=true;
nodes.terrain.clientWidth=800;nodes.terrain.clientHeight=520;
const painted=[],polygons=[];let polygon=[],closed=false;
nodes.terrain.getContext=()=>new Proxy({fillText:text=>painted.push(text),
  beginPath:()=>{polygon=[];closed=false;},moveTo:(x,y)=>polygon.push([x,y]),lineTo:(x,y)=>polygon.push([x,y]),
  closePath:()=>{closed=true;},fill:()=>{if(closed&&polygon.length===4)polygons.push(polygon.slice());}
}, {get:(target,key)=>target[key]??(()=>{})});
nodes.terrain.getBoundingClientRect=()=>({left:0,top:0});
globalThis.document={getElementById:id=>nodes[id],createElement:tag=>new Element(tag.toUpperCase())};
globalThis.window={devicePixelRatio:1,addEventListener:()=>{}};
require(process.argv[1]);
assert.equal(nodes.terrain.dataset.view,'orbit');
assert.equal(nodes['slice-control'].hidden,true);
assert.equal(nodes['column-profile'].hidden,true);
assert.equal(nodes['section-xy'].disabled,true);
nodes['section-xy'].dispatchEvent(new Event('click'));
assert.equal(nodes.terrain.dataset.view,'orbit');
if(JSON.parse(process.argv[2]).pins.length===0){
  const face=polygons.at(-1),x=face.reduce((s,p)=>s+p[0],0)/4,y=face.reduce((s,p)=>s+p[1],0)/4;
  nodes.terrain.dispatchEvent(Object.assign(new Event('click'),{clientX:x,clientY:y}));
  assert.equal(nodes['column-title'].textContent,'Observed column at selected cell minimum');
  const cell=JSON.parse(nodes.selection.textContent.match(/Cell (\[[^\]]+\])/)[1]);
  assert.ok(JSON.parse(process.argv[2]).cells.some(c=>c.slice(0,3).join(',')===cell.join(',')));
  nodes['section-xy'].dispatchEvent(new Event('click'));
  assert.equal(nodes.terrain.dataset.view,'xy');assert.equal(nodes.slice.value,String(cell[2]));
  nodes['section-zy'].dispatchEvent(new Event('click'));
  assert.equal(nodes.terrain.dataset.view,'zy');assert.equal(nodes.slice.value,String(cell[0]));
  process.exit(0);
}
nodes.pins.children[0].dispatchEvent(new Event('click'));
assert.equal(nodes['column-profile'].hidden,false);
assert.equal(nodes['column-title'].textContent,'Observed column at Band boundary');
assert.ok(nodes['column-summary'].textContent.includes('X [10, 10.5) m; Z [30, 30.5) m'));
assert.equal(nodes['column-rows'].children.length,3);
const values=row=>row.children.map(cell=>cell.textContent);
assert.deepEqual(values(nodes['column-rows'].children[1]),['[20.5, 21)','air (no ID)','0','Selected Y']);
assert.deepEqual(values(nodes['column-rows'].children[2]),['[20, 20.5)','terrain ID 2','128','']);
nodes.pins.children[1].dispatchEvent(new Event('click'));
assert.equal(nodes['column-rows'].children.length,0);
assert.equal(nodes['column-summary'].textContent,'Outside sampled X/Z; column unobserved.');
nodes.pins.children[2].dispatchEvent(new Event('click'));
assert.equal(nodes['column-rows'].children.length,3);
assert.ok(nodes['column-summary'].textContent.includes('Selected Y 21.5 m OUTSIDE sampled Y.'));
assert.ok(nodes['column-rows'].children.every(row=>row.dataset.containsPoint==='false'));
nodes.pins.children[3].dispatchEvent(new Event('click'));
assert.deepEqual(values(nodes['column-rows'].children[0]),['[21, 21.5)','building ID 157','7','Selected Y']);
nodes.pins.children[4].dispatchEvent(new Event('click'));
assert.ok(nodes['column-summary'].textContent.includes('Selected Y 19.5 m OUTSIDE sampled Y.'));
assert.ok(nodes['column-rows'].children.every(row=>row.dataset.containsPoint==='false'));
const hasPlan=Boolean(JSON.parse(process.argv[2]).plan);
assert.equal(nodes['plan-controls'].hidden,!hasPlan);
nodes.view.value='xy';nodes.view.dispatchEvent(new Event('change'));
assert.equal(nodes.terrain.dataset.view,'xy');
assert.equal(nodes.terrain.dataset.renderedFaces,'6');
assert.equal(nodes['slice-control'].hidden,false);
assert.equal(nodes['yaw-control'].hidden,true);
nodes.view.value='zy';nodes.view.dispatchEvent(new Event('change'));
assert.equal(nodes.terrain.dataset.renderedFaces,'12');
nodes.slice.value='21';nodes.slice.dispatchEvent(new Event('input'));
assert.equal(nodes['slice-value'].textContent,'X [10.5, 11) m');
assert.ok(painted.some(text=>text.includes('Section X [10.5, 11) m')),'export must identify section interval');
nodes.view.value='top';nodes.view.dispatchEvent(new Event('change'));
assert.equal(nodes.terrain.dataset.view,'top');
assert.equal(nodes.terrain.dataset.renderedFaces,'8');
assert.equal(nodes['slice-control'].hidden,true);
nodes.pins.children[0].dispatchEvent(new Event('click'));
assert.equal(nodes['section-xy'].disabled,false);
nodes['section-xy'].dispatchEvent(new Event('click'));
assert.equal(nodes.terrain.dataset.view,'xy');
assert.equal(nodes.slice.value,'60');
assert.equal(nodes['slice-value'].textContent,'Z [30, 30.5) m');
assert.ok(painted.some(text=>text.includes('Section Z [30, 30.5) m')));
nodes.pins.children[3].dispatchEvent(new Event('click'));
nodes['section-zy'].dispatchEvent(new Event('click'));
assert.equal(nodes.terrain.dataset.view,'zy');
assert.equal(nodes.slice.value,'21');
assert.equal(nodes['slice-value'].textContent,'X [10.5, 11) m');
nodes.pins.children[1].dispatchEvent(new Event('click'));
assert.equal(nodes['section-xy'].disabled,true);
assert.equal(nodes['section-zy'].disabled,true);
nodes['section-xy'].dispatchEvent(new Event('click'));
assert.equal(nodes.terrain.dataset.view,'zy');
assert.equal(nodes.slice.value,'21');
nodes.view.value='top';nodes.view.dispatchEvent(new Event('change'));
if(hasPlan){
  assert.equal(nodes.terrain.dataset.proposedTargets,'3');
  const observed=nodes.terrain.dataset.renderedFaces;
  nodes['show-plan'].checked=false;nodes['show-plan'].dispatchEvent(new Event('change'));
  assert.equal(nodes.terrain.dataset.proposedTargets,'0');
  assert.equal(nodes.terrain.dataset.renderedFaces,observed);
  nodes['show-plan'].checked=true;nodes['show-plan'].dispatchEvent(new Event('change'));
  assert.equal(nodes.terrain.dataset.proposedTargets,'3');
  nodes['plan-target'].value='3';nodes['plan-target'].dispatchEvent(new Event('change'));
  assert.ok(nodes.selection.textContent.includes('unobserved; outside capture, not drawn'));
  assert.ok(nodes.selection.textContent.includes('execution NOT authorized'));
  assert.equal(nodes['section-xy'].disabled,true);
  nodes['section-xy'].dispatchEvent(new Event('click'));
  assert.equal(nodes.terrain.dataset.view,'top');
  nodes['plan-target'].value='0';nodes['plan-target'].dispatchEvent(new Event('change'));
  nodes['section-xy'].dispatchEvent(new Event('click'));
  assert.equal(nodes.terrain.dataset.view,'xy');
  assert.equal(nodes.slice.value,'60');
}
'''
        base=scene(survey(), [
            {"label": "Band boundary", "position": [10.25, 20.5, 30.25]},
            {"label": "Outside column", "position": [11, 20.5, 30.25]},
            {"label": "Above capture", "position": [10.25, 21.5, 30.25]},
            {"label": "Partial building", "position": [10.75, 21.25, 31.75]},
            {"label": "Below capture", "position": [10.25, 19.5, 30.25]},
        ])
        for value in [base,with_plan(base,preview([[20,40,60],[20,41,60],[21,42,63],[22,40,60]])),scene(survey())]:
            result=subprocess.run([shutil.which("node"), "-e", script,
                                   str(Path(__file__).with_name("terrain_view.js")),
                                   json.dumps(value)], capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)


def preview(cells):
    return {'status':'geometry_preview_only','executable':False,'grid':{'cell_size_m':.5},
            'geometry':{'solid_cell_count':len(cells)},'cell_minima':cells}


if __name__ == "__main__": unittest.main()
