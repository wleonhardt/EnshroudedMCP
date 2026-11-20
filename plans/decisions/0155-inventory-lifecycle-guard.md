# 0155: Inventory lifecycle admission and execution

Date: 2026-11-20. Status: source fix; native qualification pending.

Inventory observations feed item planning and crafting evidence. The old dispatcher queues inventory_read without active lifecycle admission or an execution epoch comparison. A deterministic SDK-fake fixture independently retains a current player, linked inventory and seven items after lifecycle events. Entity and player-selector queue reproductions return old slots/counts after WorldLoaded changes the epoch; a pre-admission shutdown reproduction also returns an observation. These are behavior failures, not unavailable SDK getters.

Source0.51.1 reuses the existing game-free active lifecycle/session policy at admission and at queued execution, before roster selection or slot/count reads. No new lifecycle model or native identifier is introduced. A matching epoch is an MCP scope boundary, not a connection lock or native world identity. Preserve input schema, output projection, paging, alias handling, unknown inventory completeness and unknown crafting eligibility.

Three dispatcher/SDK-fake cases exercise entity and player selectors across unchanged/reload/shutdown queue barriers and both shutdown admission modes. The pinned SDK scheduler Debug value is only an enqueue barrier for the isolated queue. The published FakeHost inventory preset supplies synthetic component layout; no game memory is read. Independent direct SDK roster/slot reads prove positive retained state before and after events. Successful unchanged responses retain the seven-item count and unknown eligibility. Refusals must come from lifecycle policy before observation.

Earlier0.51.0 native evidence does not qualify this changed boundary. Native lifecycle interruption, positive player inventory/count correspondence and rendered UI remain open. No installed files, provider, client process, configuration, credentials, saves or runtime logs are changed/read by this increment. No Windows MCP or capture is used.

Baseline455 Rust/117 Python passes before changes. All three independent red reproductions fail on successful old observations; all three pass after the guard. Final458 Rust/117 Python, strict Clippy, formatting, both release builds and offline contracts pass. Publication requires exact source diff and complete sanitized snapshot/history audits.
