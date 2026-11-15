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

## Optional companion: seven additional tools

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

## Native world clock

Server 0.21.0 adds `status.clock`: `state` (`observed`/`unavailable`), `source: "published_server_clock"`, nullable `observation`, and `rendered_lighting_verified: false`. Observed fields are `hours`, `is_day`, `day_starts_hours`, `night_starts_hours`, `day_duration_seconds`, `night_duration_seconds`, and `speed`. SDK hours and speed are preserved; SDK durations are expressed in seconds, including fractions. Unavailable observations are null, not midnight or a zero speed. This supplies experiment timing evidence without asserting current rendered lighting or a safe gameplay condition.

The isolated original research namespace now runs server 0.21.0/companion 0.6.1, official acd5550c/ABI 11.30. All nineteen server/nine companion fixed probes pass, including a real daytime clock read; complete world captures and archived ledgers match after a verified cold archive. Native client stays at Menu with zero players and writes disabled. Nonempty prop template/label rows, connected-client rendering and neutral-namespace live installation remain unverified.

## Declared recipe prop/shelter observations (source 0.7.0)

Each returned `client_recipes` row adds `requirement_observations` with `state: "observed"`, `source: "vanilla_archive_recipe_requirements"`, ordered `required_prop_ids`, `requires_shelter`, `full_requirements_verified: false`, and `eligibility_verified: false`. Canonical reflection supplies field offsets, alias targets and array element strides. The parser admits every record/member span, at most 32 prop IDs and only boolean bytes zero/one. Empty lists and false flags are observed values; raw zero/duplicate IDs are not interpreted or deduplicated. Prop type IDs are not inventory item IDs or session entity selectors.

Only returned recipe matches read these fields. Legacy `required_props`, `unlock` and `feasible` remain null: definitions do not prove current station availability, knowledge/progression, full requirements, UI choices or crafting eligibility. Whole upstream archive allocation still precedes the existing byte admission cap.

Source and publication checks pass 184 Rust/31 Python tests, Clippy, formatting and both release builds. Companion 0.7.0 is not installed or live-verified. Isolated runtime remains server 0.21.0/companion 0.6.1; connected-client and neutral-namespace acceptance remain pending. Native lifecycle/input execution remains a supported-API gap.

## Queued inventory grant admission (source 0.21.1)

`give_item` still requires explicit `dry: true` for preview. Real grants check the fresh flag/confirmation before queuing and again on the game thread. Actor resolution and native request submission share that handler. A request that has not started after five seconds is cancelled before a late tick can submit it. Started work may finish after the response timeout; observe inventory and journal before retrying.

A synced `give_item_intent` audit record precedes SDK submission. The native callback syncs `give_item_outcome` independently of the HTTP receiver. Each record is capped at 4,096 bytes; intent admission checks an additional 4,096 bytes of outcome headroom against the shared journal limit. This is a capacity check, not reserved storage. A journal failure can leave a native result unknown; mod shutdown may prevent the callback entirely. Removing the flag does not revoke an already submitted SDK request. The returned `request` identifier correlates audit records only: it provides no replay protection or undo, and a new call can grant again.

Source/public checks pass 187 Rust/31 Python tests, Clippy, formatting and both releases. The revoked-flag regression and fake-host late-callback/cancellation tests pass. This patch is not installed or live-tested. Isolated runtime stays server 0.21.0/companion 0.6.1 with writes disabled; companion source 0.7.0 is also awaiting controlled deployment. Connected-client and neutral-namespace acceptance remain pending.

## Unique grant recipients (source 0.21.2)

`give_item` requires exactly one connected player matching its existing account-key/name rules. Duplicate names and a numeric name colliding with another player's account key refuse before intent journaling or native submission. Roster order never breaks ties; an account selector is not automatically preferred over a matching name. Some ambiguous rosters therefore have no usable selector for a particular player, and the grant stays unavailable rather than choosing a recipient.

The first-match regression and fake-host collision cases pass, together with the prior queued admission, cancellation and callback audit checks. Source/public validation passes 188 Rust/31 Python tests, Clippy, formatting and releases. Server 0.21.2 is not installed or tested with live recipients. Installed runtime remains 0.21.0/0.6.1 with writes disabled; manual connected-client handoff and native lifecycle/input support remain pending.


