# 0157: Server lifecycle coverage audit

Date: 2026-11-20. Reviewed source: signed feature e26dd70, server0.51.1 / companion0.22.0, SDK6a151a95/ABI12.1. Status: source coverage complete; potential gaps require behavior reproductions. No runtime behavior change or new native test.

All40 advertised server contracts are accounted for exactly once against the capability index. Classification is manually traced through GameTools::call and delegated modules, not inferred from names or tool descriptions. Mechanical row/count checks prove inventory completeness only; they do not prove the guard classifications or native behavior.

Nine world reads and seven write-capable tools capture an active MCP session at admission and compare an active unchanged epoch before world-dependent execution. Four construction writes instead capture the epoch without an admission active check and enforce active context during execution. Three status tools deliberately preserve restricted stale/unavailable observations. Seven pure/static/local metadata tools do not require a universal current-world gate. Ten dynamic observation paths lack the active admission/execution epoch pattern and need discriminating tests before a fix is proposed.

The MCP epoch comes from published lifecycle events; it is not a native world identifier, connection lock or atomic observation guarantee. The common on_game_thread helper only schedules and waits; it neither checks liveness nor cancels reads after caller timeout. A hosting flag, copied handle, world key or getter success alone does not establish unchanged lifecycle scope.

## Source authority

Trace [GameTools dispatch and scheduling](../../crates/mcp/src/tools.rs), [construction lifecycle and status](../../crates/mcp/src/construction.rs), [reposition observation](../../crates/mcp/src/reposition.rs), [save queue](../../crates/mcp/src/server_save.rs), [shutdown queue/status](../../crates/mcp/src/server_shutdown.rs), [health recovery](../../crates/mcp/src/health_recovery.rs), [grants](../../crates/mcp/src/give_item.rs), [clock control](../../crates/mcp/src/clock_control.rs), [survey](../../crates/mcp/src/building_survey.rs) and [diagnostic capture](../../crates/mcp/src/terrain_image.rs). Full original source/native evidence references remain in [the capability index](../capability-contracts.json); linked historical evidence keeps its original versions and scope.

## World observations with active admission and execution

| Tool | Admission | Execution | Evidence or next check |
|---|---|---|---|
| `entity_buffs_read` | Active MCP session captured in dispatcher. | Active session rechecked and compared before SDK observation. | Decision0143 and corresponding scheduled SDK-fake tests. Native interruption and positive-player correspondence remain separate; see indexed evidence. |
| `factory_station_read` | Active MCP session captured in dispatcher. | Active session rechecked and compared before SDK observation. | Decision0144 and corresponding scheduled SDK-fake tests. Native interruption and positive-player correspondence remain separate; see indexed evidence. |
| `knowledge_read` | Active MCP session captured in dispatcher. | Active session rechecked and compared before SDK observation. | Decision0146 and corresponding scheduled SDK-fake tests. Native interruption and positive-player correspondence remain separate; see indexed evidence. |
| `player_interaction_target` | Active MCP session captured in dispatcher. | Active session rechecked and compared before SDK observation. | Decision0145 and corresponding scheduled SDK-fake tests. Native interruption and positive-player correspondence remain separate; see indexed evidence. |
| `building_access` | Active MCP session captured in dispatcher. | Active session rechecked and compared before SDK observation. | Decision0150 and corresponding scheduled SDK-fake tests. Native interruption and positive-player correspondence remain separate; see indexed evidence. |
| `inventory_read` | Active MCP session captured in dispatcher. | Active session rechecked and compared before SDK observation. | Decision0155 and corresponding scheduled SDK-fake tests. Native interruption and positive-player correspondence remain separate; see indexed evidence. |
| `player_waypoint` | Active MCP session captured in dispatcher. | Active session rechecked and compared before SDK observation. | Decision0142 and corresponding scheduled SDK-fake tests. Native interruption and positive-player correspondence remain separate; see indexed evidence. |
| `player_shroud` | Active MCP session captured in dispatcher. | Active session rechecked and compared before SDK observation. | Decision0140 and corresponding scheduled SDK-fake tests. Native interruption and positive-player correspondence remain separate; see indexed evidence. |
| `player_buffs_read` | Active MCP session captured in dispatcher. | Active session rechecked and compared before SDK observation. | Decision0152 and corresponding scheduled SDK-fake tests. Native interruption and positive-player correspondence remain separate; see indexed evidence. |

## Dynamic observations requiring discriminating lifecycle tests

