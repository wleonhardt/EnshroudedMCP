# Tool reference

The server advertises 32 tools. Inspect `tools/list` for exact schemas on your installed build: unknown fields are refused. Spatial coordinates are metres with y up unless a response explicitly reports integer half-metre voxel cells. Entity IDs and inventory slot references are session-only.

## Server reads and pure planners

For `recipe_plan`, optional `choices` contains up to 512 entries shaped as `{"item":"board","recipe":"recipe-id"}`. Every output with multiple producers needs one matching choice. A sole producer is implicit. All catalogue entries remain validated, but only selected dependencies contribute costs, requirements or cycle detection. No automatic alternative search or optimization is performed.

| Tool | Inputs and result |
| --- | --- |
| `status` | No arguments. Hosting state, save observations and player count. Save timestamps do not prove a requested checkpoint completed. |
| `capabilities` | No arguments. Resolved APIs, versions, limits and verification caveats. Availability is not live acceptance. |
| `players` | No arguments. Connected player name, account key, slot, admin flag, position and native vitals. Null is unavailable/absent, not zero; UI unit conversion is unverified. |
| `weather_read` | One bounded x/y/z server weather sample on the game thread; preserve raw weights, unavailable state and guarded SDK dominance. No forecast mutation or rendered/shelter/safety proof. |
| `factory_station_read` | One nonzero `entity` from this server session. Optional typed FactoryStation reader; game-thread observation of running/next recipe IDs, up to four category choices each, signed raw times, stop/use-all flags and SDK missing-requirement names. Unknown differs from not observed; not every station has this component. No complete queue, UI selection, ETA conversion, eligibility, crafting or safe-removal proof. SDK decoding precedes output bounds. |
| `inventory_read` | Exactly one `player` or `entity`; optional `offset`, `limit` (1–128) and `item_ids` (one to eight unique nonzero IDs). Linked inventory slots and available per-item metadata. Pages are independent snapshots; no slots can mean absent or uninitialized inventory. Optional `item_counts` deduplicates physical slots over the same SDK result independently of page, refusing conflicting aliases or more than 65,536 returned records. Inventory completeness and crafting eligibility remain unknown, including zero counts; SDK traversal is not bounded by the post-read cap. |
| `item_info` | `item` debug name. Vanilla identity and runtime static metadata, including `placement` states `observed`, `known_no_template` or `unavailable`. Mapping does not prove placement eligibility. |
| `item_lookup` | `ids`: 1–8 unique nonzero IDs. Vanilla GUID/debug-name matches. Collisions and unmatched IDs remain explicit. |
| `entity_health` | Point, optional `radius` (up to 16) and `limit` (up to 20). Typed health on nearby placed entities, with absent/error states. It is not a complete combat scan. |
| `voxel_sample` | `x`, `y`, `z`. Exact material/fill at the containing half-metre cell. Building fill is not a validated UI shape parameter. |
| `props_in` | Two box corners `x1/y1/z1`, `x2/y2/z2`; sides at most 64 m. Placed-item props with item, transform, nullable template GUID and nullable label presence, at most 500 results. GUIDs use lowercase file-order hex. Label absence is not a safe-undo claim; exact label text and other held state are unknown. This is not complete collision discovery. |
| `entities_near` | Point, optional `radius` (up to 32), `kind`. At most 200 nearest results. `containers` selects placed containers, not every death-loot object. |
| `entity_components` | Point, optional `radius` (up to 16) and `limit` (up to 20). Names of locally configured reflected components; no arbitrary component-value dump. |
| `knowledge_search` | Optional `query`, `status`, `limit`. Curated evidence, provenance and missing checks. |
| `recipe_plan` | Supplied recipe catalogue, stock, goals, stations and unlocks. Recipes use legacy item/count or one to 16 outputs; selected outputs share batches. Coproducts assigned elsewhere stay surplus. Optional explicit item/recipe choices select among alternatives. Ambiguous or mismatched choices refuse. Pure dependency arithmetic, without automatic optimization or a live recipe/survival-feasibility claim. |
| `construction_budget` | No arguments. Current mod journal bytes and session-local recovery reservations; read-only observation, never native storage or write authority. |
| `building_plan` | Primitive dimensions/material/yaw, or `kind: "composite"`, one material label and named parts. Optional absolute `clearance` boxes report planned-cell conflicts only. At most 1024 candidates; compounds also keep construction cell/brick limits. Pure preview, never live material or execution authority. |
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
| `give_item` | Required caller `operation`, `player`, `item`; optional count 1..100. Dry by default; real call needs exact preview and write gate. Same-payload dry lookup reconciles pending/native callback status without redispatch. Native commit is not inventory read-back; no undo. |
| `building_place_cell` | Operation, source point, target point and player. Copies one exact observed building voxel into Air. Dry by default; real placement requires its preview `session`. |
| `building_apply_plan` | Operation, player, source point and `plan`. Primitive or composite union of 1–8 uniquely named primitive parts. At most 256 unique cells, eight bricks, 64 cells per brick, 16 m per axis and 1024 total candidate cells. Source voxel supplies all material/fill. Dry by default. |
| `building_accept` | Operation. Dry by default. Syncs acceptance after exact applied verification, permits another bounded operation and retains guarded session-only native undo. Does not prove persistence. |
| `building_undo_cell` | Operation. Handles a retained cell or plan transaction. Dry by default. Checks original actor, current rights/persistence and complete expected images. Later overlapping work must be undone first. |
| `player_reposition` | Operation, player and requested point. Dry by default; real request requires exact `preview` token. Sole connected living actor, maximum 16 m requested displacement, engine free-spot search. Same identity never dispatches twice. No arrival receipt, native undo, walking or camera control. |
| `server_save` | Operation identity; dry by default. Real calls require exact preview session, empty SDK character roster and write gate. Published native request only; retained replay never redispatches. Save completion and causal persisted contents remain unverified. |