## Administrative health restoration

`player_restore_health` accepts `player`, optional `dry` (default true), `preview` and `confirm`. Dry observes the sole matching living server actor and returns a preview bound to its session, entity/account, lifecycle generation and current/maximum health. Real restoration uses `dry:false`, that exact preview, a fresh write flag and `confirm:"server-write"`. It restores only to the observed maximum through the published SDK. Full health is a gated no-op; invalid/dead/unavailable actors refuse. Pending timeout cancels only unstarted work.

Intent and independent read-back outcome are synced to the journal. `restored_readback` requires the intended maximum and unchanged actor context; `command_error_unknown` and `readback_unverified` retain uncertainty. Inspect `players.vitals` and journal before any new request after an unknown outcome. No replay protection, damage, revive, native undo, item use or invulnerability is supplied.


Foundation previews stop at occupied terrain cells; they do not certify the rendered surface or physical clearance. A controlled two-cell Air fixture produced only a cap-like fragment against visible rock despite exact native read-back. Treat material IDs, Air spans and placed-prop queries as separate observations; scene registration and collision/support acceptance remain independent.

## Native transient map annotation (companion 0.8.0)

`client_map_markers {}` returns cached submitted state, not a native marker read-back. Supply `markers: [{x, y, z, label}]` for one yellow annotation, or `markers: []` to clear. Coordinates are finite metres within x/z 0..10240 and y 0..1024; labels contain 1..64 UTF-8 bytes, nonblank and no control characters. Unknown fields are refused.

Dry is default. Real calls repeat the exact request with `dry: false`, the returned `preview`, `confirm: "server-write"` and a regular `ALLOW_WRITES` file less than 30 minutes old in the companion's own mod folder. The legacy confirmation grants no server authority. Nonempty annotations require fresh InGame UI; explicit clear also permits a fresh Menu. Preview binds boot, observed InGame epoch, revision and payload. UI transitions are observed context, not an authoritative network-session identity.

Requests execute on the native frame callback. Admission is repeated before submission; synced bounded intent precedes native mutation and correlated outcome follows. Unstarted timeout cancels; started timeout requires inspecting status/journal before another request. Observed departure attempts empty-set cleanup once without requiring the old flag, including when audit storage fails. Failed/uncertain cleanup remains exposed; use a fresh explicitly gated clear rather than assuming an automatic retry.

`native_request_accepted` means SDK submission succeeded. `native_state_readback_available` and `rendered_verified` remain false. The isolated show/clear/show/logout fixture verified submission and eight audit records, with empty cached state and successful cleanup after logout. Map shortcuts failed to open the map, so icon/label appearance and disappearance remain unverified. No fast travel, scene raycast, camera control, saved-marker editing or server-world mutation is provided.

### Rendered map follow-up

A separate isolated fixture opened World/map through the authorized desktop fallback. Native marker show and clear were accepted, but no distinct annotation label/icon was observed at the near-player offset, including after reopening the map. Existing world/player icons remained visible. The small offset may overlap existing icons at this scale; the cause is unresolved. Next use a clearly separated bounded annotation and observed hover before diagnosing the native render path. No fast travel or saved-marker editing was exercised.

Explicit clear preceded logout; research settings were restored and flags removed. Complete baseline captures, operation-ledger membership/hashes and installed binaries remained exact. This adds four companion audit records and four server research-setting records. SDK submission remains verified separately from rendered visibility, which is still unaccepted. Supported native map opening and rendered feedback remain roadmap priorities.

## Site access observation (server 0.23.0; isolated fixture verified)

`building_access` requires `x1, y1, z1, x2, y2, z2`: finite ordered world corners, each side >0 and <=64 metres. An optional `player` name/account key must uniquely match a connected actor; ambiguity/missing matches refuse. Without a player, rights are `not_requested` with null `allowed`. Unknown fields, reversed/degenerate/out-of-world/oversized boxes and invalid selectors refuse before scheduling.

