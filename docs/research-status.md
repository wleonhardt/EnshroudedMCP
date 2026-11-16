# Evidence and remaining work

## Bounded native server lifecycle acceptance

The isolated alpha.17 / server 0.34.0 fixture accepts one deliberate native server_save with no connected player characters and the client closed. Missing write flags refuse both save and shutdown before dispatch. Same-session save replay returns the retained receipt without changing the journal. Independent save metadata changes, but the receipt correctly retains save_completed_verified and causal_completion_verified false; timestamps and an uncorrelated SDK event do not establish which request completed a save.

Two separately deliberate shutdown requests run in different server sessions. Each records native acceptance, shutdown_started and then world_saved_after_shutdown. The first external observer reaches its bounded timeout before the process exits; later process absence is observed, but that observer supplies no exit code. No request is repeated in that session. After a verified 91-member cold archive and restart/read-back, prior save/shutdown identities report durable_outcome_unknown and real reuse refuses without journal changes.

The second fixture attaches the independent observer's limited-query/synchronize process handle before native dispatch in the same supervising process. It verifies PID, creation identity and reviewed executable, then observes a signaled process handle with exit code 0. This avoids consuming the observation window during tool handoff. A fresh-session request is not a retry of the old process. The observer still reports final_save_verified and causal_completion_verified false.

A verified 93-member cold archive follows that exit and includes the save identity, both shutdown identities, prior operation ledgers and all ten client command identities. A separate final restart verifies every previous complete voxel capture and soil column, the explicitly restored research setting, exact archived ledger membership/hashes, all 12 installed binary/signature hashes, zero players and absent write flags. Native receipt identities are never reconstructed after reload. The isolated server is running for the next fixture; the client remains closed.

This is bounded preservation and recovery evidence for an unchanged known world, not proof that an arbitrary concurrent edit was captured by a particular save request. Connected/loading-peer races, held-save refusal, unsupported native builds, live audit/storage faults, shutdown reply loss and public-namespace installation remain separate acceptance gaps. No process kill, forced save, automated restart after unknown state or shared-server deployment occurs.

## Bounded native client-control acceptance

The isolated research pair now runs server 0.34.0 / companion 0.15.0 with official signed alpha.17, ABI 12.1. A verified 77-member cold archive, eight rollback binaries and explicit prior absence of four signature sidecars preceded matched replacement. All 12 installed hashes pass. The public namespace remains uninstalled.

A healthy sole-player session exercises native map opening, Backpack and PlayerCrafting tab changes, Workbench recipe highlighting, closing to the world, two small yaw look requests and bounded movement. Fresh native snapshots and separate rendered observations agree. The menu retained its Map HUD parent while the native inner tab correctly changed. Recipe highlighting changes selection without crafting. A 150 ms movement hold produced an independently observed server displacement of about 0.231 m with full health; this is one local input fixture, not navigation, arrival or collision acceptance. No pitch calibration is claimed.

Same-payload replay leaves the journal and durable identities unchanged; conflicting payloads refuse. A separate small 250 ms movement request cancels through the deliberate tool and reaches native Cancelled(ByMod). Clean logout is independently verified by native Menu and zero server players. The temporary research setting is explicitly restored. Complete previous captures and soil, prior operation ledgers, exact journal prefixes plus four expected server and 32 client records, ten new client identities, 12 installed hashes and four absent write flags pass.

After a clean client exit/restart, retained native IDs are not reconstructed: old operation status reports prior_boot_unavailable and a real reuse attempt refuses. The ten identities and client journal remain byte-identical. The client then exits from Menu. The isolated server remains running with zero players and no write flags for the next lifecycle fixture. This was the client-only checkpoint; server save/shutdown and post-fixture cold persistence were then pending. The later bounded lifecycle fixture records those successful isolated checks separately, preserving causal and edge-case limits.