Composite parts use absolute positions and quarter-turn yaw. Reordering parts normalizes to the same request; renaming a part changes identity. Overlaps are written once and owned by the first lexical part ID. Openings omit cells and can be filled by another part's contribution. No nested compositions or per-part material source is accepted.

## Optional companion: eleven additional tools

These are on the separate client endpoint, outside the server's count of 30.

| Tool | Result |
| --- | --- |
| `client_capabilities` | Companion APIs, limits and explicit unsupported actions. |
| `client_command` | Caller operation plus supported action payload; dry default, exact native context preview and companion write gate. Once-only SDK submission; no server authority. |
| `client_commands` | Read sorted retained outcomes and durable caller identities, at most 128 records; no dispatch or native ID reconstruction. Invalid reservation entries refuse. |
| `client_command_status` | Read retained caller operation outcome with last frame status-poll age/staleness and polling-finished state; no dispatch or ID reconstruction. |
| `client_command_cancel` | Separate dry preview and gated once-only cancellation of a retained native operation. |
| `client_snapshot` | Copied SDK state, age/session and per-part read flags; actual menu tab, local client identity and recipe highlight. Null with unread part means unavailable. No command or World access. |
| `client_map_markers` | At most one transient mod-owned annotation; dry by default. Exact preview and client write gate required for real submission. Cached submission is not rendered proof. |
| `client_state` | Freshness-tagged native screen, prompt, host position/template and observed held controls. |
| `client_events` | Bounded screen/host change history, sequence cursor and gaps. |
| `client_selection_events` | Named client-world HUD/recipe-event observations. They do not establish a current passive selection. |
| `client_item` | Vanilla identity and available localized item name. |
| `client_workshops` | Raw `workshop_id` (zero through u32 maximum), optional `limit` (1..8, default 4). Vanilla definition matches with workshop GUID, raw name tag, declared station item ID/GUID and NPC-workshop flag. Zero, collisions and truncation remain explicit. Current availability and eligibility are unknown. |
| `client_recipes` | Bounded raw vanilla recipe observations, including stack item IDs/GUIDs and workshop definition ID/GUID. Effective settings and unresolved requirements remain unknown. |

Client IDs are never server selectors. The existing frame screen and named event history do not prove current tabs. The new client_snapshot reports the native inner tab separately when its read flag is available, with explicit native freshness. Companion 0.15.0 now opens supported in-game screens and submits bounded native move/look. No join, credentials, logout, arbitrary input or crafting executor is exposed. Native outcome is not rendered proof.

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

