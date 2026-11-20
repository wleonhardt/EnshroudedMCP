# 0138: Export a projected coordinate ruler with terrain diagnostics

Date: 2026-11-20. Status: source and diagnostic-browser correspondence verified; required product checks pass.

The terrain inspector already reports coordinate deltas and distance in the HTML selection text, but the exported geometry canvas does not retain that measurement. Draw a cyan coordinate ruler from the chosen reference pin to the selected point and put horizontal X/Z run, signed Y difference and 3D distance in the canvas caption. Add dashed horizontal/vertical legs, keeping the existing numeric selection text and raw-column export unchanged.

Only draw ruler geometry when both endpoints are inside every half-open capture bound. An outside endpoint retains coordinate arithmetic but explicitly has no drawn ruler. Section projections are faint when either endpoint is off section and the caption marks that projection. Changing the reference or choosing None redraws the canvas so a PNG cannot retain an old ruler. The source capture, pins and raw bands remain unchanged. No surface, route, collision, ground-contact or arrival inference is introduced.

An independently specified event regression fails before implementation. Nineteen terrain methods then pass. The new case verifies positive and negative Y, exact horizontal/3D dimensions, projected guide geometry, current PNG drawing commands, off-section opacity and outside/None suppression through standard form/select events. No new library, endpoint, schema or mod version is needed; this is a local viewer feature.

A fresh isolated-server capture returns 4096 cells using exactly building_survey, players and props_in, preserving the original runtime and known installed state. Four real BrowserOS form/PNG cases are independently calculated and visually inspected: initial measurement, larger positive height, negative height and outside capture. The export shows the ruler and correct signed dimensions; outside capture removes both crosshair and ruler with an explicit caption. One owned diagnostic tab closes afterward. This is saved diagnostic rendering, not game capture or proof of the game terrain surface. No client input, installed-file change, process action or lifecycle retry occurs.

The source cohort passes 424 Rust executions, 108 Python tests, strict Clippy, formatting, release builds and source contracts. SDK/compiler pins and mod versions remain unchanged. Native surface/contact and actual client acceptance remain unverified; the browser evidence applies only to the saved diagnostic viewer.
