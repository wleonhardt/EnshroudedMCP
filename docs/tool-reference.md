# Tool reference

The server advertises 23 tools. Inspect `tools/list` for exact schemas on your installed build: unknown fields are refused. Spatial coordinates are metres with y up unless a response explicitly reports integer half-metre voxel cells. Entity IDs and inventory slot references are session-only.

## Server reads and pure planners

| Tool | Inputs and result |
| --- | --- |
| `status` | No arguments. Hosting state, save observations and player count. Save timestamps do not prove a requested checkpoint completed. |
| `capabilities` | No arguments. Resolved APIs, versions, limits and verification caveats. Availability is not live acceptance. |
| `players` | No arguments. Connected player name, account key, slot, admin flag, position and native vitals. Null is unavailable/absent, not zero; UI unit conversion is unverified. |
| `inventory_read` | Exactly one `player` or `entity`; optional `offset` and `limit` (1–128). Linked inventory slots and available per-item metadata. Pages are independent snapshots; no slots can mean absent or uninitialized inventory. |
| `item_info` | `item` debug name. Vanilla identity and runtime static metadata, including `placement` states `observed`, `known_no_template` or `unavailable`. Mapping does not prove placement eligibility. |
| `item_lookup` | `ids`: 1–8 unique nonzero IDs. Vanilla GUID/debug-name matches. Collisions and unmatched IDs remain explicit. |
| `entity_health` | Point, optional `radius` (up to 16) and `limit` (up to 20). Typed health on nearby placed entities, with absent/error states. It is not a complete combat scan. |
| `voxel_sample` | `x`, `y`, `z`. Exact material/fill at the containing half-metre cell. Building fill is not a validated UI shape parameter. |
| `props_in` | Two box corners `x1/y1/z1`, `x2/y2/z2`; sides at most 64 m. Placed-item props with item, transform, nullable template GUID and nullable label presence, at most 500 results. GUIDs use lowercase file-order hex. Label absence is not a safe-undo claim; exact label text and other held state are unknown. This is not complete collision discovery. |
| `entities_near` | Point, optional `radius` (up to 32), `kind`. At most 200 nearest results. `containers` selects placed containers, not every death-loot object. |
| `entity_components` | Point, optional `radius` (up to 16) and `limit` (up to 20). Names of locally configured reflected components; no arbitrary component-value dump. |
| `knowledge_search` | Optional `query`, `status`, `limit`. Curated evidence, provenance and missing checks. |
| `recipe_plan` | Supplied recipe catalogue, stock, goals, stations and unlocks. Pure dependency arithmetic, not a live recipe or survival-feasibility claim. |
| `building_plan` | Primitive `floor`, `wall` or `doorway`, dimensions, material label and optional quarter-turn `yaw`. At most 1024 candidate cells. Pure preview; material labels are unvalidated. |
| `building_survey` | Grid-aligned box dimensions and optional `player` and `foundation`. Foundation supplies grid-aligned `x`, `z`, exclusive `upper_y` in metres and integer `max_cells` from 1 to 15; the column must fit the survey. It previews only Air before occupied terrain, without a rendered surface-height claim or write authority. Up to 4096 complete cells, exact voxels, persistence and optional build rights. Props, hazards and traversal remain unchecked. |
| `building_operation` | `operation`. Reconcile native construction status and exact complete-brick images. Stale/unknown receipts never reconstruct native IDs. |

## Synthetic preview example

Call `building_plan` with these arguments to preview a doorway. The coordinates and material label are synthetic. This tool performs no game reads or writes and does not validate material availability.

```json
{
  "kind": "doorway",
  "x": 100,
  "y": 100,
  "z": 100,
  "width": 2,
  "height": 2.5,
  "depth": 0.5,
  "material": "synthetic:unvalidated",
  "door_offset": 0.5,
  "door_width": 1,
  "door_height": 2
}
```

## Administrative tools

Every real write requires the fresh flag and confirmation described in [architecture](architecture.md). Prefer dry preview and deliberate operator review. There is no batch that automatically chains these tools.

