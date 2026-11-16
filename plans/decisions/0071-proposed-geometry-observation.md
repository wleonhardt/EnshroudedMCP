# 0071: Proposed geometry remains separate from observed terrain

Date: 2026-11-16. Status: bounded source, native-preview and browser acceptance passed; engine surface/contact and execution remain unverified.

Extend the saved technical viewer with an optional building_plan geometry result. Accept only geometry_preview_only, executable false, the half-metre grid, one to 256 unique complete in-world integer cells and an agreeing solid count. Explicit malformed plan files refuse before constructing a live client or writing output. Neither a supplied file nor its status field authenticates a native receipt or grants execution permission.

For each target, retain its minimum cell coordinate and independently look up the captured raw material/fill. Distinguish sampled Air, terrain, building, unknown and unobserved; missing coverage never becomes Air. The original sampled cells are unchanged. Outside-capture targets remain inspectable in the list but are not drawn. Proposed dashed x-ray outlines are drawn after observed cells, so visual occlusion cannot be interpreted as physical clearance. Gold marks sampled Air, coral terrain/building, violet unknown and white selection. Top view projects target footprints and can overlay different heights on the same pixels; sections retain their exact half-metre plane.

The visibility toggle affects proposed geometry only. Target inspection reports raw observations or explicit missing coverage, never build rights, support, collision, survival eligibility or authorization. Live capture still calls exactly the same three read tools. Plan compilation is a separately observed pure native preview, not a new viewer dispatch path or mutation.

## Verification and corrections

Four new Python behavior cases cover all five target classifications with immutable samples, discriminating invalid/duplicate/outside/over-limit/count/grid refusals, invalid-plan preflight before any live client/output and offline CLI packaging. Existing standard EventTarget checks now run both without and with a proposal, verify toggle/selection behavior and require the exact section interval in canvas export. JavaScript failures now expose assertion stderr instead of an opaque subprocess command.

The native composite preview agrees with seven independently enumerated targets. In a complete isolated capture, five are sampled Air, one terrain and one outside the half-open capture. Every one of the 4,096 observed cells still matches the previous capture, including the sixteen platform cells. Primary installation, ledger, write-flag and gameplay-setting hashes remain exact. No character movement, construction or gameplay write is requested.

BrowserOS verifies six in-capture orbit targets, hide/show, the explicit unobserved target and five frame cells on the requested Z section. The exported PNG is downloaded, recognized and visually inspected. The first section PNG lacks the plane coordinate; a failing canvas-text regression reproduces that omission. The fix prints the exact half-open section interval and sampled/proposed counts. A 320-pixel layout remains within viewport width; game/desktop capture stays off.

The first event-mutation attempt recorded under decision 0070 was invalid: its isolated copy omitted investigate.py, causing ModuleNotFoundError before the event test. Its nonzero exit did not prove mutation discrimination. The original browser reproduction and passing production tests remain valid. A corrected complete-copy baseline passes, input-only select mutation fails the exact orbit-versus-xy Node assertion, unobserved-as-Air mutation fails the expected classification, and omitted-section-caption mutation fails the intended export assertion. Each audit rejects import failures and checks the specific expected failure. Preserve this correction instead of rewriting history or treating any red test as the intended red test.

Private/public checks pass 373 Rust executions, 94 Python tests, strict Clippy, formatting, releases, JavaScript syntax and current contract-index checks. SDK, runtime versions, 49 tool schemas and installed binaries remain unchanged. Actual captures and images stay private; public tests are synthetic.

Native viewer/resource integration, engine surface/camera/raycast/depth correspondence, contact, collision, functional construction, faults and cold recovery remain open. Do not infer them from proposed outlines or sampled Air.