## Historical queued inventory grant admission (source 0.21.1)

This historical contract applied through 0.32.1. Server 0.33.0 requires caller identities and defaults to dry; use the replay-safe grant contract for current requests.

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

### player_interaction_target

Read-only arguments: `{ "player": "connected name or account key" }`. Name matching follows the SDK; ambiguous or absent actors refuse. The response separates an observed server prompt target, no target, and unknown/unavailable reader state. Target entity IDs last one server session; position/template may be null. This tool does not interact, raycast terrain, establish contact, or map client IDs to server IDs. All native reads run on the scheduled server game thread.

### Prop discovery scope

`props_in` reports item-placed props whose origins lie in the requested box. The published SDK deliberately excludes flame altars, item-placed creatures and survivors; entity queries can observe those separately. It also requires the placed-template mapping to match. An empty prop list cannot establish an empty rendered volume, absence of an altar, collision clearance or terrain contact. An isolated altar fixture agrees across prompt target, entity/used-item queries and item placed-template metadata while correctly yielding no props.

### Client interaction prompt text

Companion 0.9.0 `client_state.observation.prompt` preserves `verb` and adds nullable `verb_text`, `verb_text_state` (`observed` or `unavailable`), `verb_text_truncated`, `language` and `verb_text_source`. Title and verb text are bounded to 256 UTF-8 bytes. The published localization reader uses the requested client language with English fallback; missing remains null and an observed empty string remains distinct. This is copied frame evidence, subject to the outer age/stale fields, and does not execute the interaction or prove rendered text.

Server 0.24.1 adds `observation.target.item_id` to `player_interaction_target`. This is the exact target's native UsedItem value; null means absent/unavailable and zero remains a raw observation. A nonzero ID can be examined with `item_lookup`; collisions, effective definitions and placement eligibility remain unresolved by this field. `template_guid_encoding` identifies the existing file-order hexadecimal bytes, which differ from the companion SDK's formatted GUID string.

### knowledge_read

Read-only arguments: `{ "ids": [1, 2], "player": "optional connected name or account key" }`. Supply one to eight unique nonzero u32 IDs. Omit player for world queries; supplied player must match exactly one connected actor. Responses preserve order and report `query`, selected actor, and rows with native `type`, `scope`, `value`, `expected_flags` and observed/unknown state. Unknown rows have null metadata; unknown can mean an unavailable read, unknown ID or wrong scope. Observed zero is retained. Player queries can return shared world values; inspect row scope. Native IDs need an independently qualified source. No grants/revokes, completion changes, recipe eligibility or effective-mod-definition claim is made.

Companion 0.10.0 `client_recipes` rows include `recipe_id`, the raw u32 game ID from the published archive record field. Zero remains an observed record value; it is not an admissible `knowledge_read` ID. The value is not calculated from its GUID/name. A qualified nonzero ID can be queried through server `knowledge_read` for a uniquely selected actor, preserving type/scope and unknown state. Archive identity, seen/crafted flags, current recipe selection and crafting eligibility remain separate observations; effective modified definitions are unverified.

## Deliberate forward clock control

Server 0.27.0 adds `clock_advance`. Immediate read-back may retain the old hour even after native acceptance. The retained receipt is not promoted by a later independent observation; read operation status and `status.clock` separately, without redispatching. A bounded isolated fixture observed this behavior.

Supply `operation` (1..64 lowercase ASCII letters, digits, underscores or hyphens) and `target_minute` (0..1439). Preview defaults to `dry: true`. A real request also requires `session`, recently observed `expected_hours`, `dry: false`, a fresh `ALLOW_WRITES` flag and `confirm: "server-write"`. Only an empty published SDK player-character roster, paused/normal speed and a target one to twelve game hours forward are admitted. The roster is not a connection lock; coordinate client access separately.

An operation-only request reads retained status. Never reuse an identity to retry an uncertain outcome: same-request replay does not dispatch, changed requests are refused, and durable identities remain blocked across restarts. At most 128 identities are admitted. Native acceptance and near-target read-back do not prove elapsed progression or rendered lighting. Returning to an earlier hour advances another cycle and is not undo. No speed control is exposed.

