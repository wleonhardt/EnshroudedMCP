# 0026: Explicit planned-volume clearance checks

Date: 2026-11-15. Status: implemented; private and sanitized-public source verified; installed runtime unchanged.

The rendered floor/stair/roof fixture has only 1.5 m, 1.0 m and 0.5 m vertical air between successive treads and the roof. A matching solid-cell capture does not establish usable headroom. Extend the existing pure building_plan preview with optional clearance volumes rather than create another executor or infer avatar collision dimensions.

The caller supplies one to eight absolute axis-aligned boxes using x, y, z, width, height and depth in metres on the half-metre grid. Each extent is positive and at most 16 m; the full half-open boxes must remain within world bounds and total at most 1,024 requested cells, counting overlapping requests separately. Nested objects refuse unknown/missing fields and malformed, off-grid or excessive values in game-free policy.

Intersect checked volumes with the existing compiled solid cells after quarter-turn/compound union. Report each requested extent, cell count, exact unique conflict cell minima and planned_clear. Touching half-open boundaries do not collide. Results explicitly say planned_cells_only, world_observed false and walkability_verified false. A clear result cannot establish empty terrain, props, support, shelter, avatar clearance or current live state. Empty requests refuse; omission preserves existing responses. The construction schemas and write authority remain unchanged.

Verify an independent cramped stair/roof intersection, boundary-only clearance, rotated primitive, strict nested admission, total workload limits, bounds and unchanged absent responses. Qualify a new matched runtime separately before calling this live accepted. The next workshop must include explicit requested headroom, an upper landing and grounded support plus actual healthy native traversal and complete recovery.