| Tool | Contract |
| --- | --- |
| `give_item` | Player, vanilla item debug name, count up to 100. Inventory transaction, journaled. **Dry is opt-in:** explicitly use `dry: true` to preview. It is not a construction cost/refund system or a once-only construction operation. |
| `building_place_cell` | Operation, source point, target point and player. Copies one exact observed building voxel into Air. Dry by default; real placement requires its preview `session`. |
| `building_apply_plan` | Operation, player, source point and `plan`. Primitive or composite union of 1–8 uniquely named primitive parts. At most 256 unique cells, eight bricks, 64 cells per brick, 16 m per axis and 1024 total candidate cells. Source voxel supplies all material/fill. Dry by default. |
| `building_accept` | Operation. Dry by default. Syncs acceptance after exact applied verification, permits another bounded operation and retains guarded session-only native undo. Does not prove persistence. |
| `building_undo_cell` | Operation. Handles a retained cell or plan transaction. Dry by default. Checks original actor, current rights/persistence and complete expected images. Later overlapping work must be undone first. |
| `player_reposition` | Operation, player and requested point. Dry by default; real request requires exact `preview` token. Sole connected living actor, maximum 16 m requested displacement, engine free-spot search. Same identity never dispatches twice. No arrival receipt, native undo, walking or camera control. |

Composite parts use absolute positions and quarter-turn yaw. Reordering parts normalizes to the same request; renaming a part changes identity. Overlaps are written once and owned by the first lexical part ID. Openings omit cells and can be filled by another part's contribution. No nested compositions or per-part material source is accepted.

## Optional companion: six additional tools

These are on the separate client endpoint, outside the server's count of 22.

| Tool | Result |
| --- | --- |
| `client_capabilities` | Companion APIs, limits and explicit unsupported actions. |
| `client_state` | Freshness-tagged native screen, prompt, host position/template and observed held controls. |
| `client_events` | Bounded screen/host change history, sequence cursor and gaps. |
| `client_selection_events` | Named client-world HUD/recipe-event observations. They do not establish a current passive selection. |
| `client_item` | Vanilla identity and available localized item name. |
| `client_recipes` | Bounded raw vanilla recipe observations. Effective settings and unresolved requirements remain unknown. |

Client IDs are never server selectors. Screen enum names need not expose tabs: a crafting tab may still report its Backpack parent. No tool opens vanilla menus, synthesizes gameplay input, crafts an item or drives the camera.

### Isolated research setting

`enemy_pacification` with no arguments observes the published boolean. An `enabled` proposal defaults to dry. A real change additionally requires exact `expected`, current `session`, `dry: false`, the fresh write flag and `confirm: "server-write"`. It rechecks admission on the game thread, durably journals intent/outcome and changes only pacify_all_enemies. It is not invulnerability, has no native undo or durable once identity, and an unknown result requires fresh observation rather than blind retry. Configuration saves may persist the setting; explicitly restore and read back the prior boolean. The gameplay preset may become Custom; no complete preset-restoration claim is made.

Companion `client_recipes` now preserves raw stacks and adds typed per-input item/category GUIDs, IDs and counts, decoded only for admitted returned matches through the canonical reflected type hash. Category membership, catalysts, unlocks, current UI selection and feasibility remain unknown.

Companion source 0.5.2 fixes selection-event cursor gap reporting after session-clear. Cursors older than cleared observations report a gap before any new event. Overwrite counts retain their existing meaning. Isolated research-namespace installation passes fixed native probes; actual join/logout loss reporting remains unverified.

## Companion 0.6.1 recipe-category observations

`client_recipes` now adds `category_membership` to each returned `input_observations` row. State is `observed`, `not_found`, or `no_category_id`; unresolved states have `members: null`. Observed categories contain ordered `members` with `item_id`, nullable `scaling`, `scaling_finite`, and exact `scaling_bits`, plus `total_members`, `truncated`, and `scaling_semantics_verified: false`. Member entries are archive observations; no unique-member, effective-cost, eligibility or UI-selection claim is made. Non-finite scaling remains null in JSON with its bits preserved.

The parser admits a 16 MiB registry, at most 4,096 category rows, 1,024 member records per category and reflected strides from 1 to 4,096 bytes. Every category header/member span is admitted before indexing; duplicate category IDs refuse. Only the first 32 members of categories referenced by returned recipe inputs are decoded. Whole upstream registry allocation still precedes admission. All array/field offsets derive from published reflection, including typedef resolution; no game offsets or new hooks are used.

Original research-namespace companion 0.6.1 is installed isolated-only; server remains 0.19.0 and official framework acd5550c/ABI 11.30 is unchanged. Nine companion and eighteen installed-version server probes pass. A Workbench category returns six members whose item IDs independently resolve through the server vanilla item registry. This is limited static membership evidence; scaling interpretation, displayed choices and full crafting feasibility remain unverified. Both complete world captures and archived ledgers match; native client stays at Menu, zero players and writes disabled. Public namespace binaries are not live-installed.