Persistence reports the native zone rule as `kept_by_zone_rule`, or unknown/null on error. Build rights distinguish observed allowed, explicit refused and unchecked unknown/null. Reads run on the game thread, mutate nothing and grant no authority for a later write. The caller supplies the box: no item footprint, collision, contact, live save/reload or general construction feasibility is inferred. Source tests cover bounded admission and false versus unavailable results; deployment/live fixture acceptance remains pending.

## Site access live checkpoint

The isolated research pair now runs server 0.23.0/companion 0.8.0. A verified 47-file cold archive and two rollback copies preceded server-only replacement; six companion/framework/loader hashes remained unchanged. Nineteen server and nine companion fixed probes pass, with incomplete verdicts preserving their semantic limits.

Read-only building_access observed the native persistence rule true for a known fixture box and false for a distant box. Missing actor, unexpected write fields and reversed corners refused. A bounded joined fixture with native full health observed allowed build rights and persistence true for an explicit half-metre box. The separate construction dry preview could not load its source region, so exact preview comparison remains unverified. No construction write or reservation occurred.

After logout and explicit restoration of the research setting, complete prior captures, operation ledgers and installed binary hashes remain exact. The prior server journal prefix has exactly four expected setting records; the companion journal is unchanged. Flags are absent and zero players remain. This validates limited native rule observations, not item footprints, collision/contact, complete base-boundary behavior, save/reload effects or public-namespace installation.

## Loaded construction comparison and selector follow-up

A separate healthy sole-player fixture refreshed source/target voxel reads before the construction dry preview. The preview's exact half-metre box matched building_access's allowed rights and native persistence result. Source material/fill and Air target agreed across the separate reads. No construction mutation, reservation or journal change occurred. Both paths use the same underlying published rules; this agreement is not an independent game-rule oracle or rendered contact proof. Logout and explicit setting restoration preserved all captures, operation ledgers, companion journal and installed binaries.

Source 0.23.1 also corrects a name-selector inconsistency: building_access now uses the pinned SDK's case-insensitive name comparison, preserving exact account-key matching and refusal of multiple candidates. A regression first failed on the old exact-name behavior; case-variant duplicate names and name/account collisions are covered. This patch is not live-installed; runtime remains 0.23.0/0.8.0.

## Site selector live acceptance

The isolated research pair now runs server 0.23.1/companion 0.8.0. A verified 47-file cold archive and two rollback copies preceded server-only replacement. Six companion/framework/loader binaries remained protected; nineteen server and nine companion fixed probes pass with their semantic verdict still incomplete.

A bounded healthy sole-player fixture queried the same explicit box by account key, exact name and changed-case name. All three returned the identical actor, world key, allowed rights and native persistence observation. The read queries changed no journal or operation record. Live duplicate-name/Unicode/reduced-permission rosters were not exercised; offline ambiguity and Unicode regressions remain separate evidence.

After logout and explicit research-setting restoration, complete prior captures, operation-ledger membership/hashes, companion journal and installed binaries remain exact. Only four expected server setting records follow the archive prefix. Flags are absent, pacification is false and zero players remain. Public-namespace installation, scene clearance/contact and functional workshop acceptance remain unverified.

## Foundation stop detail (source 0.23.2)

An admitted `building_survey` foundation preview now includes `terrain_stop: {material, fill}` beside `terrain_stop_cell`. Material uses the same native debug string as voxel_sample; fill is the original byte, including partial terrain. The selected metadata comes from the copied complete capture. Refused spans still return no plan. Air-only geometry, `terrain_overwrite: false` and `rendered_contact_verified: false` remain unchanged.

A daylight observation session found vegetation and rock still obscured confident seam selection. Native reads near the standing actor observed partial terrain with Air above; a complete capture agreed with the separate finite-column oracle. This does not establish a fill-to-height formula, a collision raycast or contact-safe placement. No construction was applied. Settings were restored after logout and all prior captures, operation ledgers, companion journal and binary hashes stayed exact. Source checks pass 205 Rust/32 Python tests, Clippy, formatting and releases; this response increment is not live-installed. Native client scene/look/control gaps remain open.
