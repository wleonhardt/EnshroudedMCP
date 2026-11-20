# 0178: Verify a live read-only terrain capture and diagnostic view

Date: 2026-11-20. Status: native voxel capture and local browser view verified; engine surfaces and positive entity annotations remain unverified.

The standalone viewer at source ec22df4 completes one bounded capture against the existing signed server0.48.0, companion0.22.0 and SDK12.1 installation. The server process creation identity and loopback listener ownership are verified before capture. There is no server restart, installation, client launch, write flag, gameplay command or native connection request.

The selected region comes from an earlier private capture reference. The three capture reads are building_survey, players and props_in, once each. Each native reply is retained before interpretation; refusal would stop the attempt without automatic retry. All 4,096 half-metre cells in the generated scene match the retained survey's coordinates, material IDs/kinds and raw fill. The region contains both terrain and building cells. All 64 caller reference pins retain their labels, order and coordinates.

The roster and returned prop-origin set are empty. Available/shown player positions and prop-origin counts correctly remain zero, without truncation. This does not exercise live automatic-annotation overflow or label collisions; the three failing-then-passing synthetic cases in decision0177 retain that evidence. Caller pins are references, not independently located entities, and none proves ground contact or a safe destination.

BrowserOS opens the newly generated private HTML. Orbit and X/Y section canvases are visually inspected, the last caller pin is selected through its actual button, and the selection displays that reference's coordinates. The owned tab is closed after checking. This is a local reconstruction of native voxel observations, not a game-frame screenshot or engine-mesh validation. Raw captures, coordinates and generated HTML remain private.

Post-capture preservation independently confirms the same primary process, signed installed images, original MCP durable namespace, closed client and disabled write flags. Frozen native Join review0175 and its artifacts remain unchanged and unstarted. Provider adoption, native player/prop correspondence, collision/support, rendering in the game and atomic cross-tool world/session correspondence remain unverified.

Required source checks pass 488 Rust and 128 Python tests, strict Clippy, formatting, release builds and contracts. No product source implementation changes in this evidence increment; the prior three red/green annotation regressions remain intact. Exact publication audits exclude the private native scene and browser artifacts.