| Tool | Admission | Execution | Evidence or next check |
|---|---|---|---|
| `weather_read` | Argument bounds only. | Queued weather getter and world-key annotation; no epoch check. | Retain nonzero weather weights across reload/shutdown before queued read. Source inspection only; no new failure or native interruption is proven by this audit. |
| `building_survey` | Bounded survey arguments only. | Hosted-world flag inside read; no admission/execution epoch comparison. | Retain a positive voxel region and zone/right observations across events. Source inspection only; no new failure or native interruption is proven by this audit. |
| `terrain_image` | Bounded diagnostic capture arguments only. | Delegates to building_survey; PNG encoding later uses copies. No independent epoch check. | Test survey/capture refusal before encoding; preserve diagnostic-only limitations. Source inspection only; no new failure or native interruption is proven by this audit. |
| `entity_health` | Bounded point/radius and resolved typed component. | Queued nearby entity discovery and health reads; no epoch check. | Retain positive entity/health state across events, distinguishing absent/read-error. Source inspection only; no new failure or native interruption is proven by this audit. |
| `status` | Hosting flag may return a no-world diagnostic before queueing. | Hosted branch reads save/roster/clock/world key without epoch comparison. | Retain hosted=true and nonempty world data across events; keep no-world diagnostics available. Source inspection only; no new failure or native interruption is proven by this audit. |
| `players` | No active-session admission. | Queued roster, position and vitals reads; no epoch check. | First priority: retain a positive roster across queued reload/shutdown and verify independent SDK data. Source inspection only; no new failure or native interruption is proven by this audit. |
| `voxel_sample` | World bounds and cell conversion. | Queued sample; SDK errors stay errors, but no epoch check. | Retain a known voxel value across events; never turn refused reads into air. Source inspection only; no new failure or native interruption is proven by this audit. |
| `props_in` | Bounded world box. | Queued props/template/label-presence reads; no epoch check. | Retain positive props and distinguish stale scope from SDK error or empty output. Source inspection only; no new failure or native interruption is proven by this audit. |
| `entities_near` | Bounded selector and resolved component type. | Queued discovery and per-entity metadata; no epoch check. | Retain positive spatial results and test all kind modes without implying complete discovery. Source inspection only; no new failure or native interruption is proven by this audit. |
| `entity_components` | Bounds and configured component catalogue availability. | Queued discovery/reflection; no epoch check. | Retain positive configured components; preserve incomplete-catalogue/absence limitations. Source inspection only; no new failure or native interruption is proven by this audit. |

## Write-capable tools with active admission and execution

| Tool | Admission | Execution | Evidence or next check |
|---|---|---|---|
| `player_restore_health` | Active session captured in dispatcher or delegated queue. | Active session and unchanged epoch checked before world-dependent execution. | `health_recovery.rs` and indexed source/native evidence. Actor generation, health preview, cancellation and write gates remain separate. This audit adds no new write or native interruption acceptance. |
| `enemy_pacification` | Active session captured in dispatcher or delegated queue. | Active session and unchanged epoch checked before world-dependent execution. | `pacification.rs` and indexed source/native evidence. Dispatcher guards read/dry/write modes; expected value and write gate remain separate. This audit adds no new write or native interruption acceptance. |
| `give_item` | Active session captured in dispatcher or delegated queue. | Active session and unchanged epoch checked before world-dependent execution. | `give_item.rs` and indexed source/native evidence. Delegated run compares session before retained-operation and native grant logic. This audit adds no new write or native interruption acceptance. |
| `player_reposition` | Active session captured in dispatcher or delegated queue. | Active session and unchanged epoch checked before world-dependent execution. | `reposition.rs` and indexed source/native evidence. Dispatcher compares epoch; delegated actor/generation/preview/write checks follow. This audit adds no new write or native interruption acceptance. |
| `clock_advance` | Active session captured in dispatcher or delegated queue. | Active session and unchanged epoch checked before world-dependent execution. | `clock_control.rs` and indexed source/native evidence. Delegated queue compares session before clock context and once-only write logic. This audit adds no new write or native interruption acceptance. |
| `server_save` | Active session captured in dispatcher or delegated queue. | Active session and unchanged epoch checked before world-dependent execution. | `server_save.rs` and indexed source/native evidence. Delegated queue compares session before current roster checks and save request. This audit adds no new write or native interruption acceptance. |
| `server_shutdown` | Active session captured in dispatcher or delegated queue. | Active session and unchanged epoch checked before world-dependent execution. | `server_shutdown.rs` and indexed source/native evidence. Delegated queue compares session before current roster checks and stop request. This audit adds no new write or native interruption acceptance. |

## Construction writes with a different admission pattern