## Recipe lookup by archive ID

Companion 0.12.0 extends `client_recipes` to accept exactly one `item` output debug name or `recipe_id` integer from 0 to 4294967295. Optional `limit` remains 1..8, default 4. Existing output-name responses are unchanged. ID responses contain `selected_recipe_id` and `id_ambiguous`; they preserve every bounded matching row, total match count and truncation instead of choosing an arbitrary collision.

This connects an observed native recipe ID to raw vanilla archive metadata. It does not establish current UI selection, effective definitions, eligibility or crafting execution. Zero remains a valid observed ID. The isolated 0.27.0/0.12.0 pair passes a matched cold deployment and live comparison: two observed archive IDs return complete unchanged output-query metadata at Menu and during a healthy connected session. Zero/max-ID queries and invalid selectors are exercised. Clean logout, restored settings, complete captures, archived ledgers and eight installed binary hashes pass. Live duplicate/zero/max-ID records, actual selected-UI applicability and public-namespace installation remain unverified.

## Native dedicated-server shutdown

Server 0.28.0 adds `server_shutdown` through the published native stop request. Supply `operation` (1..64 lowercase ASCII letters, digits, underscores or hyphens). New identities preview by default; real calls require `dry: false`, the preview `session`, a fresh server-folder `ALLOW_WRITES` flag and `confirm: "server-write"`. An empty SDK player-character roster is checked again after durable intent. It is not a connection lock and omits loading clients; coordinate access separately.

The request runs on the game thread and queued pre-start timeouts cancel execution. Exclusive durable identities, synced intent/outcome journaling, at most 64 identities and one native attempt per active session prevent blind redispatch. A dry request for an existing identity returns retained status; after restart its durable identity stays unknown and cannot be reissued. Do not remove identities to retry. Native errors and lost replies also require reconciliation.

`stop_request_accepted` reports native acceptance only. `process_exit_verified` and `final_save_verified` remain false: observe the exact process and verify save/restart/read-back separately. The published route is dedicated-server-only and may be unavailable; preview does not prove support. No automatic restart, countdown, save hold, process kill or client logout is exposed. Live shutdown acceptance is pending; installed research runtime remains 0.27.0/0.12.0.

## Shutdown lifecycle observations

Server 0.28.1 optionally subscribes to published shutdown-start/save callbacks. A request reports `lifecycle_event_sources` for shutdown, save and world-generation hooks, and separate `lifecycle_observations`. At most one shutdown-start and one later save event are journaled for an attempted operation; early, duplicate and other-generation events are ignored. Journal failures stay explicit. The observer uses no World API and never retries shutdown.

Four-record headroom is checked before intent. These passive observations do not promote the original request receipt: `process_exit_verified`, `final_save_verified` and causal completion remain unverified. Live event order and save/restart acceptance are pending. Installed research runtime remains 0.27.0/0.12.0; deployment and client acceptance are deferred.

## Solid staircase geometry

Server 0.29.0 adds `kind: "staircase"` to `building_plan` and existing `building_apply_plan` primitive/composite geometry. Width stays constant; steps ascend local +z and rotate through the existing origin-vertex quarter turns. Each rise is one 0.5 m cell. Step count is height divided by 0.5 m; depth must be at least height and a whole multiple of height, producing equal positive grid-aligned treads. For example, width 1 m, height 1.5 m and depth 3 m yields three rises, 1 m treads and 24 solid cells.

Pure previews report step/rise/tread metadata, `walkability_verified: false` and `omitted_above_steps_cells`; those voids are not doorway openings. Omission never erases existing occupants. Existing floor/wall/doorway responses are unchanged. This is solid voxel geometry, not a native stair blueprint or a climbability claim.

All limits remain: 1,024 candidates before omission, 256 unique write cells, eight bricks, 64 cells per brick, eight composite parts and 16 m compact extent per axis. Material/fill still comes only from the observed source voxel. Every write/session/actor/rights/persistence/capture/read-back/undo gate remains in force. Independent offline column/count/rotation and construction-admission tests pass; live rendering, traversal, placement, undo/reload and workshop usability remain deferred while the client is unavailable.

## Solid gable-roof geometry

