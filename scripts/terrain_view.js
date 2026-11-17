/* Local diagnostic projection. No game input, network, engine mesh or collision. */
(() => {
  "use strict";
  function project(p, view, yaw = 0, pitch = 0) {
    if (view === "top") return {x: p[0], y: -p[2], depth: p[1]};
    if (view === "xy") return {x: p[0], y: -p[1], depth: p[2]};
    if (view === "zy") return {x: p[2], y: -p[1], depth: p[0]};
    const r = p[0] * Math.cos(yaw) + p[2] * Math.sin(yaw);
    const d = -p[0] * Math.sin(yaw) + p[2] * Math.cos(yaw);
    return {x: r, y: -p[1] * Math.cos(pitch) + d * Math.sin(pitch),
      depth: p[1] * Math.sin(pitch) + d * Math.cos(pitch)};
  }
  function section(cells, view, slice) {
    const axis = view === "xy" ? 2 : 0;
    return cells.filter(c => c[axis] === slice);
  }
  function topCells(cells) {
    const columns = new Map();
    for (const c of cells) {
      const key = `${c[0]},${c[2]}`, old = columns.get(key);
      if (!old || (c[3] !== "air" && (old[3] === "air" || c[1] > old[1]))) columns.set(key, c);
    }
    return [...columns.values()];
  }
  function column(cells, position) {
    const x = Math.floor(position[0]*2), z = Math.floor(position[2]*2);
    return cells.filter(c => c[0] === x && c[2] === z).sort((a,b) => b[1]-a[1]);
  }
  function sectionIndex(position, view, min, max) {
    if (!["xy","zy"].includes(view) || position.some((v,i) => !Number.isFinite(v) || v < min[i] || v >= max[i])) return null;
    return Math.floor(position[view === "xy" ? 2 : 0]*2);
  }
  const normals = [[-1,0,0],[1,0,0],[0,-1,0],[0,1,0],[0,0,-1],[0,0,1]];
  const corners = [
    [[0,0,0],[0,0,1],[0,1,1],[0,1,0]], [[1,0,0],[1,1,0],[1,1,1],[1,0,1]],
    [[0,0,0],[1,0,0],[1,0,1],[0,0,1]], [[0,1,0],[0,1,1],[1,1,1],[1,1,0]],
    [[0,0,0],[0,1,0],[1,1,0],[1,0,0]], [[0,0,1],[1,0,1],[1,1,1],[0,1,1]]
  ];
  function exposedFaces(cells, yaw, pitch) {
    const lookup = new Map(cells.map(c => [c.slice(0,3).join(","), c]));
    const direction = [-Math.sin(yaw)*Math.cos(pitch), Math.sin(pitch), Math.cos(yaw)*Math.cos(pitch)];
    const faces = [];
    for (const c of cells) {
      if (c[3] === "air") continue;
      normals.forEach((normal, index) => {
        if (normal.reduce((n,v,i) => n + v*direction[i], 0) <= 1e-8) return;
        const neighbor = lookup.get(normal.map((v,i) => c[i]+v).join(","));
        if (neighbor && (neighbor[3] === "terrain" || neighbor[3] === "building")) return;
        faces.push({cell: c, face: index, points: corners[index].map(p => p.map((v,i) => (c[i]+v)/2))});
      });
    }
    return faces;
  }
  globalThis.TerrainGeometry = {project, section, topCells, column, sectionIndex, exposedFaces};
  if (typeof document === "undefined") return;

  const data = JSON.parse(document.getElementById("terrain-data").textContent);
  const canvas = document.getElementById("terrain"), ctx = canvas.getContext("2d");
  const controls = Object.fromEntries(["view","yaw","pitch","zoom","slice"].map(k => [k,document.getElementById(k)]));
  const info = document.getElementById("selection");
  const measure = document.getElementById("measure");
  const proposal = data.plan || null, showPlan=document.getElementById("show-plan"), planTarget=document.getElementById("plan-target");
  const colors = {air: "#253347", terrain: "#b5986b", building: "#63b7d0", unknown: "#be97e8"};
  let pick = [], selected = null, selectedTarget = null;
  let selectedPosition = null, selectionText = "";
  const sectionButtons = {xy:document.getElementById("section-xy"),zy:document.getElementById("section-zy")};
  for (const [view, button] of Object.entries(sectionButtons)) {
    button.disabled = true;
    button.addEventListener("click", () => {
      const index = selectedPosition === null ? null : sectionIndex(selectedPosition,view,data.min_m,data.max_m);
      if (index === null) return;
      controls.view.value = view;setupSlice();controls.slice.value = String(index);draw();
    });
  }
  const center = data.min_m.map((v,i) => (v + data.max_m[i])/2);
  const local = p => p.map((v,i) => v-center[i]);
  const bounds = [];
  for (let x=0;x<2;x++) for(let y=0;y<2;y++) for(let z=0;z<2;z++) {
    bounds.push([x?data.max_m[0]:data.min_m[0],y?data.max_m[1]:data.min_m[1],z?data.max_m[2]:data.min_m[2]]);
  }
  function describeColumn(position, label) {
    selectedPosition = position.slice();
    for (const [view, button] of Object.entries(sectionButtons)) {
      button.disabled = sectionIndex(position,view,data.min_m,data.max_m) === null;
      button.title = button.disabled ? "Selection is outside the captured box; no section selected." : "Open the exact containing half-metre section.";
    }
    document.getElementById("column-profile").hidden = false;
    document.getElementById("column-title").textContent = `Observed column at ${label}`;
    const rows = column(data.cells, position), body = document.getElementById("column-rows");
    body.replaceChildren();
    const x = Math.floor(position[0]*2), z = Math.floor(position[2]*2);
    const insideY = position[1] >= data.min_m[1] && position[1] < data.max_m[1];
    document.getElementById("column-summary").textContent = rows.length
      ? `X [${x/2}, ${(x+1)/2}) m; Z [${z/2}, ${(z+1)/2}) m; sampled Y [${data.min_m[1]}, ${data.max_m[1]}) m. Selected Y ${position[1]} m ${insideY ? "inside" : "OUTSIDE"} sampled Y.`
      : "Outside sampled X/Z; column unobserved.";
    for (const c of rows) {
      const row = document.createElement("tr"), contains = position[1] >= c[1]/2 && position[1] < (c[1]+1)/2;
      row.dataset.kind = c[3];row.dataset.containsPoint = String(contains);
      for (const value of [`[${c[1]/2}, ${(c[1]+1)/2})`, `${c[3]} ${c[4]===null ? "(no ID)" : `ID ${c[4]}`}`, String(c[5]), contains ? "Selected Y" : ""]) {
        const cell = document.createElement("td");cell.textContent = value;row.append(cell);
      }
      body.append(row);
    }
  }
  function describeSelection(text) {
    selectionText = text;
    info.textContent = text;
    if(measure.value!==""){
      const origin=data.pins[Number(measure.value)],delta=selectedPosition.map((v,i)=>v-origin.position[i]);
      info.textContent+=` · from ${origin.label}: ΔXYZ [${delta.map(v=>v.toFixed(3)).join(", ")}] m; distance ${Math.hypot(...delta).toFixed(3)} m`;
    }
  }
  function describePin(index) {
    const pin = data.pins[index];
    selected = index;
    selectedTarget = null;
    const cell = pin.position.map(v => Math.floor(v*2));
    describeColumn(pin.position, pin.label);
    describeSelection(`${index+1}. ${pin.label} · ${pin.kind} · XYZ metres [${pin.position.join(", ")}] · containing cell [${cell.join(", ")}] · ${pin.inside_capture ? "inside capture" : "OUTSIDE capture"}`);
    draw();
  }
  data.pins.forEach((pin,index) => {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = `${index+1}. ${pin.label}${pin.inside_capture ? "" : " (outside)"}`;
    button.addEventListener("click", () => describePin(index));
    document.getElementById("pins").append(button);
    const option=document.createElement("option");option.value=String(index);option.textContent=`${index+1}. ${pin.label}`;measure.append(option);
  });
  function describeTarget(index){
    const target=proposal.targets[index],observed=target.observed;
    selected=null;selectedTarget=index;planTarget.value=String(index);
    describeColumn(target.min_cells.map(v=>v/2), "proposed target");
    describeSelection(`PROPOSED cell [${target.min_cells.join(", ")}] · minimum XYZ metres [${target.min_cells.map(v=>v/2).join(", ")}] · ${target.state}${observed?`; observed ID ${observed.id}, raw fill ${observed.fill}`:"; outside capture, not drawn"} · geometry only; clearance and execution NOT authorized`);
    draw();
  }
  document.getElementById("plan-controls").hidden=!proposal;
  if(proposal){
    proposal.targets.forEach((target,index)=>{const option=document.createElement("option");option.value=String(index);option.textContent=`[${target.min_cells.join(", ")}] ${target.state}`;planTarget.append(option);});
    const counts=proposal.counts;
    document.getElementById("plan-summary").textContent=`Proposed ${proposal.targets.length} cells: ${counts.sampled_air} sampled Air, ${counts.sampled_terrain} terrain, ${counts.sampled_building} building, ${counts.sampled_unknown} unknown, ${counts.unobserved} unobserved (not drawn). X-ray outlines; no clearance, support or build authorization.`;
    showPlan.addEventListener("change",draw);
    planTarget.addEventListener("change",()=>describeTarget(Number(planTarget.value)));
  }
  document.getElementById("provenance").textContent = JSON.stringify({
    bounds_metres: {min:data.min_m,max_exclusive:data.max_m}, cell_size_m: .5,
    counts:data.material_counts, native_session:data.native_session, atomic_snapshot:data.atomic_snapshot,
    capture:data.capture, missing_player_positions:data.missing_player_positions,
    prop_origins:data.prop_origins, plan:proposal?{counts:proposal.counts,source:proposal.source,executable:false,clearance_verified:false}:null, rendered_surface_verified:false
  },null,2);
  function setupSlice() {
    const view=controls.view.value, axis=view==="xy"?2:0;
    document.getElementById("slice-control").hidden = !["xy","zy"].includes(view);
    document.getElementById("yaw-control").hidden = view!=="orbit";
    document.getElementById("pitch-control").hidden = view!=="orbit";
    controls.slice.min = data.min_m[axis]*2;
    controls.slice.max = data.max_m[axis]*2-1;
    controls.slice.value = Math.floor((Number(controls.slice.min)+Number(controls.slice.max))/2);
  }
  function draw() {
    const width=canvas.clientWidth, height=canvas.clientHeight;
    const ratio=window.devicePixelRatio||1;
    canvas.width=Math.round(width*ratio);canvas.height=Math.round(height*ratio);
    ctx.setTransform(ratio,0,0,ratio,0,0);
    ctx.fillStyle="#182332";ctx.fillRect(0,0,width,height);
    const view=controls.view.value, yaw=Number(controls.yaw.value)*Math.PI/180, pitch=Number(controls.pitch.value)*Math.PI/180;
    const pinRows=Math.min(data.pins.length,5),sectionMode=["xy","zy"].includes(view),plotTop=(proposal?110:90)+(sectionMode?20:0),plotBottom=height-90-pinRows*16;
    const raw=p=>project(local(p),view,yaw,pitch);
    const extent=bounds.map(raw), xs=extent.map(p=>p.x), ys=extent.map(p=>p.y);
    const fit=Math.min((width-120)/Math.max(.5,Math.max(...xs)-Math.min(...xs)),(plotBottom-plotTop)/Math.max(.5,Math.max(...ys)-Math.min(...ys)))*Number(controls.zoom.value)/100;
    const screen=p=>{const q=raw(p);return {x:width/2+q.x*fit,y:(plotTop+plotBottom)/2+q.y*fit,depth:q.depth};};
    function path(points) {
      const projected=points.map(screen);ctx.beginPath();
      projected.forEach((p,i)=>i?ctx.lineTo(p.x,p.y):ctx.moveTo(p.x,p.y));ctx.closePath();return projected;
    }
    ctx.strokeStyle="#61718a";ctx.lineWidth=1;ctx.setLineDash([4,4]);
    for(let i=0;i<bounds.length;i++) for(let j=i+1;j<bounds.length;j++) {
      if(bounds[i].filter((v,k)=>v!==bounds[j][k]).length===1){const a=screen(bounds[i]),b=screen(bounds[j]);ctx.beginPath();ctx.moveTo(a.x,a.y);ctx.lineTo(b.x,b.y);ctx.stroke();}
    }
    ctx.setLineDash([]);pick=[];
    let faces;
    if(view==="orbit") faces=exposedFaces(data.cells,yaw,pitch);
    else {
      const cells=view==="top"?topCells(data.cells):section(data.cells,view,Number(controls.slice.value));
      const face=view==="top"?3:view==="xy"?5:1;
      faces=cells.map(c=>({cell:c,face,points:corners[face].map(p=>p.map((v,i)=>(c[i]+v)/2))}));
    }
    faces.sort((a,b)=>a.points.reduce((n,p)=>n+raw(p).depth,0)-b.points.reduce((n,p)=>n+raw(p).depth,0));
    for(const face of faces){
      const points=path(face.points),c=face.cell;
      ctx.fillStyle=colors[c[3]];ctx.globalAlpha=face.face===3?1:.82;ctx.fill();ctx.globalAlpha=1;
      ctx.strokeStyle=c[3]==="air"?"#3e5067":"#1a2737";ctx.lineWidth=.7;ctx.stroke();
      pick.push({points,cell:c});
    }
    let proposedCount=0;
    if(proposal&&showPlan.checked){
      const slice=Number(controls.slice.value),axis=view==="xy"?2:0;
      proposal.targets.forEach((target,index)=>{
        if(target.state==="unobserved"||(["xy","zy"].includes(view)&&target.min_cells[axis]!==slice))return;
        proposedCount++;
        const c=[...target.min_cells,"unknown",null,0];
        const outlines=view==="orbit"?exposedFaces([c],yaw,pitch):[{points:corners[view==="top"?3:view==="xy"?5:1].map(p=>p.map((v,i)=>(c[i]+v)/2))}];
        ctx.strokeStyle=selectedTarget===index?"#ffffff":target.state==="sampled_air"?"#f6d06f":target.state==="sampled_unknown"?"#be97e8":"#ff8994";
        ctx.lineWidth=2;ctx.setLineDash([5,3]);
        for(const face of outlines){const points=path(face.points);ctx.stroke();pick.push({points,plan:index});}
      });
      ctx.setLineDash([]);
    }
    ctx.font="12px system-ui";
    const axes=view==="top"?[0,2]:view==="xy"?[0,1]:view==="zy"?[2,1]:[0,1,2];
    axes.forEach(axis=>{
      const origin=project([0,0,0],view,yaw,pitch),unit=project([0,0,0].map((v,i)=>i===axis?1:0),view,yaw,pitch);
      const a={x:48,y:plotBottom-20},b={x:a.x+(unit.x-origin.x)*35,y:a.y+(unit.y-origin.y)*35};
      ctx.strokeStyle=["#ff9e9a","#9ce6b1","#94bfff"][axis];ctx.fillStyle=ctx.strokeStyle;ctx.lineWidth=2;
      ctx.beginPath();ctx.moveTo(a.x,a.y);ctx.lineTo(b.x,b.y);ctx.stroke();ctx.fillText(`+${"XYZ"[axis]}`,b.x+5,b.y-5);
    });
    const sliceAxis=view==="xy"?2:0,slice=Number(controls.slice.value);
    document.getElementById("slice-value").textContent=`${"XYZ"[sliceAxis]} [${slice/2}, ${(slice+1)/2}) m`;
    data.pins.forEach((pin,index)=>{
      if(!pin.inside_capture)return;
      const p=screen(pin.position),outsideSection=["xy","zy"].includes(view)&&Math.floor(pin.position[sliceAxis]*2)!==slice;
      ctx.globalAlpha=outsideSection?.4:1;ctx.fillStyle=index===selected?"#ffffff":"#f5c66c";
      ctx.beginPath();ctx.arc(p.x,p.y,12,0,2*Math.PI);ctx.fill();ctx.fillStyle="#132034";ctx.textAlign="center";ctx.fillText(String(index+1),p.x,p.y+4);ctx.textAlign="start";ctx.globalAlpha=1;
      if(outsideSection){ctx.fillStyle="#b6c5d7";ctx.fillText("off slice",p.x+16,p.y+4);}
      pick.push({pin:index,x:p.x,y:p.y});
    });
    ctx.fillStyle="#e3eaf3";ctx.font="bold 16px system-ui";ctx.fillText("Native voxel occupancy · "+(view==="orbit"?"3D orbit":view==="top"?"top X/Z":`section ${view.toUpperCase()}`),16,27,width-32);
    ctx.fillStyle="#b6c5d7";ctx.font="12px system-ui";
    function fitText(text,y){ctx.fillText(text,16,y,width-32);}
    fitText("0.5 m cells · y up · raw fill preserved · outside capture unobserved",48);
    fitText(`Capture minimum XYZ [${data.min_m.join(", ")}] m; maximum exclusive [${data.max_m.join(", ")}] m`,66);
    if(proposal)fitText(`PROPOSED ${showPlan.checked?"x-ray":"hidden"}: ${proposal.counts.sampled_air} Air, ${proposal.counts.sampled_terrain} terrain, ${proposal.counts.sampled_building} building, ${proposal.counts.sampled_unknown} unknown, ${proposal.counts.unobserved} unobserved/not drawn; clearance NOT verified`,84);
    if(sectionMode)fitText(`Section ${"XYZ"[sliceAxis]} [${slice/2}, ${(slice+1)/2}) m; off-section pins projected faintly`,proposal?102:84);
    data.pins.slice(0,5).forEach((pin,index)=>fitText(`${index+1}. ${pin.label} · XYZ ≈ [${pin.position.map(v=>v.toFixed(3)).join(", ")}] m${pin.inside_capture?"":" · outside capture"}`,height-55-(pinRows-1-index)*16));
    fitText("Terrain: tan   Building: cyan   Unknown: violet   Pins: gold"+(proposal?"   Proposed: dashed (white selection)":""),height-31);
    fitText("Diagnostic reconstruction; rendered terrain/contact NOT verified",height-12);
    canvas.dataset.renderedFaces=String(faces.length);canvas.dataset.view=view;canvas.dataset.proposedTargets=String(proposedCount);
  }
  function contains(points,x,y){
    let inside=false;
    for(let i=0,j=points.length-1;i<points.length;j=i++){
      const a=points[i],b=points[j];
      if((a.y>y)!==(b.y>y)&&x<(b.x-a.x)*(y-a.y)/(b.y-a.y)+a.x)inside=!inside;
    }return inside;
  }
  canvas.addEventListener("click",event=>{
    const box=canvas.getBoundingClientRect(),x=event.clientX-box.left,y=event.clientY-box.top;
    for(const item of [...pick].reverse()){
      if(item.pin!==undefined){if(Math.hypot(item.x-x,item.y-y)<=14){describePin(item.pin);return;}}
      else if(item.plan!==undefined&&contains(item.points,x,y)){describeTarget(item.plan);return;}
      else if(contains(item.points,x,y)){
        const c=item.cell;if(!c)continue;selected=null;selectedTarget=null;
        describeColumn(c.slice(0,3).map(v=>v/2), "selected cell minimum");
        describeSelection(`Cell [${c.slice(0,3).join(", ")}] · minimum XYZ metres [${c.slice(0,3).map(v=>v/2).join(", ")}] · ${c[3]} ID ${c[4]===null?"unavailable":c[4]} · raw fill ${c[5]} · half-open 0.5 m cube`);
        draw();return;
      }
    }
  });
  for(const element of Object.values(controls))element.addEventListener(element.tagName==="SELECT"?"change":"input",()=>{if(element===controls.view)setupSlice();draw();});
  window.addEventListener("resize",draw);
  measure.addEventListener("change",()=>{if(selectedPosition!==null)describeSelection(selectionText);});
  document.getElementById("export").addEventListener("click",()=>{const a=document.createElement("a");a.download="native-terrain-occupancy.png";a.href=canvas.toDataURL("image/png");a.click();});
  setupSlice();draw();
})();