Join, protected credential entry, logout, process lifecycle and rendered capture still use specifically scoped desktop/host fallbacks. The supported SDK supplies no native executors for those operations. Protected source-reference entry and one deliberate join succeed without exposing the credential. Generic screenshots or getters do not close those native gaps. Broader screen targets, tab targets, pitch, player-input cancellation, disconnect timing and live audit-fault behavior remain unverified.

## Official alpha.17 source foundation

The foundation checkpoint is server 0.34.0 / companion 0.13.0, official release 6a151a9550ed7a6ade99c23fa817a91e25b71bc9, ABI 12.1. Cargo uses the unmodified exact Git dependency; no source override or SDK patch is active. Official archive checksum, both framework signatures and tamper rejection pass. Required consumer checks and targeted SDK checks pass. Installed alpha.16 research files remain unchanged; matched alpha.17 deployment and live acceptance are pending. See [decision 0024](../plans/decisions/0024-alpha17-native-control-foundation.md).

Alpha.17 now publishes native client snapshots and bounded screen/tab, movement/look and recipe-highlight commands with status/cancel. This compatibility increment adds no MCP adapter. Implement adapters next with explicit client scope, session/screen gates and durable outcome reconciliation; native join, protected credential entry, logout and rendered evidence remain separate gaps. Historical evidence below retains its original source/runtime scope.

## Direct recipe-ID comparison

The isolated 0.27.0/0.12.0 pair passes a matched cold deployment and live comparison: two observed archive IDs return complete unchanged output-query metadata at Menu and during a healthy connected session. Zero/max-ID queries and invalid selectors are exercised. Clean logout, restored settings, complete captures, archived ledgers and eight installed binary hashes pass. Live duplicate/zero/max-ID records, actual selected-UI applicability and public-namespace installation remain unverified.

## Forward clock fixture

The isolated 0.27.0/0.11.0 pair now passes a bounded clock fixture after a verified cold archive and server-only replacement. One native request was accepted; immediate read-back retained the old hour and correctly stayed `readback_unverified`. A later independent observation reached the requested hour, without promoting the retained receipt. Dry preview, missing-gate and invalid-target refusals, same-request replay, altered-request refusal and operation status pass. A healthy connected client separately rendered daylight consistent with its native daytime observation; connected preview was refused. Clean logout, restored settings, complete voxel captures, archived ledgers and installed binary hashes pass. This does not verify a before/after lighting transition, elapsed world progression, unrelated gameplay effects or public-namespace installation.


## ABI 12 source candidate

Server 0.26.0 / companion 0.11.0 qualify exact official alpha.16 revision 5436ffdd, ABI 12.0, in separate source checkouts. Product checks pass 215 Rust tests, 35 Python tests, Clippy, formatting and release builds. The patched SDK full check passes 4,050 main tests, both compile fixtures, full/stable doctests and Clippy, documentation/link checks and cargo-deny; nine main tests remain skipped by upstream. Generated Luau definitions match the metadata field. A reproduced shared-directory compile-fixture race is addressed by scoped nextest serialization, preserving separate processes and every assertion.

The matching official development artifact's digest and ZIP integrity pass. It lacks release signatures; published loader code reports unsigned builds as E1011/unofficial while allowing them to run. A namespace-correct research rebuild passed the same checks before matched cold installation. A verified 47-file save/ledger archive and all eight rollback binaries preceded replacement. After restart, all eight installed hashes, complete prior captures/ledgers, fresh native Menu, zero players, restored settings and absent write flags pass. Independent rendered Menu agrees with native state and displays the alpha.16+dev.5436ffdd framework footer. Connected gameplay and neutral-namespace installation remain unverified. Earlier ABI 11.30 fixtures retain their original scope. See [source and artifact reproduction](../patches/README.md).

The first fixed probes refused discovery because their version/provenance pins still described the earlier pair. Current-version test fixtures reproduce the failure. Updating the exact pins makes all 35 helper tests pass while explicitly rejecting prior approved ABI/version values. All 21 server and nine companion fixed read-only checks now pass on the cold ABI 12 pair; the report deliberately stays incomplete for missing fixture/gameplay acceptance.