Server 0.30.0 adds `kind: "gable_roof"` to pure `building_plan` and existing `building_apply_plan` primitive/composite compilation. The ridge runs along local z before the existing origin-vertex quarter turn. Height must be at least 1 m; width must be a whole multiple of twice height on the half-metre grid. Equal paired column runs rise from one cell at either eave to the full height at the central two runs. Depth extrudes this symmetric profile. The gable is solid and fills its attic volume; it is not a hollow or game-native roof blueprint.

For width 3 m, height 1.5 m and depth 1 m, occupied column heights are 0.5, 1, 1.5, 1.5, 1 and 0.5 m. This gives 24 solid cells in 36 candidates and a 1 m flat ridge. Preview metadata exposes `attic_filled: true`, `shelter_verified: false`, the half-metre rise/run, local ridge axis and `omitted_outside_gable_cells`. Those omissions are separate from doorway openings and never erase existing occupants.

All candidate/write/brick/composite/source and recovery limits remain unchanged. Material comes from the observed source voxel; existing write gates and native transaction handling apply. Independent offline cell/count/rotation/admission tests do not prove shelter, support, appearance or workshop usability. Deployment, placement, rendered slope/ridge/clearance, undo and cold-reload acceptance remain deferred while the client is unavailable.

## Native server save requests

Server 0.31.0 adds `server_save` through published `World::save`, independently of shutdown. Supply `operation` (1..64 lowercase ASCII letters, digits, underscores or hyphens). New identities preview by default. Real calls require `dry: false`, the exact preview `session`, a fresh server-folder `ALLOW_WRITES` flag and `confirm: "server-write"`. The empty SDK player-character roster is rechecked after durable intent. This omits loading peers and is not a connection lock; coordinate access separately.

Requests run on the game thread. A five-second queued timeout cancels before execution; a started timeout is unknown and must be reconciled using the same identity without automatic retry. Exclusive synced `server-save/save-{operation}.json` files retain at most 64 identities, with bounded intent/outcome journal headroom. Existing in-memory identities return retained same-session status. After reopening, a durable identity reports unknown on dry lookup and cannot dispatch again. Stale sessions, changed gates and audit failures refuse dispatch; native errors never authorize retry. A distinct identity is a separately deliberate request, not proof that a prior save completed. Preserve the namespace in cold archives and never remove identities to bypass capacity.

`save_dispatched` and `save_request_accepted` describe the native request only. `save_completed_verified` and `causal_completion_verified` remain false; the API treats an already-running save as asked, and holds may refuse it. A timestamp or uncorrelated `WorldSaved` event does not prove the exact requested world state reached disk. No backup, save hold, forced save, native undo, shutdown or client control is added. Use dry lookup for retained status; real replay still requires the write gate. Live save/restart/full-content acceptance remains deferred while client use is reserved.

## Pure compound layout previews

Server 0.32.0 extends `building_plan` with `kind: "composite"`, one top-level unvalidated `material` label, and one to eight uniquely named `parts`. Parts use existing absolute floor/wall/doorway/staircase/gable-roof dimensions and quarter turns, with `id` added and no material field. Nested composites and mixed/write fields are refused. Existing primitive inputs and responses remain unchanged.

The shared construction compiler preserves 1,024 candidates before omission, 16 m compact full extents, eight parts, 256 unique cells, eight bricks and 64 cells per brick. Output contains `canonical_plan`, full `footprint_m`, unique `cell_minima` and separate candidate, contributed-solid, omission and overlap counts. Candidate, contribution and omission counts are summed per part before union; overlap counts repeated solid contributions, not unique physical voids. Omitted voids do not subtract another part's solids or erase occupants. Lexical part IDs keep overlapping ownership and input reorder deterministic.

For example, a 2 m square floor at (4,4,4), a 1 m wide/1 m high/2 m deep staircase at the same origin, and a 2 m wide/1 m high/1 m deep solid gable at (4,5,4) produce 32 unique cells from 48 candidates, 40 solid contributions, eight omissions and eight overlap contributions. These are geometric test dimensions, not a measured functional workshop.

