# Evidence and remaining work

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

The framework maintainer [reported a suspected held-input defect and planned diagnostic support](https://github.com/enshwick/enshwick/issues/1107#issuecomment-6883127694). Native look/movement/menu work is being investigated upstream. These are maintainer plans, not released capabilities. A focused cached-diagnostic draft has been shared upstream for review; it is not an input executor and its candidate loader remains uninstalled.

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

## Queued inventory grant admission (source 0.21.1)

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
