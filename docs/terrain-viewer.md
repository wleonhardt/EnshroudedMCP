# Native terrain inspector

The inspector draws diagnostic geometry from bounded native voxel observations. It runs outside the game and adds no MCP gameplay executor. Python 3.11 or later and a browser are sufficient; generated HTML has no external scripts, libraries or network requests. Node is optional for the renderer's source-test cases.

## Capture and open

Choose the minimum X/Y/Z coordinates and width/height/depth in metres. Values must lie on the half-metre grid. Each side is at most 8 metres and the complete box is at most 4,096 cells. For an existing loopback endpoint:

```powershell
python scripts/terrain_view.py --port 8751 --box 10 20 30 8 8 8 --output terrain.html
```

These are example coordinates; select a useful box in your world. The script calls only building_survey, players and props_in, once each. It does not start the server, read credentials, enable writes, move the character or place map markers. A loading/refusal error stops capture instead of drawing empty space. Native reads can initiate chunk loading; retry deliberately after inspecting the failure. The output path must not exist.

Open the generated file in a browser. Choose 3D orbit, top X/Z, section X/Y or section Z/Y. Yaw, elevation and zoom change only this view. Section slices represent half-open half-metre intervals. Select a cell for minimum-corner coordinates, material ID and raw fill. Numbered pins retain exact metre positions; Measure from reports coordinate deltas and straight-line distance between selected pins. These distances are geometric measurements, not route lengths.

Save PNG exports the drawn canvas, including bounds, axis directions, material legend and the first five numbered pin coordinates. All pins remain available in the viewer. Files contain local world observations; review them before sharing. Account names/keys and entity identities are discarded from live pin data. Player pins are numbered in capture order and must not be reused as persistent selectors. Prop pins identify origins only; no footprint or collision box is inferred.

## Vertical column profile

Select a numbered pin, a drawn cell or a proposed target to inspect every sampled half-metre Y band at that point's containing X/Z column. Bands are listed highest first, with material ID, raw fill and a highlight only where the selected Y lies in the half-open band. The complete captured column remains visible regardless of the current projection or section slice.

Outside the sampled X/Z box means an unobserved column. A point outside sampled Y can still show the known column, with no selected band. No sample is extrapolated and no extra MCP request is made. Partial/zero fill, Air and unknown materials remain raw data; the profile proves no surface height, support, contact or clearance. Save PNG still exports only the main geometry canvas; the profile stays in the HTML. Previously saved HTML remains unchanged: generate a new output file to include this feature.

Source behavior and fresh native-data DOM correspondence pass. Browser visual/layout acceptance for this new profile remains pending a disconnected BrowserOS link; see decision 0095. Earlier browser acceptance below covers the older projection/pin/proposal features.

## Offline input and reference pins

The offline JSON shape is an object containing survey (the complete decoded building_survey result) and optional pins. Each pin has a unique printable label of at most 64 characters and a three-number position in metres. There are at most 64 pins including generated player/prop pins; at most 32 prop origins are shown and truncation is explicit. Optional --pins adds a JSON list of reference pins to either input path.

```powershell
python scripts/terrain_view.py --input captured-survey.json --output terrain-offline.html
```

For example, a reference pin is `{"label":"Inspection point","position":[10.25,21,30.75]}`. Labels are encoded as data, never page markup. Duplicate labels, missing/duplicate/outside cells, contradictory counts/bounds, unknown material classifications and malformed coordinates refuse. The source cell order may differ; complete coordinate coverage is required.

## Proposed-build overlay

Pass --plan with the decoded geometry-only building_plan result, not a construction receipt or raw primitive request:

```powershell
python scripts/terrain_view.py --input captured-survey.json --plan geometry-preview.json --output proposed.html
```

The preview must have status geometry_preview_only, executable false, grid.cell_size_m 0.5, geometry.solid_cell_count equal to the length of cell_minima, and one to 256 unique in-world integer cell coordinates. An offline input object can alternatively supply plan; supplying it both ways refuses. Invalid explicit --plan files refuse before live capture. This is supplied geometry data, not authenticated execution authority.

