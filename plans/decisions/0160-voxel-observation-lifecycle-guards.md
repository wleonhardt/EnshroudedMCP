# 0160: Voxel observation lifecycle guards

Date: 2026-11-20. Status: source fix; native qualification pending.

Six independent SDK-fake reproductions confirm three decision0157 candidates. Queued building_survey, terrain_image capture and voxel_sample return retained positive terrain after reload, and each also answers after shutdown before admission. A direct published SDK read independently retains Terrain(9)/fill255 in one half-metre cell before and after events. Hosting stays true, demonstrating that the existing survey hosted-world flag is not unchanged MCP lifecycle scope.

Source0.51.3 reuses the established game-free active admission/execution epoch policy at the three dispatch sites, before native reads. Keep scheduling, timeout behavior, hosted checks, bounds, output projection, raw material/fill and chunk-loading caveats unchanged. This is an MCP scope boundary, not an atomic native snapshot, native world identity or connection lock. No shared scheduling abstraction or client path changes.

Six cases cover each tool across queued unchanged/reload/shutdown and pre-admission shutdown. Direct positive SDK reads prevent empty/error data from masking missing guards. Refused calls must add no voxel read. The unchanged image case captures positive terrain/reference pin, raises shutdown, then still encodes a diagnostic PNG from the captured copy with no further SDK voxel reads. Rendering metadata continues to deny rendered-game, surface-height, clearance and world-mutation claims. This is copied diagnostic output, not a live client image or current-world proof.

Earlier0.51.2 native evidence does not qualify the new boundary. Native interruption, physical support/collision/traversal and rendered correspondence remain open. The historical audit now has five remaining candidate paths requiring separate discriminators: weather, typed health, props, nearby entities and configured components. Their guard absence alone does not prove a bug.

No installed file, provider, client, actor position, configuration, credential, save file or runtime log is changed/read. No Windows MCP or game capture occurs. Fake voxels use the published fixture API; no game memory or unsupported event injection is used.

Baseline463 Rust/117 Python passes before edits. Six separate reproductions fail on successful retained observations; unchanged copied-image presentation also passes before the fix. All six cases pass after guards, including refusal read counters and copied presentation after shutdown. Final469 Rust/117 Python, strict Clippy, formatting, both releases and offline contracts pass. Exact publication audits remain required.
