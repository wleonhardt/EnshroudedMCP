# 0070: Read-only technical terrain viewer from native observations

Date: 2026-11-16. Status: source, bounded native capture and browser acceptance passed; engine geometry/contact correspondence remains open.

Build a small standalone diagnostic viewer around existing building_survey, players and props_in tools. Keep the product SDK, installed packages, 49 advertised tools and write admission unchanged. Avoid a new game-side renderer, gameplay executor, browser dependency or generic scene service.

Python validates the complete half-open half-metre box, at most sixteen cells per side and 4,096 total. Exact unique coordinate coverage, material counts, metre/cell bounds, raw fill and finite in-world pin positions must agree. Invalid/loading reads do not become Air or empty success. Capture uses exactly three read tools. Player names/account keys and entity identities are discarded; separate read intervals and unavailable native session remain explicit. Prop origin coverage is bounded/truncated, never collision geometry.

The self-contained browser viewer provides yaw/elevation/zoom, top-down and two vertical sections, raw cell inspection, numbered metre pins, pin-to-pin deltas/distance and PNG export. Requested gameplay, map markers and movement remain outside the viewer. A sampled occupied cell is drawn as a diagnostic cube without inventing terrain interpolation or a fill-to-height equation. Unknown classifications, Air and outside capture remain distinct. Pins off a section are marked; pins outside the capture stay listed.

## Verification

Ten new behavior tests include an asymmetric 2-by-3-by-4-cell fixture with independent axis/projection/face expectations. They preserve precise pin coordinates and raw fill, refuse missing/duplicate/outside cells and contradictory counts/bounds, escape labels as data, preserve exclusive output, restrict live dispatch and expose missing/truncated origins. The actual JavaScript geometry case passes with installed Node; Node absence is an explicit skip, not a renderer pass.

The first native box correctly marks the known overhead platform pin outside its sampled height and cannot establish its geometry. A deliberate one-metre vertical bounds correction captures both ground and platform: 4,096 cells comprise 3,681 Air, 399 terrain and sixteen building cells. All sixteen building cells independently match the prior platform capture in coordinate, material and fill; all three reference/player pins lie inside. Installed hashes, ledgers, write flags and original gameplay setting remain unchanged. No character movement or mutation is requested.

Initial local rendering used an isolated headless browser while BrowserOS tools were unavailable; only the generated viewer was captured. After the user supplies BrowserOS's local MCP endpoint, all remaining interactive verification uses that browser. A real select interaction changes the dropdown while leaving canvas in orbit; the saved reproduction discriminates the event bug. Listen for select change and range input separately. An additional standard EventTarget regression passes production and fails an isolated input-only select mutation. The same BrowserOS interaction then redraws the section, selects the expected half-metre interval, and hides orbit controls. All four views, yaw, exact coordinate measurement, 320-pixel layout without horizontal overflow, console checks and the PNG download button pass. The exported file is independently recognized as PNG and visually inspected. Browser task tab is closed afterward; game/desktop capture stays off.

Current private/public product checks pass 373 Rust executions, 90 Python tests, strict Clippy, formatting, releases and contract-index checks. This helper does not change the runtime ABI or require cold installation. Public examples/tests are synthetic; actual captures, screenshots and raw observations remain private.

## Remaining acceptance

The current helper is a bounded saved snapshot, not live streaming or an in-game overlay. Planned-build overlays, richer surface inspection and native viewer/resource integration can build on this first verified read path. Qualify rendered engine geometry, terrain contact, camera/raycast/depth APIs and usable traversal separately. Do not use a viewer cube or pin as proof of collision, support, reachability or action authorization.
