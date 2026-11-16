"""Behavior tests for read-only phase decomposition with a bounded geometry oracle."""
import unittest
from collections import Counter
from phase_plan import compile_phases, mcp_preview, main


def floor(name, x=0, z=0, width=4, depth=4):
    return dict(id=name, kind="floor", x=x, y=4, z=z, width=width, height=.5, depth=depth)


def route(poses=None, **fields):
    return {"id":"access", "body":{"width":1,"height":2,"depth":1},
            "poses":poses if poses is not None else [[4,4.5,4],[4,4.5,5]], **fields}


def oracle(args):
    if args["kind"]=="gable_roof":
        cells={(4+x,7+y,4+z) for x,top in enumerate([1,2,3,4,4,3,2,1]) for y in range(top) for z in range(12)}
        return {"status":"geometry_preview_only","executable":False,"cell_minima":[list(c) for c in sorted(cells)]}
    parts = args["parts"] if args["kind"] == "composite" else [args]
    cells = {(x, int(p["y"]*2), z) for p in parts
             for x in range(int(p["x"]*2), int((p["x"]+p["width"])*2))
             for z in range(int(p["z"]*2), int((p["z"]+p["depth"])*2))}
    return {"status": "geometry_preview_only", "executable": False,
            "cell_minima": [list(c) for c in sorted(cells)]}


def clearance_oracle(volumes, cells):
    reports=[]
    for v in volumes:
        low=[v[k] for k in ("x","y","z")]
        high=[low[i]+v[k] for i,k in enumerate(("width","height","depth"))]
        conflicts=sorted(list(c) for c in cells if all(low[i] <= c[i]/2 < high[i] for i in range(3)))
        reports.append(dict(min_m=low,max_exclusive_m=high,
                            requested_cell_count=int(v["width"]*v["height"]*v["depth"]*8),
                            planned_clear=not conflicts,conflicting_cell_count=len(conflicts),conflicting_cell_minima=conflicts))
    return dict(scope="planned_cells_only",world_observed=False,walkability_verified=False,volumes=reports)