| Tool | Admission | Execution | Evidence or next check |
|---|---|---|---|
| `building_place_cell` | Dispatcher captures session/epoch; it does not call active_session at admission. | Dispatcher compares queued epoch; construction requires active context before work and checks operation epoch as applicable. | construction.rs and construction_flow tests; indexed historical evidence. Execution protection exists. Do not call this an unguarded write or require a new admission policy without a concrete requirement/reproduction. |
| `building_apply_plan` | Dispatcher captures session/epoch; it does not call active_session at admission. | Dispatcher compares queued epoch; construction requires active context before work and checks operation epoch as applicable. | construction.rs and construction_flow tests; indexed historical evidence. Execution protection exists. Do not call this an unguarded write or require a new admission policy without a concrete requirement/reproduction. |
| `building_accept` | Dispatcher captures session/epoch; it does not call active_session at admission. | Dispatcher compares queued epoch; construction requires active context before work and checks operation epoch as applicable. | construction.rs and construction_flow tests; indexed historical evidence. Execution protection exists. Do not call this an unguarded write or require a new admission policy without a concrete requirement/reproduction. |
| `building_undo_cell` | Dispatcher captures session/epoch; it does not call active_session at admission. | Dispatcher compares queued epoch; construction requires active context before work and checks operation epoch as applicable. | construction.rs and construction_flow tests; indexed historical evidence. Execution protection exists. Do not call this an unguarded write or require a new admission policy without a concrete requirement/reproduction. |

## Operation status with deliberately limited stale outcomes

| Tool | Admission | Execution | Evidence or next check |
|---|---|---|---|
| `building_operation` | No general active admission gate. | Construction status returns inactive/stale unknown before native reads; current operations require matching retained epoch. A verified undone baseline may release local journal recovery credit. | construction.rs::status; indexed decisions0065/0069. Preserve unknown outcomes and retained transaction limitations. |
| `player_reposition_status` | No general active admission gate. | Observer checks retained actor session/generation before current actor reads, then rechecks facts. Prior boot stays unavailable. | reposition.rs::observe and decision0056. Preserve unavailable/stale observations; do not promote retained results into movement authority. |
| `server_shutdown_status` | No active-world requirement. | Listener-side retained state/typed-file existence only; no World getter. | server_shutdown.rs::status and decision0057; decision0156 verifies prior-boot unavailability. It deliberately denies current-world, exit/save and reconstruction claims. |

## Pure planning and non-world metadata

| Tool | Admission | Execution | Evidence or next check |
|---|---|---|---|
| `knowledge_search` | Offline dispatch precedes game-handle access. | Pure curated knowledge/planning; no scheduler or World getter. | tools.rs::offline_call and module behavior tests. Do not add a world gate; planners must remain usable without hosting. |
| `recipe_plan` | Offline dispatch precedes game-handle access. | Pure curated knowledge/planning; no scheduler or World getter. | tools.rs::offline_call and module behavior tests. Do not add a world gate; planners must remain usable without hosting. |
| `building_plan` | Offline dispatch precedes game-handle access. | Pure curated knowledge/planning; no scheduler or World getter. | tools.rs::offline_call and module behavior tests. Do not add a world gate; planners must remain usable without hosting. |
| `construction_budget` | Early local report dispatch. | Journal file metadata and process-local reservation counters; no World getter. | journal.rs::report and budget tests. This is local audit capacity, not current world liveness or write permission. |
| `capabilities` | No active-world requirement. | Copied initialization handles/header and static limits, not World observations. | tools.rs::capabilities; framework policy tests. Availability/header metadata never proves current getter fidelity or rendering. |
| `item_lookup` | Item IDs and resolved registry layout. | Queued published archive resource read; no session-specific actor/geometry selection or epoch gate. | item_lookup.rs and indexed registry fixtures. Treat as build/archive metadata; unavailable resources remain errors, not current-world authority. |
| `item_info` | Vanilla debug-name and resolved inventory API. | Queued static item/placement metadata with world-key annotation; no epoch gate. | tools.rs::item_inspection tests and indexed evidence. World-key annotation is not liveness. Resolve intended metadata availability before imposing a world gate. |

## Priorities and limits

1. Reproduce players first because roster observations feed operator decisions. Then test the hosted branch of status while preserving its explicit no-world diagnostic. Keep independent positive SDK data through queued WorldLoaded/ShutdownStarted events; do not accept an empty fixture as proof.
2. Reproduce the geometry/scene family: survey and terrain capture, voxel sample, props, entities, configured components and typed health. Separate lifecycle refusal from SDK unavailable/error states and preserve existing incomplete/diagnostic limitations.
3. Reproduce nonzero weather weights across events. A metadata world key is not a weather or actor-safety observation.
4. Review static item/archive metadata and specialized status contracts separately. Do not add a blanket world gate to the dispatcher. Check settled decisions before introducing a shared scheduling/guard abstraction; preserve existing write controllers and pure planners.

This audit establishes source coverage, not ten proven bugs or a completed native safety review. No new retained-state reproduction, game action, installation, write gate, provider adoption, client/UI acceptance, native in-flight interruption or final-save proof is added. Decision0156 remains the bounded current native cohort. A later fix needs a failing behavior test, version discipline and the required workspace/publication checks.

Baseline and final458 Rust/117 Python, strict Clippy, formatting, both release builds and offline contracts pass. All crate files remain byte-identical to the reviewed source. Guard classifications remain source inspection and are unverified as new runtime guarantees by this audit; prior behavior tests and native receipts retain only their original applicability.
