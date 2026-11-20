# 0143: Refuse queued buff reads across MCP lifecycle changes

Date: 2026-11-20. Status: independently reproduced and corrected; full source checks pass; native timing remains unverified.

Decision0142 exposed a request crossing an MCP lifecycle epoch while retaining SDK data. Inspect the neighboring server dispatcher for entity and player observations. entity_buffs_read accepts a raw session-only entity and previously called its SDK adapter with no lifecycle admission or execution check. A new actual-dispatcher regression on exact base f332435 first succeeds unchanged, then queues a request, observes the pinned isolated scheduler pending, raises the documented SDK-fake WorldLoaded event and confirms the SDK still supplies one level3 buff for entity17. The request returns that buff instead of refusing, so the test fails independently of a missing component or empty SDK result.

Reuse the existing Construction active lifecycle and game-free policy::construction::check_session before executing this read. Require active admission and active execution with an unchanged epoch, matching player_shroud and player_waypoint. Preserve output shape, nullable levels, duration strings, truncation, SDK errors and thread ownership. Two regressions cover pre-admission shutdown and queued unchanged/reload/shutdown scenarios with retained typed buff data. Synthetic buff setup is fixture preparation through the published fake API; the MCP request only reads. Pinned Scheduler Debug pending is an enqueue barrier, never an execution or native liveness observation.

Bump source server to0.50.2 before compiling changed runtime behavior. SDK6a151a95/ABI12.1, Rust1.99.1, companion0.22.0, tool count and input schema remain unchanged. Main and installed runtime remain separate. Existing historical positive native holder observations do not establish the new lifecycle gate or a countdown/UI correspondence.

## Bounded source inventory

These are dispatcher observations, not a claim of complete runtime acceptance.

| Observation | Selector boundary | Source result / next discriminator |
|---|---|---|
| player_shroud | Exact current player selector | Active admission and queued epoch check; decisions0140/0141 source cases. Native event timing open. |
| player_waypoint | Exact current player selector | Active admission and queued epoch check; decision0142 source regression. Native event timing open. |
| entity_buffs_read | Raw server-session entity ID | This decision independently reproduces and fixes queued lifecycle crossing. Caller must still reacquire IDs after reload. |
| factory_station_read | Raw server-session entity ID | No dispatcher lifecycle check observed at this base. Next reproduce a retained typed station read across a queued lifecycle event; do not infer a positive current station from unknown/not-observed metadata. |
| player_interaction_target | Exact current player selector | Fresh roster lookup but no dispatcher epoch check observed. Next isolate a retained prompt target across a queued lifecycle event. |
| knowledge_read / building_access | Optional current player selector | Queued fresh actor lookup without dispatcher epoch check observed. These need separate contract-specific reproduction before changing behavior. |

This gate detects lifecycle changes during a request, not an old numeric entity ID supplied after reload. No native durable ID, actor generation, connection lock, safety or mutation authority is added. Do not claim all observations are lifecycle-bound from this local fix. Coordinate-based reads and status queries have different contracts and are outside this bounded inventory.

Full checks pass 437 Rust/109 Python tests, strict lint/format, release builds and source contracts. Exact publication audits remain required. Native event delivery/timing, recycled-actor identity and buff countdown/effect/UI correspondence remain unverified. No installation, input, process action or native lifecycle request occurs.
