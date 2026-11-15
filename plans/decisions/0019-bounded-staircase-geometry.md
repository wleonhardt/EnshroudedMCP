# 0019: Bounded solid staircase geometry

Date: 2026-11-15. Status: implemented; private and sanitized-public source verified; client-dependent acceptance deferred.

The explicit workshop goal includes stairs. Extend the existing building_plan primitive and building_apply_plan primitive/composite compiler with kind staircase, rather than inventing a new job engine or execution route. Keep the existing dimensions, half-metre grid, origin-vertex quarter turns and copied-source material authority.

Use solid stepped geometry ascending local +z. Each rise is exactly one 0.5 m voxel cell; the number of steps equals height in cells. Depth must be at least height and divisible by height in integer grid cells, yielding equal positive tread depths. Width is constant. Cells above each tread are omitted, never erased. This is geometric intent, not a claim about game-native staircase blueprints or avatar climbability.

Preserve 1,024 candidate cells before omission, all world bounds, compact 16 m composite extent, eight parts, 256 unique write cells, eight bricks and 64 cells per brick. Reuse stable local cell keys and canonical kind/dimensions/yaw; input reorder remains harmless while actual request changes conflict. Preserve all write/source/actor/session/rights/persistence/capture/read-back/undo gates. Add no ramp, roof, custom rise or material palette.

Verify an independent stepped-column/count oracle, rotations and complete extents, invalid ratios and workload refusal, unchanged existing shapes, primitive/composite identity and source/brick/write limits. Keep omission reporting distinct from doorway openings. Client hold prohibits live placement, rendering and traversal tests for now; queue tiny healthy dry/apply/read-back/rendered/undo/cold-reload acceptance before any usability claim.