This path needs no game reads, player, source, write flag or session and always reports `executable: false` and `material_validated: false`. Its material label cannot authorize construction. Use the separate construction dry path with an observed source, current actor/session and all live gates before any write. Collision, rights, persistence, support, shelter, traversal, appearance, functional props and recovery remain unverified. Keep complete matched live acceptance deferred while the client is unavailable.

## Strict legacy grant inputs

Server 0.32.1 refuses supplied `give_item` counts unless they are integers from 1 to 100. Only an omitted count defaults to one; negative, zero, fractional, string, boolean, null and excessive counts never fall back to a one-item grant. Player and item selectors must contain 1..128 UTF-8 bytes, cannot be whitespace-only and cannot contain control characters. Admission runs before game-thread lookup or queueing, including direct tool-handler calls.

In 0.32.1, valid grant behavior and the legacy real-by-default contract were preserved. Server 0.33.0 replaces that contract with required caller identity, dry default and preview binding as described below. Real grants require the fresh server write flag and confirmation, with checks before queueing and again before intent/native submission. Pending timeout cancellation and durable callback auditing remain. Audit IDs generated internally are not caller-supplied idempotency keys: lost outcomes still require independent journal/inventory inspection without blind retry. The replay-safe migration below adds caller identity/session binding; independent native inventory read-back and live acceptance remain unverified.

## Replay-safe inventory grants

Server 0.33.0 intentionally changes `give_item`: `operation`, `player` and `item` are required, `count` is optional/default one (1..100), and `dry` now defaults true. Legacy calls without an identity fail closed. Operation IDs use 1..64 lowercase ASCII letters, digits, underscores or hyphens. A new dry request returns `preview` and `would_give`, binding the active world session and observed actor/item to the requested identity/payload. Preview resolves identity only; it does not prove inventory space, effective modded definitions or native acceptance. Real calls require `dry: false`, that exact current `preview`, fresh server `ALLOW_WRITES` and `confirm: "server-write"`.

The request queues on the game thread, syncs exclusive `give-operations/grant-{operation}.json` intent, rechecks actor/item/session and final flag/deadline, then submits once to the native inventory API. At most 64 identities and bounded intent/outcome journal headroom are admitted. A queued five-second timeout cancels; a started timeout remains unknown. Return state may be `submitted_outcome_pending`, with `given: null`; the native callback separately retains and journals completion after HTTP reply loss or flag removal.

Repeat the original operation/player/item/count with `dry: true` to inspect retained same-session status. It never redispatches. Changed payload conflicts. Real replay still requires write admission and preview syntax; it only returns retained status. Reopened durable identities are unknown and cannot dispatch; old in-memory sessions refuse. Keep this namespace in cold archives and never remove identities to bypass capacity. A new ID is a new deliberate grant, not reconciliation or a hidden retry.

`native_outcome_reported` and `given` report native commit/rollback, not independent inventory contents. `inventory_readback_verified` remains false. Late callbacks remain scoped to their original session; audit failures stay explicit in cache. There is no native undo, inventory-loss cleanup, automatic retry or survival accounting. Live inventory baseline/read-back, matched deployment and connected acceptance remain deferred while the client is unavailable.

## Guarded native client commands

Companion 0.15.0 exposes `client_command`, `client_command_status` and `client_command_cancel`. All command requests require caller `operation` and one `action`. Dry is the default. Use `open_screen`/`choose_tab` with a whitelisted `target`; `close_screen` has no payload; `look` requires `yaw` and `pitch` radians (at most a quarter turn each, before sensitivity); `move` requires `dx`, `dy`, `hold_ms` (nonzero direction, length at most one, 1..250 ms); `highlight_recipe` requires a nonzero `recipe_id` and highlights without crafting. Fields belonging to another action are refused.

Read `client_snapshot` first. A dry command binds boot, operation, normalized payload, native session, HUD and observed tab in `preview`. Real submission requires `dry: false`, that exact preview, a fresh companion-folder `ALLOW_WRITES` and `confirm: "server-write"`. Admission runs on a client frame, then the SDK queues execution in its later client phase. SDK session/HUD checks and player-input cancellation protect execution; the admission-time tab observation is not an atomic native tab guard. Movement/look require InGame. Tab changes need an observed menu; recipe highlighting needs the observed PlayerCrafting tab. No main-menu join, credentials, logout, arbitrary input or server World access is exposed.

