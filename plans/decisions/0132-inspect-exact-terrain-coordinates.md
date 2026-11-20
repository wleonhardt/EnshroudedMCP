# 0132: Inspect exact coordinates in the local terrain viewer

Date: 2026-11-20. Status: source and browser verification pass against a fresh native capture; required private/public checks pass.

The technical terrain viewer already exposes raw half-metre cells, saved coordinate pins, selected columns and section jumps. Inspecting an arbitrary precise coordinate previously required making a new input fixture or selecting a nearby cell minimum. Add a local XYZ form so an operator can inspect that exact point without another game read or a gameplay request.

Accept finite numeric world coordinates within the existing world limits, preserving their precision without snapping or clamping. Invalid, empty or out-of-world input preserves the previous selection. Valid input selects the existing raw column, supports exact containing X/Y and Z/Y sections and reuses distance and JSON export behavior. Selecting a pin, cell or proposed target also updates the coordinate fields. The source capture and its pins remain unchanged.

Draw a white selected-point crosshair over the diagnostic geometry, with a precise coordinate caption included in PNG exports. Off-section points project faintly and are explicitly marked. Points outside any half-open capture axis have no crosshair and cannot open a containing section. An observed X/Z column can still be shown for a point above or below the captured Y range; every selected-band marker then remains false. Outside X/Z stays unobserved rather than becoming known Air. This is diagnostic occupancy, not engine mesh, ground-contact, collision, traversal or placement proof.

An independently specified coordinate test fails before implementation. Eighteen terrain methods then pass, covering finite bounds, precision, nonmutation, actual standard form events, invalid selection preservation, exact section indices, outside capture and JSON downloads. Existing plans, pins, measurements and renderer cases remain covered. The source baseline is414 Rust/105 Python; one new test method brings the current Python cohort to106.

A fresh4096-cell isolated-server capture uses exactly the existing building_survey, players and props_in read tools. Three actual BrowserOS form submissions and JSON downloads independently match32 native bands with selected-band counts1/0/0. Fractional precision survives real number inputs and exported JSON. Both exact section jumps succeed. The saved PNG is visually inspected and contains the selected-point crosshair and precise coordinate caption. One owned browser tab closes; no game input, game capture, provider installation, game-process control or extra export read occurs.

The original primary identity, empty roster, setting, durable namespace and known installed state remain checked. Pending restoration and native getter qualification remain separate. Server0.48.0, companion0.22.0, SDK/ABI12.1 and Rust pins do not change; this increment changes the standalone diagnostic viewer only.

Both source cohorts pass414 Rust/106 Python checks, strict lint/format, release builds and live server metadata contracts. The exact eight-file publication diff and316-file managed snapshot pass identifier/secret audits; private capture data and browser downloads are excluded.