A bounded connected-client fixture preserved health at 100/100 and fresh InGame frames. The complete native recipe response equals the prior qualified response, including direct recipe IDs. Scoped knowledge preserves Player/Flags values 1 and 0 versus unknown/null, requested order, and account/case-variant selection. World queries remain unknown for those player-scoped IDs. The server prompt target's item/template agrees with independent item metadata and client archive identity; localized prompt text agrees with the rendered English altar fixture. Client screen-host template/position remain unavailable, as in the preceding fixture, so no cross-client entity/position correspondence is accepted.

Temporary pacification was enabled and restored through fresh gates, read-back and four journal records. Clean logout left fresh Menu, zero players, settings restored and flags absent. Complete prior captures, operation/client ledgers, server journal prefix and all eight binary hashes pass. This does not requalify ABI 12 construction/reposition/health-restoration/client-marker writes, screen-host position units, current recipe/tab selection or crafting eligibility. Native semantic movement/look/menu/crafting executors remain unavailable.

An existing client interruption left its HTTP listener answering with absent/stale UI frames. Authorized desktop fallback identified a crash/upload dialog and a later startup question reporting a prior GPU crash. Upload and dump collection were declined; no dump was inspected. One unchanged-pair relaunch recovered fresh native Menu, and complete preservation checks passed. The crash cause remains unknown. This motivates native lifecycle/crash readiness and explicit recovery controls, not an inference that an answering endpoint proves rendered-client health.

## Native zone-rule fixture

Ninety-two read-only queries admitted one directly queried 39 m cube and refused six independent half-metre face expansions. Repeated local axis transition observations agree; no journal, operation, capture or binary changed. These are bounded native-rule observations, not general base dimensions, collision/contact or save proof. See [method and limits](zone-rule-fixture.md).

This is a research prototype with bounded administrative capabilities. Evidence is separated into source tests, fixed transport probes and controlled game fixtures. It does not establish general automation or a functional workshop.

## Reproducible source checks

The server 0.16.0 / companion 0.3.0 checkpoint passed 146 Rust tests, 30 Python helper tests, Clippy with warnings denied, formatting and release builds. The exact framework revision and SDK-only metadata patch passed 538 tests under the framework's prescribed nextest harness. These counts identify a development checkpoint, not an ongoing CI badge or a guarantee for another game build.

## Controlled observations

| Area | Evidence | Boundary |
| --- | --- | --- |
| Reads and transport | Eighteen fixed server probes and nine companion probes passed on the isolated installation. | Reports deliberately retain incomplete fixture/gameplay acceptance. |
| Inventory and health | Controlled stack changes, one durability comparison and one enemy health decrease were observed. Native player vitals distinguish unavailable data from zero. | Broad item semantics and UI-unit conversion are unverified. Positive health alone does not establish safety or quiescence. |
| Construction | One-cell, multi-cell, multi-brick and staged frame/filler trials verified exact images, deliberate replay and guarded reverse undo. | No general concurrent-edit safety, cost accounting or maximum-cap performance claim. |
| Composite shell | Six parts yielded 164 unique cells; all 3932 other cells in the 4096-cell region stayed unchanged. Rendered front/roof-edge/oblique views and disappearance after undo were observed. | Raised entrance, traversal, stairs, sloped roofs and functional fixtures were not accepted. |
| Cold recovery | Reverted geometry remained exact across controlled cold reloads; stale receipts/tokens were refused and ledgers remained unchanged. | A baseline reload does not prove an applied multi-part structure or player position persists. |
| Repositioning | Controlled once-command trials recorded one intent/outcome per identity, discarded-response reconciliation and unchanged replay journals. Subsequent server positions changed. | This is not direct live instrumentation of native invocation count, exact arrival/return, camera control or bounded engine correction. |
| Client companion | Fresh Menu/InGame observations agreed with visible screens. Named events have isolated historical evidence. | Current passive tab/recipe selection, migrated named-event delivery, a non-null host-template fixture and held-input semantics need further evidence. |