class PhaseTests(unittest.TestCase):
    def test_declared_route_checks_footing_headspace_and_preserves_exact_legacy_phases(self):
        document={"material":"stone","parts":[floor("deck",x=4,z=4,width=1,depth=2)]}
        legacy=compile_phases(oracle,document)
        report=compile_phases(oracle,{**document,"routes":[route()]})
        audit=report.pop("routes")
        self.assertEqual(report,legacy)
        self.assertTrue(audit["checks"][0]["planned_route_valid"])
        self.assertTrue(all(p["planned_footing_present"] and p["planned_headspace_clear"] for p in audit["checks"][0]["poses"]))
        for key in ("world_observed","walkability_verified","continuous_sweep_verified"):
            self.assertFalse(audit[key])
        self.assertEqual(audit["vertical_delta_limit_m"],.5)

    def test_clear_headspace_does_not_hide_missing_footing_or_a_gap_between_poses(self):
        report=compile_phases(oracle,{"material":"stone","parts":[floor("deck",x=4,z=4,width=1,depth=1)],"routes":[route()]})
        checked=report["routes"]["checks"][0]
        self.assertFalse(checked["planned_route_valid"])
        self.assertTrue(checked["connected"])
        self.assertTrue(checked["poses"][1]["planned_headspace_clear"])
        self.assertEqual(checked["poses"][1]["missing_footing_cell_minima"],[[8,8,10],[8,8,11],[9,8,10],[9,8,11]])
        report=compile_phases(oracle,{"material":"stone","parts":[floor("deck",x=4,z=4,width=1,depth=3)],
                                     "routes":[route([[4,4.5,4],[4,4.5,6]])]})
        checked=report["routes"]["checks"][0]
        self.assertFalse(checked["connected"])
        self.assertFalse(checked["planned_route_valid"])
        self.assertTrue(all(p["planned_footing_present"] for p in checked["poses"]))

    def test_route_headspace_includes_later_modules_and_excludes_touching_upper_face(self):
        deck=floor("deck",x=4,z=4,width=1,depth=2)
        roof={**floor("roof",x=4,z=5,width=1,depth=1),"y":6}
        report=compile_phases(oracle,{"material":"stone","parts":[deck,roof],"routes":[route()]})
        pose=report["routes"]["checks"][0]["poses"][1]
        self.assertEqual(pose["headspace_conflicting_cell_minima"],[[8,12,10],[8,12,11],[9,12,10],[9,12,11]])
        self.assertFalse(pose["planned_headspace_clear"])
        roof["y"]=6.5
        report=compile_phases(oracle,{"material":"stone","parts":[deck,roof],"routes":[route()]})
        self.assertTrue(report["routes"]["checks"][0]["planned_route_valid"])

    def test_route_admission_is_bounded_and_finishes_before_any_preview(self):
        bad=[[],[route()]*9,[route(),route()],
             [route(extra=True)],[route(body={"width":2.5,"height":2,"depth":1})],
             [route(body={"width":1,"height":4.5,"depth":1})],
             [route(body={"width":1,"height":float("nan"),"depth":1})],
             [route(poses=[])],[route(poses=[[4,4.5,4]])],[route(poses=[[4,4.5,4]]*65)],
             [route(poses=[[4,4.25,4],[4,4.5,5]])],[route(poses=[[4,0,4],[4,.5,5]])],
             [route(poses=[[True,4.5,4],[4,4.5,5]])],[route(poses=[[10240,4.5,4],[4,4.5,5]])]]
        for routes in bad:
            calls=[]
            def preview(args):calls.append(args);return oracle(args)
            with self.subTest(routes=routes),self.assertRaises(ValueError):
                compile_phases(preview,{"material":"stone","parts":[floor("deck")],"routes":routes})
            self.assertEqual(calls,[])

    def test_routes_accept_both_directions_on_each_axis_but_report_high_steps_and_diagonals(self):
        for dx,dz in [(0,.5),(0,-.5),(.5,0),(-.5,0)]:
            for rise in [.5,1]:
                poses=[[4,4.5,4],[4+dx,4.5+rise,4+dz]]
                parts=[{**floor(str(i),x=x,z=z,width=.5,depth=.5),"y":y-.5} for i,(x,y,z) in enumerate(poses)]
                paths=[route(poses=poses,id="up",body={"width":.5,"height":.5,"depth":.5}),
                       route(poses=list(reversed(poses)),id="down",body={"width":.5,"height":.5,"depth":.5})]
                report=compile_phases(oracle,{"material":"stone","parts":parts,"routes":paths})
                with self.subTest(dx=dx,dz=dz,rise=rise):
                    self.assertTrue(all(p["planned_footing_present"] and p["planned_headspace_clear"] for r in report["routes"]["checks"] for p in r["poses"]))
                    self.assertEqual([r["planned_route_valid"] for r in report["routes"]["checks"]],[rise==.5]*2)
        for poses in [[[4,4.5,4],[4.5,4.5,4.5]],[[4,4.5,4],[4,4.5,4]]]:
            report=compile_phases(oracle,{"material":"stone","parts":[floor("deck",x=4,z=4,width=2,depth=2)],"routes":[route(poses)]})
            self.assertFalse(report["routes"]["checks"][0]["connected"])

    def test_route_workload_limits_and_per_module_scope_are_explicit(self):
        paths=[route(poses=[[4,4.5,4],[4,4.5,5]]*32,id=str(i)) for i in range(8)]
        document={"material":"stone","parts":[floor("deck",x=4,z=4,width=1,depth=2)],"routes":paths}
        report=compile_phases(oracle,document)
        self.assertEqual(len(report["routes"]["checks"]),8)
        self.assertTrue(all(len(r["poses"])==64 for r in report["routes"]["checks"]))
        self.assertTrue(all(r["planned_route_valid"] for r in report["routes"]["checks"]))
        calls=[]
        def preview(args):calls.append(args);return oracle(args)
        with self.assertRaisesRegex(ValueError,"document level"):
            compile_phases(preview,{"material":"stone","parts":[{**floor("deck"),"routes":paths}]})
        self.assertEqual(calls,[])

    def test_per_module_clearance_is_not_silently_discarded(self):
        module={**floor("a"),"clearance":[dict(x=0,y=4,z=0,width=.5,height=.5,depth=.5)]}
        with self.assertRaisesRegex(ValueError,"clearance"):
            compile_phases(oracle,{"material":"stone","parts":[module]})

    def test_clearance_checks_complete_union_and_preserves_half_open_boundaries(self):
        calls=[]
        def checked(args):
            calls.append(args)
            result=oracle(args)
            if "clearance" in args:
                result["clearance"]=clearance_oracle(args["clearance"], result["cell_minima"])
            return result
        volumes=[dict(x=100,y=4,z=0,width=.5,height=.5,depth=.5),
                 dict(x=100,y=4.5,z=0,width=.5,height=.5,depth=.5)]
        doc={"material":"stone","parts":[floor("near",width=.5,depth=.5),floor("far",x=100,width=.5,depth=.5)],"clearance":volumes}
        result=compile_phases(checked,doc)
        self.assertEqual(len(result["phases"]),2)
        self.assertFalse(result["clearance"]["volumes"][0]["planned_clear"])
        self.assertEqual(result["clearance"]["volumes"][0]["conflicting_cell_minima"],[[200,8,0]])
        self.assertTrue(result["clearance"]["volumes"][1]["planned_clear"])
        self.assertFalse(result["clearance"]["world_observed"])
        self.assertFalse(result["clearance"]["walkability_verified"])
        self.assertEqual(sum("clearance" in call for call in calls),2)
        plain={k:v for k,v in doc.items() if k!="clearance"}
        expected=compile_phases(oracle,plain)
        result.pop("clearance")
        self.assertEqual(result,expected)

    def test_clearance_more_than_one_native_batch_and_untrusted_reports(self):
        def checked(args):
            result=oracle(args)
            if "clearance" in args:
                self.assertEqual(len(args["clearance"]),1)
                result["clearance"]=clearance_oracle(args["clearance"], result["cell_minima"])
            return result
        volume=dict(x=0,y=5,z=0,width=.5,height=.5,depth=.5)
        doc={"material":"stone","parts":[floor("a")],"clearance":[volume]*32}
        self.assertEqual(len(compile_phases(checked,doc)["clearance"]["volumes"]),32)
        for bad in [[],[volume]*33,None,[{**volume,"width":.25}],[{**volume,"x":True}],
                    [{**volume,"height":16,"width":16,"depth":16}],[{**volume,"x":10240}],
                    [{**volume,"confirm":"server-write"}]]:
            with self.assertRaises(ValueError):compile_phases(checked,{**doc,"clearance":bad})
        with self.assertRaisesRegex(ValueError,"clearance"):
            compile_phases(oracle,{**doc,"clearance":[volume]})
        def dishonest(args):
            result=checked(args)
            if "clearance" in args:result["clearance"]["world_observed"]=True
            return result
        with self.assertRaisesRegex(ValueError,"clearance"):
            compile_phases(dishonest,{**doc,"clearance":[volume]})

    def test_cli_bounds_input_before_transport_and_preserves_existing_output(self):
        import contextlib, io, json, tempfile
        from pathlib import Path
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as folder:
            source=Path(folder)/"input.json";output=Path(folder)/"output.json"
            source.write_bytes(b"x"*(1024*1024+1))
            with patch("sys.argv",["phase_plan",str(source),str(output)]),patch("phase_plan.Client") as client,contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit):main()
                client.assert_not_called()
            source.write_text(json.dumps({"material":"stone","parts":[floor("a")]}))
            output.write_text("keep")
            class Fake:
                def call(self,method,args):
                    return {"isError":False,"content":[{"text":json.dumps(oracle(args["arguments"]))}]}
            with patch("sys.argv",["phase_plan",str(source),str(output)]),patch("phase_plan.Client",return_value=Fake()):
                with self.assertRaises(FileExistsError):main()
            self.assertEqual(output.read_text(),"keep")
    def test_compatible_module_fragments_share_phase_with_provenance(self):
        doc={"material":"stone","parts":[floor("a",width=.5,depth=.5),floor("b",x=.5,width=.5,depth=.5)]}
        result=compile_phases(oracle,doc)
        self.assertEqual(len(result["phases"]),1)
        self.assertEqual(result["phases"][0]["modules"],["a","b"])
    def test_one_legal_roof_does_not_gain_unnecessary_phases(self):
        doc={"material":"stone","parts":[dict(id="roof",kind="gable_roof",x=2,y=3.5,z=2,width=4,height=2,depth=6)]}
        result=compile_phases(oracle,doc)
        self.assertEqual(result["unique_cells"],240)
        self.assertEqual(len(result["phases"]),1)
    def test_overlap_is_removed_and_reordered_modules_keep_exact_union(self):
        a, b = floor("a"), floor("b", x=2)
        first = compile_phases(oracle, {"material":"stone", "parts":[a,b]})
        second = compile_phases(oracle, {"material":"stone", "parts":[b,a]})
        self.assertEqual(first["cell_minima"], second["cell_minima"])
        self.assertEqual(first["unique_cells"], 96)
        self.assertFalse(first["executable"])
        cells = [tuple(c) for p in first["phases"] for c in oracle(p["plan"])["cell_minima"]]
        self.assertEqual(len(cells), len(set(cells)))

    def test_large_and_distant_modules_split_without_relaxing_limits(self):
        r = compile_phases(oracle, {"material":"stone", "parts":[floor("a",width=8,depth=8),floor("b",x=100)]})
        self.assertEqual(r["unique_cells"],320)
        self.assertGreaterEqual(len(r["phases"]),2)
        for p in r["phases"]:
            cells = oracle(p["plan"])["cell_minima"]
            self.assertLessEqual(len(cells),256)
            self.assertLessEqual(len(p["plan"]["parts"]),8)
            bricks=Counter(tuple(n//8 for n in c) for c in cells)
            self.assertLessEqual(len(bricks),8)
            self.assertLessEqual(max(bricks.values()),64)
            for axis in range(3):
                self.assertLessEqual(max(c[axis] for c in cells)+1-min(c[axis] for c in cells),32)

    def test_bad_input_and_untrusted_geometry_refuse(self):
        for doc in [{"material":"stone","parts":[]},{"material":"stone","parts":[floor("a"),floor("a")]},
                    {"material":"stone","parts":[{**floor("a"),"material":"other"}]},
                    {"material":"stone","parts":[floor("a")],"confirm":"server-write"}]:
            with self.assertRaises(ValueError): compile_phases(oracle,doc)
        for cells in [[[True,0,0]],[[0,0]],[[0,0,0],[0,0,0]],[[0,-1,0]]]:
            with self.assertRaises(ValueError):
                compile_phases(lambda a:{"status":"geometry_preview_only","executable":False,"cell_minima":cells}, {"material":"stone","parts":[floor("a")]})

    def test_phase_oracle_mismatch_refuses_report(self):
        def bad(args):
            result=oracle(args)
            if args["kind"]=="composite":result["cell_minima"].pop()
            return result
        with self.assertRaises(ValueError):compile_phases(bad,{"material":"stone","parts":[floor("a")]})

    def test_transport_calls_only_pure_building_plan(self):
        class Client:
            def call(self,method,params):
                self_call=(method,params["name"])
                if self_call != ("tools/call","building_plan"):raise AssertionError(self_call)
                import json
                return {"isError":False,"content":[{"text":json.dumps(oracle(params["arguments"]))}]}
        compile_phases(mcp_preview(Client()),{"material":"stone","parts":[floor("a",x=4,z=4,width=1,depth=2)],"routes":[route()]})

    def test_workload_and_phase_caps_refuse_without_execution(self):
        with self.assertRaises(ValueError):
            compile_phases(oracle,{"material":"stone","parts":[floor(str(i)) for i in range(33)]})
        with self.assertRaises(ValueError):
            compile_phases(oracle,{"material":"stone","parts":[floor(str(i),x=i*20,width=16,depth=16) for i in range(9)]})
        def scattered(args):
            if args["kind"]=="composite":return oracle(args)
            return {"status":"geometry_preview_only","executable":False,"cell_minima":[[40*i,8,0] for i in range(65)]}
        with self.assertRaisesRegex(ValueError,"64 phases"):
            compile_phases(scattered,{"material":"stone","parts":[floor("a")]})


if __name__ == "__main__": unittest.main()