Show proposed cells toggles dashed x-ray outlines while leaving observed samples intact. Gold targets overlap sampled Air, coral sampled terrain/building, and violet unknown material; selected outlines are white. Inspect target reports exact minimum coordinates and raw sampled material/fill. Unobserved targets remain listed but are not drawn. Neither Air nor a projected outline proves clearance, support, collision, reachability or write authorization. Top view projects footprints, so different planned heights may share pixels; use sections for vertical inspection.

PNG exports include proposal counts and exact half-open section coordinates. Proposals do not initiate new tool calls: live capture retains its three read tools and the preview is supplied separately. The installed game packages, SDK, runtime tools and write gates do not change.

## Evidence limits

The drawing preserves raw material classification and fill. It does not reproduce the engine's interpolated terrain mesh, textures, vegetation, collision, water, props or actual camera. Unknown materials stay visibly distinct. Air cells are sampled Air, not proof of physical clearance. Outside the sampled box is unobserved. Top view shows the highest classified non-Air cell in each sampled column, which can conceal caves or lower structures; use sections to inspect them. Off-section pins are marked and outside-capture pins remain listed.

Survey/player/prop reads have separate UTC intervals and are not an atomic snapshot. The current read contract exposes no native world-session identifier; the viewer reports unavailable. Offline input does not acquire capture/session evidence merely by being rendered. Save/reload persistence, terrain contact and avatar safety require separate checks.

Fifteen behavior tests cover an asymmetric coordinate fixture, precise pins/raw fill, complete captures, strict input values, data-safe labels, exclusive output, read-only dispatch, coverage limits and actual JavaScript projection/section/face behavior. Proposal cases also preserve sampled/unknown coverage and reject malformed geometry before live access. The JavaScript cases use Node when installed and otherwise report explicit skips. A connected isolated-server fixture separately matches all sixteen known platform cells. BrowserOS verifies view/slice changes, exact pin measurements, narrow layout and PNG download; a select-change reproduction was fixed before acceptance. Neither observation establishes a faithful engine terrain mesh. See decisions 0070 and 0071. The original isolated mutation attempt omitted a required import and is not intended-failure evidence; corrected complete-copy checks discriminate the exact event, missing-coverage and section-caption assertions.

## Direct MCP image integration status

The current server 0.47.0 exposes terrain_image and returns metadata text plus a diagnostic PNG directly through MCP, without opening this external viewer. Current native output is observed in decision 0095; the earlier bounded image/caption acceptance is decision 0076. Neither qualifies the new column's browser layout or actual engine terrain rendering.

Use view top with no slice, view xy with a Z slice minimum, or view zy with an X slice minimum. Captions and metadata preserve the distinction between sampled classification and engine mesh, camera, contact or clearance. See decision 0073 for complete argument bounds and caller-reference pin semantics. The external inspector remains a separate saved snapshot with its own column/proposal controls; no game-frame capture or new native executor is added.

## Direct MCP pin columns

Installed server 0.48.0 adds `column` to every `terrain_image` metadata pin. `xz_min_cells` gives its containing X/Z cell; `state` is `observed` or `unobserved`. `bands` lists every sampled half-metre Y interval highest first, with `min_cells`, `y_min_m`, `y_max_exclusive_m`, raw `kind`, `id`, `fill` and half-open `contains_pin_y`. Columns remain complete in section views, even for an off-section pin.

Outside sampled X/Z returns no bands. Above/below sampled Y still returns the known column without a selected band. At most sixteen pins each contain sixteen bands. These are caller references and copied samples, not entity selectors, surface height, support, clearance or ground contact. No extra world read occurs. PNG drawing and request schema are unchanged. Matched 0.48.0 empty-server endpoint acceptance now verifies 720 bands and unchanged diagnostic pixels in decision 0099. Connected-client consumption and engine contact remain separate. Decision 0098 preserves the original source/native-data replay checkpoint.