At most 64 exclusive files in `client-commands/` reserve identities durably before dispatch. Same-payload replay returns retained status; conflicting payloads and prior-boot IDs cannot dispatch. Native IDs stay in memory and are never reconstructed. Inspect `client_command_status` by operation after any lost reply. Never invent another operation ID to retry an uncertain action. Preserve the companion journal and complete client-commands directory in cold backups.

Cancellation has its own dry preview and the same client write gate. It targets only a retained native identity and attempts cancellation at most once. Native accepted cancellation is not completion: poll status separately. Pending/applied/observed/cancelled/unavailable/unknown remain distinct from rendered evidence. Observed movement means the hold ended; observed look means input was written. `rendered_verified` and `arrival_verified` remain false. Source tests do not prove a live effect.

## Explicit planned-volume clearance

Server 0.35.0 accepts optional `clearance` in pure `building_plan`: one to eight absolute axis-aligned boxes with `x`, `y`, `z`, `width`, `height`, `depth`. All values use the 0.5 m grid. Positive extents are at most 16 m; full boxes must fit world bounds. Their combined volume is at most 1,024 cells, counting repeated/overlapping requests separately. Unknown nested fields and empty lists refuse.

For example, add `"clearance":[{"x":4,"y":5.5,"z":5,"width":1,"height":2,"depth":1}]` to a plan to request an empty 2 m high volume. Boxes use world axes independently of each primitive's yaw. Each result reports requested minimum/maximum-exclusive bounds, cell count, `planned_clear`, `conflicting_cell_count` and exact unique `conflicting_cell_minima`. Boundaries that only touch are clear. A box outside the plan footprint may be planned-clear because no planned cell intersects it.

This intersects compiled plan cells only. `scope: "planned_cells_only"`, `world_observed: false` and `walkability_verified: false` remain explicit. It does not inspect terrain, props, collision, support, shelter or avatar dimensions. Omission preserves the previous response. Construction inputs do not accept clearance or gain authority from this result. Matched cold installation and isolated server transport now pass; companion relaunch/connected checks of the rebuilt bundle remain unverified.

## Retained construction recovery budget

Server 0.36.0 reserves bounded future undo audit bytes for each new native construction request. Accepting the operation releases placement exclusivity while preserving that reservation. All mod journal append paths protect aggregate retained credit. Undo admission checks its full audit allowance again; only the operation's own durably synced recovery records consume its credit. Exact full-baseline verification or native world lifecycle invalidation releases unused space. Dry undo budget checks change no reservation. Unknown outcomes remain retained and native requests are never retried automatically.

The 8 MiB journal cap and record limits remain unchanged. New operations or unrelated writes may refuse earlier to preserve recovery capacity. This does not reserve native SDK undo storage, guarantee filesystem availability, protect against external file edits or establish save/reload identity. Matched installation and read-only server/client Menu readiness pass. Native staged-recovery acceptance remains pending.

## Construction budget observation

Server 0.37.0 adds `construction_budget` with an empty argument object. It reports `journal_bytes`, the fixed `journal_limit_bytes`, `retained_recovery_bytes`, `recovery_reservation_count`, `unreserved_bytes` and `over_capacity`. Remaining capacity is clamped to zero when file bytes plus protected allowances exceed the cap. A missing or empty journal reports zero file bytes; neither case proves a new or empty world. Counted reservations are runtime accounting owners, including conservative unknown/pending outcomes; they are not a count of available native undo transactions.

Observation is serialized with this mod's shared append/reservation accounting and reads file metadata plus process memory, not World state. It changes no reservation, creates no journal and returns no owner IDs or paths. Scope is `mod_journal_and_session_local_recovery`; `mode` stays `report_only`, and world/native-storage observation, write authorization and reservation-by-read flags stay false. Future appends or external filesystem changes can invalidate the observation. Construction admission and full undo checks remain authoritative. Matched installation and zero-reservation live getter acceptance pass. Nonzero native reservations and staged recovery remain unverified.

