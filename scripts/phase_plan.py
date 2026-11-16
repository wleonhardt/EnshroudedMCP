"""Read-only decomposition using only pure building_plan previews. Never executes phases."""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import re

from investigate import Client, InvalidReply


def mcp_preview(client):
    def preview(args):
        reply = client.call("tools/call", {"name":"building_plan", "arguments":args})
        if reply.get("isError"):
            raise ValueError(reply["content"][0]["text"])
        return json.loads(reply["content"][0]["text"])
    return preview


def _cells(report):
    if not isinstance(report,dict): raise InvalidReply("expected geometry object")
    cells = report.get("cell_minima")
    if (report.get("status") != "geometry_preview_only" or report.get("executable") is not False
            or not isinstance(cells,list) or not 1 <= len(cells) <= 1024):
        raise InvalidReply("expected bounded pure geometry preview")
    for c in cells:
        if (not isinstance(c,list) or len(c)!=3 or any(type(n) is not int for n in c)
                or not all(0 <= n < bound for n,bound in zip(c,(20480,2048,20480)))):
            raise InvalidReply("invalid complete cell minimum")
    result = {tuple(c) for c in cells}
    if len(result)!=len(cells): raise InvalidReply("duplicate preview cells")
    return result


def _rectangles(cells, brick_aligned=False):
    rows = defaultdict(list)
    for x,y,z in cells: rows[y,z].append(x)
    spans = defaultdict(list)
    for (y,z),xs in sorted(rows.items()):
        xs=sorted(xs); start=prev=xs[0]
        for x in xs[1:]+[None]:
            if x is not None and x==prev+1 and (not brick_aligned or x//8==start//8):
                prev=x; continue
            spans[start,y,prev-start+1].append(z); start=prev=x
    result=[]
    for (x,y,w),zs in sorted(spans.items()):
        zs=sorted(zs); start=prev=zs[0]
        for z in zs[1:]+[None]:
            if z is not None and z==prev+1 and (not brick_aligned or z//8==start//8):
                prev=z; continue
            result.append(dict(id=f"rect_{len(result)}",kind="floor",x=x/2,y=y/2,z=start/2,
                               width=w/2,height=.5,depth=(prev-start+1)/2))
            start=prev=z
    return result


def _rectangle_cells(parts):
    return {(x,int(p["y"]*2),z) for p in parts
            for x in range(int(p["x"]*2),int((p["x"]+p["width"])*2))
            for z in range(int(p["z"]*2),int((p["z"]+p["depth"])*2))}


def _fits(parts):
    cells=_rectangle_cells(parts)
    bricks=Counter(tuple(n//8 for n in c) for c in cells)
    return (len(parts)<=8 and len(cells)<=256 and len(bricks)<=8 and max(bricks.values())<=64
            and all(max(c[i] for c in cells)+1-min(c[i] for c in cells)<=32 for i in range(3)))


def _volume(volume):
    keys=("x","y","z","width","height","depth")
    if not isinstance(volume,dict) or set(volume)!=set(keys):
        raise ValueError("clearance requires exactly x, y, z, width, height and depth")
    for key,low,high in zip(keys,(0,0,0,.5,.5,.5),(10240,1024,10240,16,16,16)):
        n=volume[key]
        if type(n) not in (int,float) or not low<=n<=high or n*2%1:
            raise ValueError("clearance must use bounded half-metre coordinates and dimensions")
    start=tuple(int(volume[k]*2) for k in keys[:3])
    size=tuple(int(volume[k]*2) for k in keys[3:])
    end=tuple(a+b for a,b in zip(start,size))
    count=size[0]*size[1]*size[2]
    if count>1024 or any(n>bound for n,bound in zip(end,(20480,2048,20480))):
        raise ValueError("clearance exceeds world bounds or 1,024 cells per volume")
    return start,end,count


def _clearance_report(volumes, cells):
    reports=[]
    for volume in volumes:
        start,end,count=_volume(volume)
        conflicts=[list(c) for c in sorted(cells) if all(start[i]<=c[i]<end[i] for i in range(3))]
        reports.append(dict(min_m=[n/2 for n in start],max_exclusive_m=[n/2 for n in end],
                            requested_cell_count=count,planned_clear=not conflicts,
                            conflicting_cell_count=len(conflicts),conflicting_cell_minima=conflicts))
    return dict(scope="planned_cells_only",world_observed=False,walkability_verified=False,volumes=reports)


def compile_phases(preview, document):
    if (not isinstance(document,dict) or not {"material","parts"}<=set(document)
            or set(document)-{"material","parts","clearance"}):
        raise ValueError("expected material, parts and optional clearance")
    if "clearance" in document:
        volumes=document["clearance"]
        if not isinstance(volumes,list) or not 1<=len(volumes)<=32:
            raise ValueError("clearance requires 1..32 global volumes")
        for volume in volumes:_volume(volume)
    parts=document["parts"]; material=document["material"]
    if not isinstance(parts,list) or not 1<=len(parts)<=32:
        raise ValueError("requires 1..32 primitive modules")
    if not isinstance(material,str) or not re.fullmatch(r"[A-Za-z0-9_.:-]{1,128}",material):
        raise ValueError("invalid unverified material label")
    ids=set()
    for p in parts:
        if (not isinstance(p,dict) or not isinstance(p.get("id"),str)
                or not re.fullmatch(r"[A-Za-z0-9_-]{1,64}",p["id"]) or p["id"] in ids
                or "material" in p or p.get("kind") not in ("floor","wall","doorway","staircase","gable_roof")):
            raise ValueError("requires uniquely named primitives without per-part material")
        if "clearance" in p:raise ValueError("put clearance at document level; per-module clearance is unsupported")
        ids.add(p["id"])
    phases=[]; union=set(); modules=[]; batch=[]; owners=[]; rectangle_index=0
    def emit(owners,batch):
        if len(phases)>=64: raise ValueError("phase plan exceeds 64 phases")
        plan={"kind":"composite","material":material,"parts":batch}
        expected=_rectangle_cells(batch)
        if _cells(preview(plan))!=expected: raise InvalidReply("phase preview differs from exact target set")
        phases.append({"index":len(phases),"modules":list(dict.fromkeys(owners)),"plan":plan,"solid_cells":len(expected)})
    for p in parts:
        args={k:v for k,v in p.items() if k!="id"}; args["material"]=material
        cells=_cells(preview(args)); new=cells-union
        if len(union|cells)>8192: raise ValueError("phase plan exceeds 8,192 distinct cells")
        modules.append({"id":p["id"],"new_cells":len(new),"overlap_cells":len(cells)-len(new)})
        union|=cells
        rects=[]
        for rect in _rectangles(new):
            rects.extend([rect] if _fits([rect]) else _rectangles(_rectangle_cells([rect]),True))
        for rect in rects:
            rect["id"]=f"rect_{rectangle_index}"; rectangle_index+=1
            if not _fits(batch+[rect]): emit(owners,batch); batch=[]; owners=[]
            batch.append(rect); owners.append(p["id"])
    if batch: emit(owners,batch)
    actual=[c for phase in phases for c in _rectangle_cells(phase["plan"]["parts"])]
    if len(actual)!=len(union) or set(actual)!=union: raise InvalidReply("phase union mismatch")
    result={"status":"phase_plan_only","executable":False,"world_observed":False,
            "material_reference":material,"material_validated":False,"modules":modules,"phases":phases,
            "unique_cells":len(union),"cell_minima":[list(c) for c in sorted(union)],
            "unchecked":["current occupants and source material","actor, rights and persistence",
                         "operation capacity and journal/native undo storage","construction order, stability, rendering, support and traversal",
                         "functional props, native phased execution and recovery"]}
    if "clearance" in document:
        plan=phases[0]["plan"]; first=_rectangle_cells(plan["parts"])
        for volume in volumes:
            checked=preview({**plan,"clearance":[volume]})
            if _cells(checked)!=first or checked.get("clearance")!=_clearance_report([volume],first):
                raise InvalidReply("native clearance preview differs from requested volume and phase cells")
        result["clearance"]=_clearance_report(volumes,union)
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input",type=Path); parser.add_argument("output",type=Path)
    parser.add_argument("--port",type=int,default=8753)
    args=parser.parse_args()
    if not 1<=args.port<=65535: parser.error("port must be 1..65535")
    with args.input.open("rb") as stream: raw=stream.read(1024*1024+1)
    if len(raw)>1024*1024: parser.error("input exceeds 1 MiB")
    result=compile_phases(mcp_preview(Client(args.port)),json.loads(raw))
    with args.output.open("x",encoding="utf-8") as stream:json.dump(result,stream,indent=2)
    print(f"Planned {result['unique_cells']} cells in {len(result['phases'])} phases; no execution or world observation.")


if __name__=="__main__": main()
