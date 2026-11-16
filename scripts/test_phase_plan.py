"""Behavior tests for read-only phase decomposition with a bounded geometry oracle."""
import unittest
from collections import Counter
from phase_plan import compile_phases, mcp_preview, main


def floor(name, x=0, z=0, width=4, depth=4):
    return dict(id=name, kind="floor", x=x, y=4, z=z, width=width, height=.5, depth=depth)


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
        compile_phases(mcp_preview(Client()),{"material":"stone","parts":[floor("a")]})

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