Companion 0.16.0 adds `item_id` to each raw `inputs` and `outputs` stack, and `workshop_id` to each recipe row. These are copied u32 archive fields, preserving zero and the full range independently of GUIDs. Correlate nonzero stack item IDs through item lookup and retain all collisions/unmatched results. Workshop definition IDs are not item or entity selectors. No category branch, scaling, catalyst, unlock, station availability or crafting eligibility is established.

Companion 0.17.0 client_workshops reads only the published vanilla workshop registry on the native frame callback. The input workshop ID belongs to the recipe-workshop-definition namespace. Only returned nonzero station_item_id values belong to item lookup; no returned field selects a live entity. The parser admits at most 16 MiB and 4,096 entries after SDK allocation, validates every complete array/record span and decodes at most eight matching rows. Missing definitions are not proof that crafting is impossible, and observed NPC flags do not locate or control a survivor.

Observed recipe workshop GUID references can be zero even when the raw workshop ID resolves to a definition. Preserve raw references and use explicitly scoped ID lookup; do not claim GUID-reference verification in that case. A zero/NIL declared station item is an unresolved declaration, not evidence that no station is required or available.

Companion 0.18.0 successful client_recipes results include workshop_reference_layout_checked: true. The SDK recipe array/record layout and workshopGuid/workshopId field positions and kinds must agree with canonical published reflection before parsing. This checks interface-description agreement only, preserves raw values and never fills a zero reference from client_workshops. Why observed GUIDs are zero, their value-level agreement with definitions and crafting applicability remain unverified.


## Command status observation age

Companion 0.20.0 adds `status_observation` to `client_command_status` and each `client_commands` row: monotonic milliseconds since the last client-frame SDK status poll, `stale` after 1,000 whole milliseconds, and whether polling has finished. Never-polled and prior-boot records have null age and are stale; prior-boot polling state is also unknown. A poll error is a recently checked unknown outcome, not successful native execution. Retained terminal outcomes remain historical when their poll becomes old. Listener reads compute age without polling the SDK, refreshing the timestamp, submitting input or changing journals. Initial submissions and same-identity command replays retain their existing receipts; explicitly read status for freshness. Poll freshness never proves current UI, arrival or rendering. Native acceptance of this metadata remains unverified.


## Declared routes in phase plans

Optional document-level `routes` audit one to eight named declared box routes. Each route has exactly `id`, `body` (`width`, `height`, `depth`) and two to 64 `poses` as `[x,y,z]` box minima. Use the half-metre grid; body width/depth are 0.5..2 m and height is 0.5..4 m. Every pose must fit the world and leave an in-world cell directly below its footprint. The report checks every planned footing cell, every headspace cell, axis-adjacent footprints and vertical changes of at most 0.5 m. Gaps, diagonal/duplicate transitions and excessive rises/drops remain explicit failures. Per-module routes are refused. Without routes, legacy output remains identical. These are caller-declared boxes and a geometric step constraint, not measured avatar dimensions, continuous swept collision, structural support or walkability. No current world occupancy, movement, execution or recovery authority is established.

For a one-metre-wide, two-metre-deep deck at y=4, a synthetic route at its top can be written as:

```json
{"material":"stone","parts":[{"id":"deck","kind":"floor","x":4,"y":4,"z":4,"width":1,"height":0.5,"depth":2}],"routes":[{"id":"access","body":{"width":1,"height":2,"depth":1},"poses":[[4,4.5,4],[4,4.5,5]]}]}
```

The optional routes report retains each pose and join, all conflicting headspace and missing footing cells, and planned_route_valid. Review the declared dimensions and unverified flags before interpreting that result. Conflicts are planning evidence, not an execution gate or a supported movement command.


## Server weather observation

Server 0.42.0 adds read-only `weather_read` with required finite world-bounded `x`, `y`, `z` in metres. It queues one published SDK sample on the server game thread and reports world key, position, clear/rain/snow/blizzard weights, their sum and the SDK strongest-state name. Unavailable has null weights; observed clear and observed zero weights remain distinct. Finite weights are preserved without normalization. Dominance requires every weight in 0..1 with a positive signal; otherwise its name is null. A null inside the weight object denotes a non-finite SDK value, and a non-finite sum is null. This is a point observation, not rendered weather, shelter or actor-safety verification. No weather/front mutation or unbounded front enumeration is exposed. Native qualification remains pending.