An isolated character died while idle during read-only observation. Normal client respawn and grave retrieval recovered the inventory; the final per-item totals matched the earlier snapshot. This is not evidence that a read caused death, nor evidence that a healthy-player check can prevent game hazards. No health restoration, inventory grant or write-gate bypass was used for recovery.

## Latest isolated checkpoint

The corrected server write gate was installed alone after a verified 40-file cold archive. Loader and companion hashes stayed unchanged. Directory and future-dated flag fixtures were refused using deliberately invalid player/item selectors; no inventory grant occurred. All 4096 preserved cells and complete pre-restart ledger bytes remained exact. An initial startup run passed 17 of 18 probes while construction was inactive; after observed readiness, all 18 passed, with broader acceptance still incomplete. These fixtures use the existing research installation; the neutral public mod namespace has not been independently live-installed.

The framework maintainer [reported a suspected held-input defect and planned diagnostic support](https://github.com/enshwick/enshwick/issues/1107#issuecomment-6883127694). That statement was historical: alpha.17 now publishes bounded native look/movement and in-game screen/tab APIs, as described above. Main-menu join and logout remain unsupported in the reviewed API. A focused cached-diagnostic draft has been shared upstream for review; it is not an input executor and its candidate loader remains uninstalled.

## Next acceptance milestones

- Make a small shell accessible with independently reviewed support and entry geometry, then verify traversal visually and in the game.
- Establish supported prop identity, complete footprint and label-safe recovery before placing storage, doors or stations.
- Prove functional storage and one workbench recipe separately from geometric placement.
- Validate applied composed-structure persistence on a disposable fixture without overwriting unrelated state.
- Expand measured building modules and durable plan revisions before considering a general job executor.
- Obtain supported current-selection and semantic client-action APIs rather than treating input observations as actuators.

The long-term benchmark is a usable multi-storey workshop with stairs, roof, doors, storage and workstations. It remains open. Survival debit/refund, progression feasibility, arbitrary stateful rollback and unattended operation remain outside current acceptance.

Detailed sanitized checkpoints remain in [public evidence](../plans/research/live-validation.md). Private runtime identities, exact test locations, saves and screenshots are intentionally excluded.

## Current source and isolated boundaries

Server 0.20.0 and companion 0.5.1 pass 173 Rust tests and thirty Python tests, Clippy, formatting and release builds. The new prop template/label-presence output is not live-installed. The isolated research pair runs server 0.19.0/companion 0.5.1 on official acd5550c, ABI 11.30, and passes 27 fixed native probes. Native foundation preview agrees with an independent two-cell oracle; rendered contact is pending. Workbench item/category branch counts are observed, without category membership or feasibility. Explicit pacification false-to-true-to-false changes and boolean restoration passed independent read-back and journaling; full preset restoration and actual combat behavior remain unverified. No prop-placement lifecycle is claimed.

The reviewed diagnostic draft is now shared upstream; it remains separate from the missing supported native menu/movement executor. Public namespace binaries have not been independently live-installed. Follow the [phase gates](../plans/phases.md); source tests and transport probes never replace rendered acceptance.

## Companion 0.5.2 isolated deployment

The original research namespace companion 0.5.2 is installed on the isolated pair after a verified 44-file cold archive and two hashed DLL rollback copies. Only companion DLLs changed; server 0.19.0, official framework acd5550c/ABI 11.30 and loader binaries are unchanged. Nine companion and eighteen version-matched server read probes pass. Native client state reports a fresh Menu. Both complete world captures (6,144 cells total), archived ledger membership/hashes, zero players and disabled writes are verified.

An initial startup probe was unavailable; the same running client subsequently became ready without restart. The selection-history gap correction still has offline regression coverage only: no real join/logout was performed. Native lifecycle/menu/movement execution remains unavailable. Neutral publication-namespace binaries have not been independently live-installed.

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


## Matched isolated deployment: server 0.21.2 / companion 0.7.0

The original research namespace now runs the matched pair on official acd5550c/ABI 11.30. A verified 44-file cold archive and four hashed rollback DLL copies preceded replacement. Framework and loader hashes stayed unchanged. All nineteen server and nine companion fixed probes pass after readiness; the first server startup probe failed its unknown-operation check. Reports retain incomplete semantic acceptance.

A bounded desktop fallback joined the intended isolated server using redacted source-reference password input. The server observed one connected player and a rendered in-game menu was visible. The character showed a red health indication, so the session ended immediately. The attempted native health lookup used an unsupported tool name; no native health value was established. Fresh native Menu, zero players, both complete world captures and unchanged ledger membership/hashes were verified afterward. Writes remained disabled; no administrative grants, construction or research-setting changes occurred. Neutral publication-namespace binaries remain untested live.

The installed companion successfully decoded the Workbench recipe requirement fields in its fixed archive probe. UI eligibility, complete requirements, positive grant execution and connected selection events remain unverified. Desktop join/logout evidence does not establish a native lifecycle executor.


## Server 0.22.0 native administrative health restoration

The matched isolated research installation now runs server 0.22.0/companion 0.7.0. A verified 44-file cold archive and two rollback DLLs preceded the server-only update; six companion/framework/loader binaries stayed unchanged. After readiness, nineteen server and nine companion fixed probes pass. The first server startup report failed only its unknown-operation readiness check and is retained.

A bounded sole-player session used the new dry preview and separately gated native restoration. Health was independently observed at 17/100 before and 100/100 afterward; the rendered bar appeared full. Exactly one correlated intent/outcome pair was appended, with the prior journal prefix unchanged. The write flag was removed immediately, then the client logged out. Fresh Menu/zero players, complete world captures, unchanged other operation ledgers and all installed binary hashes were verified. Health was never deliberately reduced for this fixture. This provides limited restorative evidence, not general combat safety, UI-unit conversion, item consumption or public-namespace live installation.

Policy/adapter tests cover invalid health/context, dry/full no-op, gate and journal failures, stale actor state, independent read-back uncertainty and native SDK scheduling. A revoked-flag regression first failed and now refuses dispatch after the final observation. Fake-host tests also verify late-tick cancellation and actor-lifecycle preview invalidation.


## Two-cell foundation fixture and cold recovery

On server 0.22.0/companion 0.7.0, a bounded isolated session tested a two-cell Air-only pier above an independently observed terrain stop. The native foundation preview matched a separate finite-column oracle. Full health and observed pacification were checked before preview and placement. Exactly two of the 512 captured cells changed to the intended material/fill; all other cells stayed exact. The pier was visible from two positions, then disappeared after guarded undo.

Grass obscured portions of the base seam. These views establish rendered placement and disappearance, but do not satisfy complete ground-contact acceptance. No vegetation removal, terrain edit, traversal or functional-workshop claim follows. Desktop menu/private-entry, movement/look and capture were temporary fallbacks; supported native client executors remain missing.

A verified 44-file cold archive preceded the experiment. After undo and logout, the original pacification false value was explicitly restored and read back. A verified 45-file post-undo archive preceded restart. Cold reload preserved the exact 512-cell fixture and both larger world baselines, archived ledger membership/hashes and installed binaries. The old native receipt was not reconstructed; stale apply/accept/undo were refused and an old preview was rejected for session mismatch without a new reservation or journal change. The prior journal prefix remained exact plus six construction records and four setting records. Writes ended disabled with zero players. This proves baseline recovery, not persistence while placed or general terrain contact.

The next fixture must expose the terrain/building seam without changing vegetation or unrelated world data. Native support/clearance/oracle checks and healthy, quiet rendered observations remain prerequisites.


## Native Air versus rendered clearance

A second isolated candidate selected a different terrain material after complete native capture and independent two-cell oracle agreement. A night attempt stopped before placement; the client logged out and the setting was restored while a bounded native clock watcher waited for day. The later fixture required observed day, full health and pacification, with a separate rendered review.

The candidate did not provide an exposed seam. A small reversible registration trial changed exactly its two native Air cells as intended, but rendered only a cap-like fragment against an existing rock from two views. The fragment disappeared after exact undo. This is evidence of apparent visual overlap/occlusion; the rock's representation and the underlying cause are unresolved. Neither native Air, a terrain material ID nor an empty placed-prop query proves render-space or collision clearance. Complete contact acceptance remains open.

Verified 45/46-file cold archives bracketed the attempts. After logout and explicit setting restoration, complete candidate/global captures remained exact; old ledgers and the prior journal prefix were preserved plus the expected six construction and eight setting records. Cold reload retained all archived ledgers and binaries; stale requests and the old-session preview refused without new reservations. No vegetation removal, terrain replacement, clock mutation or administrative healing occurred.

## Native map control checkpoint

Companion 0.8.0 implements bounded transient map-marker submission and explicit clearing. The isolated joined-client fixture accepted show/clear/show and automatic observed-leave cleanup, preserving empty submitted state after logout. Rendered visibility remains pending because fallback map shortcuts did not open the map. Prioritize supported native join/private entry, map/menu opening and logout executors; do not equate cached observations with control. See [tool reference](tool-reference.md).

### Rendered map follow-up

A separate isolated fixture opened World/map through the authorized desktop fallback. Native marker show and clear were accepted, but no distinct annotation label/icon was observed at the near-player offset, including after reopening the map. Existing world/player icons remained visible. The small offset may overlap existing icons at this scale; the cause is unresolved. Next use a clearly separated bounded annotation and observed hover before diagnosing the native render path. No fast travel or saved-marker editing was exercised.

Explicit clear preceded logout; research settings were restored and flags removed. Complete baseline captures, operation-ledger membership/hashes and installed binaries remained exact. This adds four companion audit records and four server research-setting records. SDK submission remains verified separately from rendered visibility, which is still unaccepted. Supported native map opening and rendered feedback remain roadmap priorities.

### Separated annotation result

A distinct bounded annotation farther from the player also failed to establish a visible label/icon. Existing altar hover responded; fresh companion state remained InGame/submitted, ruling out observed-leave cleanup during this view. The cause remains unresolved. Stop equivalent visibility retries pending a supported per-frame publication diagnostic or another concrete discriminator. Copied-set acceptance is still not rendered proof.

Explicit clear and logout passed. Research settings were restored, flags removed, complete baseline captures and operation ledgers remained exact, and installed binaries were unchanged. No native rendering fix or workshop-prop acceptance is claimed. General safe prop recovery still needs supported complete placement-footprint and exact prior-label observations.

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

## Copied native snapshot adapter

Companion 0.14.0 adds client_snapshot over the SDK's copied any-thread getter. The actual inner tab and recipe highlight are distinct from named event history and archive lookup. State/age, session and per-part availability are preserved; IDs remain client-only and zero player/recipe sentinels become null. The tool submits no commands and touches no World. Private and independent sanitized-public checks pass 275 Rust/40 Python tests, Clippy, formatting and both release builds. FakeHost verifies freshness, availability, sentinel semantics and absence of command submission; the fixed probe refuses malformed scope/availability and omits sensitive identities, selections and held input from summaries. Live acceptance remains unverified; installed files remain unchanged. Native guarded screen commands are next, before matched deployment.

## Supported native client command adapters

Companion 0.15.0 wraps all six alpha.17 command families with default dry preview, fresh session/HUD/tab admission, companion write gate, durable once-only identities, retained status and deliberate cancellation. Private and independent sanitized-public checks pass 280 Rust/40 Python tests, Clippy, formatting and both release builds. FakeHost covers supported dispatch, replay/conflict, stale preview, cancellation, prior-boot refusal, audit failure and client-frame queue cancellation. A failing underflow reproduction now refuses a direction that vanishes at native float precision. No live alpha.17 command effect is claimed. Matched cold deployment and independent native/rendered acceptance are next. See [decision 0025](../plans/decisions/0025-once-only-native-client-commands.md).
