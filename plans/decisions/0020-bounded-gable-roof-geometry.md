# 0020: Bounded solid gable-roof geometry

Date: 2026-11-15. Status: implemented; private and sanitized-public source verified; live acceptance deferred.

The workshop goal explicitly requires a roof section. Reuse the existing building_plan and building_apply_plan primitive/composite path with kind gable_roof. Choose symmetric solid stepped gables, not a hollow shell or a game-native roof blueprint. This creates a filled crown above the caller's roof origin; occupied attic space is explicit. Shelter/weather classification, appearance and structural support are unverified.

The ridge runs along local z. Height is at least 1 m. In half-metre cells, width must be a whole multiple of twice height, giving equal positive horizontal runs on both slopes. Each paired column rises from one cell at either eave to height cells at the central two runs. Depth extrudes this profile. The two central runs form a flat ridge; no arbitrary pitch, overhang, attic or material palette is added. Existing origin-vertex yaw and complete bounding-box checks apply. Omitted above-slope cells never erase occupants.

Preserve 1,024 candidate cells before omission, 256 unique writes, eight bricks, 64 cells per brick, eight parts and compact 16 m extent. Reuse source-voxel material/fill, stable element keys, canonical kind/dimensions/yaw, overlap deduplication, write admission, session/actor/rights/persistence/capture/read-back and guarded undo. No write or scheduler implementation changes.

An independent paired-column set oracle and closed-form count must cover multiple heights/runs/depths. Check rotations and occupied extents, invalid slopes/door fields/workloads/world edges, preview omission metadata, primitive/union identity, source overlap and brick/write limits. The tiny 3 m wide, 1.5 m high, 1 m deep example has 24 solids in 36 candidates, a 1 m ridge and a filled gable. Live placement, rendered slope/ridge/clearance, shelter semantics, undo and cold reload remain queued until the user releases the client.
