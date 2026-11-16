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
  globalThis.TerrainGeometry = {project, section, topCells, exposedFaces};
  if (typeof document === "undefined") return;

  const data = JSON.parse(document.getElementById("terrain-data").textContent);
  const canvas = document.getElementById("terrain"), ctx = canvas.getContext("2d");
  const controls = Object.fromEntries(["view","yaw","pitch","zoom","slice"].map(k => [k,document.getElementById(k)]));
  const info = document.getElementById("selection");
  const measure = document.getElementById("measure");
  const colors = {air: "#253347", terrain: "#b5986b", building: "#63b7d0", unknown: "#be97e8"};
  let pick = [], selected = null;
  const center = data.min_m.map((v,i) => (v + data.max_m[i])/2);
  const local = p => p.map((v,i) => v-center[i]);
  const bounds = [];
  for (let x=0;x<2;x++) for(let y=0;y<2;y++) for(let z=0;z<2;z++) {
    bounds.push([x?data.max_m[0]:data.min_m[0],y?data.max_m[1]:data.min_m[1],z?data.max_m[2]:data.min_m[2]]);
  }
  function describePin(index) {
    const pin = data.pins[index];
    selected = index;
    const cell = pin.position.map(v => Math.floor(v*2));
    info.textContent = `${index+1}. ${pin.label} · ${pin.kind} · XYZ metres [${pin.position.join(", ")}] · containing cell [${cell.join(", ")}] · ${pin.inside_capture ? "inside capture" : "OUTSIDE capture"}`;
    if(measure.value!==""){
      const origin=data.pins[Number(measure.value)],delta=pin.position.map((v,i)=>v-origin.position[i]);
      info.textContent+=` · from ${origin.label}: ΔXYZ [${delta.map(v=>v.toFixed(3)).join(", ")}] m; distance ${Math.hypot(...delta).toFixed(3)} m`;
    }
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
  document.getElementById("provenance").textContent = JSON.stringify({
    bounds_metres: {min:data.min_m,max_exclusive:data.max_m}, cell_size_m: .5,
    counts:data.material_counts, native_session:data.native_session, atomic_snapshot:data.atomic_snapshot,
    capture:data.capture, missing_player_positions:data.missing_player_positions,
    prop_origins:data.prop_origins, rendered_surface_verified:false
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
    const pinRows=Math.min(data.pins.length,5),plotTop=90,plotBottom=height-90-pinRows*16;
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
    data.pins.slice(0,5).forEach((pin,index)=>fitText(`${index+1}. ${pin.label} · XYZ ≈ [${pin.position.map(v=>v.toFixed(3)).join(", ")}] m${pin.inside_capture?"":" · outside capture"}`,height-55-(pinRows-1-index)*16));
    fitText("Terrain: tan   Building: cyan   Unknown: violet   Pins: gold",height-31);
    fitText("Diagnostic reconstruction; rendered terrain/contact NOT verified",height-12);
    canvas.dataset.renderedFaces=String(faces.length);canvas.dataset.view=view;
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
      else if(contains(item.points,x,y)){
        const c=item.cell;selected=null;
        info.textContent=`Cell [${c.slice(0,3).join(", ")}] · minimum XYZ metres [${c.slice(0,3).map(v=>v/2).join(", ")}] · ${c[3]} ID ${c[4]===null?"unavailable":c[4]} · raw fill ${c[5]} · half-open 0.5 m cube`;
        draw();return;
      }
    }
  });
  for(const element of Object.values(controls))element.addEventListener(element.tagName==="SELECT"?"change":"input",()=>{if(element===controls.view)setupSlice();draw();});
  window.addEventListener("resize",draw);
  measure.addEventListener("change",()=>{if(selected!==null)describePin(selected);});
  document.getElementById("export").addEventListener("click",()=>{const a=document.createElement("a");a.download="native-terrain-occupancy.png";a.href=canvas.toDataURL("image/png");a.click();});
  setupSlice();draw();
})();
