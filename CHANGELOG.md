# Changelog

## Unreleased (phase clearance and regression coverage)

- Add check_contracts.py for offline capability-index/profile checks and optional read-only loopback discovery/provenance comparison. Contract drift refuses without running gameplay tools; nine behavior tests cover mismatches and fixed metadata-only dispatch.

- Add a source cancellation contract for non-file audit refusal, deliberate recovery, once-only attempt ownership and prior-boot refusal. An isolated missing-attempt-guard mutation fails the independent SDK state check. Production behavior is unchanged.

- Repair the client invalid-context regression to assert the intended refusal after proving a valid fixture. An isolated missing-gate mutation now fails instead of passing through an unrelated preview error. Production guards and runtime behavior are unchanged.

- Add source-only client frame fault contracts for cancelled late real writes and reply loss after native dispatch. Explicit same-ID replay preserves exactly one native command and durable reservation; production code and two-second deadlines are unchanged. Live stalls/disconnect remain unverified.

- Fix health-recovery test fixture setup when Cargo uses an external target directory. Create the local parent explicitly and constrain cleanup to its canonical direct child. Runtime behavior and SDK pin are unchanged.

- Add inactive merged-SDK placement consumer contracts preserving raw corner order, unknown bounds, native errors and deferred reads. Product workspace/pins and installed runtime are unchanged; no MCP metadata tool or native acceptance is added.

- Record merged upstream placement and diagnostic proposals. Matching dev artifact digest passes, but absent detached signatures keep provider/native adoption unqualified; product SDK and ABI remain unchanged.

- Record native Backpack/PlayerCrafting tab transitions with HUD/tab separation and complete preservation. SDK outcomes and getter fields remain separate from pending rendered confirmation; no craft or movement action is added.

- Add read-only retained shutdown status independent of active World and write admission. Busy/prior-boot states remain explicit; no native retry or process/save proof. Private/public source qualification and matched 0.45.0/0.20.2 installation pass. Sixteen native retained replies, including fifteen after ShutdownStarted, and 24 prior-boot identities preserve proof limits and audit bytes. Causal save and native busy-state cases remain unverified.

- Server 0.44.0 source adds read-only current-boot reposition observation, with separate immutable native outcome and fresh bound actor position/distance. No arrival threshold, dispatch, retry or journal change. Source qualification and matched installation pass, including five prior-boot reposition identities, 35 server probes and 13 client probes. Joined current-boot flag-free position/distance observation now passes with preserved receipts and full baselines. Native stale-session/disconnect behavior remains unverified.

- Record existing-platform standing correspondence and a failed exact-return criterion after native spawn adjustment. Preserve receipts, raw nonzero building fill and all world/ledger baselines. Queue read-only reposition observation; no new movement or construction executor.

- Qualify joined 0.43.0/0.20.2 command reply freshness, user-confirmed visible map open/close, eight nonempty inventory counts and one positive buff response. Preserve all voxel fixtures, ledger prefixes and installed bytes; pending cancellation, countdown/effects, traversal and complete workshop acceptance remain open.

- Companion 0.20.2 source returns polling freshness consistently on command submissions/replays and cancellation outcomes, including dry last_outcome. Retained reads do not refresh observations or change execution, journals or retry gates. Matched 0.43.0/0.20.2 native read-only/prior-boot regression now passes; joined/current-boot reply behavior remains unverified.

- Server 0.43.0 adds read-only active buff observations for one server entity, with bounded output, preserved SDK errors and exact projected game-time durations. No mutation or safety claim. Installed native missing-entity/error, strict refusal and full read-only preservation checks pass; positive holder and UI/countdown semantics remain unverified.

- Resolve upstream #1141 conflicts with a normal merge of b86d94d1. Append placement metadata at ABI 12.8/slot 212, preserving settlement and statistics APIs. Preserve the older exact-base artifacts; no runtime migration.

- Companion 0.20.1 clears stale native command age after SDK status-poll errors. Status, inventory and same-identity replay retain unknown outcomes with null age; no redispatch. Read-only probe checks refuse mixed old-age/error observations.

- Audit continuous linear declared-box intersections between admitted planned route poses. Expose swept-cell conflicts, preserve unknown skipped joins and require clear sweeps for route validity. Stair risers can fail despite clear supported endpoints; no native traversal claim or runtime change.

- Refresh the inactive placement proposal as proposed ABI 12.6/slot 211 against exact SDK 73c18f31; record source qualification, submitted request and ready-for-review PRs. No runtime migration.

- Preserve inactive ABI 12.4 client-feedback consumer contracts and an isolated source migration patch against exact published main dc0cf4ce. Hovered slots and entity-backed tooltip data remain client-only observations, without selection/visibility or execution authority. Mark the older placement proposal for rebase because its slot is now occupied. Product dependencies and installed runtime remain unchanged; native provider compatibility is unverified.

- Server 0.42.0 adds bounded read-only weather_read through the published SDK on the game thread. Preserve unavailable versus observed weights, raw finite values, weight sum and guarded SDK dominance without normalizing, changing forecasts or claiming rendered/shelter/safety evidence. Strict coordinate/write-argument admission and deferred fake-host sample behavior have regressions. Matched native 0.42.0/0.20.0 acceptance passes six samples at three points, independent weight/sum/dominance checks, eleven strict refusals and full preservation. Rendered/shelter/safety interpretation remains unverified.

- Read-only phase planning now audits declared box routes against the complete planned union: occupied footing, empty headspace, adjacent footprints and bounded vertical changes. Preserve route-free output exactly and reject invalid/bounded inputs before native previews. Native pure-preview qualification detects old compact roof conflicts and a missing waypoint while retaining the complete revised workshop geometry. No runtime binary, world write or walkability claim is added.

- Companion 0.20.0 reports last frame status-poll age/staleness and polling-finished state for command status and inventory. Distinguish never-polled, stalled and historical terminal results without refreshing age on reads or changing native dispatch, receipts or journals. Matched 0.41.0/0.20.0 native Menu acceptance verifies unknown/stale metadata for all 18 prior-boot identities and read-only preservation. Current-boot SDK polling and callback-stall behavior remain unverified.

- Companion 0.19.0 adds a bounded read-only command inventory for recovering caller operation identities and retained outcomes after lost replies or reloads. It never decodes old SDK IDs or dispatches input. Invalid reservation entries refuse; terminal cancellation/unavailability and unchanged replay/audit behavior have regression coverage. Matched 0.41.0/0.19.0 native Menu inventory acceptance passes, with 18 prior-boot identities, exact status agreement, strict refusals and unchanged client ledgers. Connected fault/rendered acceptance remains unverified.

- Preserve an inactive, reproducible SDK prop-placement metadata proposal. It reuses the existing resolver, appends ABI 12.2 data/entry without moving old slots and coordinates safe/fake/native/C/Luau interfaces. Upstream model/lint/docs checks pass; no product dependency, installed loader or control authority changes. Native provider acceptance remains unverified.
- Make the clock adapter fixture deterministic with explicit scheduler passes, controlled elapsed time and separate reply observation. Cover expiry while queued, cancellation, ready results/errors and uncertainty after execution starts. Production keeps the original timer start point, five-second limits, durable replay and admission rules; no clock-tool behavior or authority changes.
- Remove wall-clock worker races from the health-recovery SDK fixture by driving queued requests and receipt observation separately. Preserve the production five-second deadline, cancellation before execution, uncertainty after execution starts and result/error propagation. Add deterministic receipt-boundary coverage; no health-tool behavior or authority changes.
- Extend the optional SDK qualification suite with prop placement-permission and mixed-write/undo held-state contracts. Test rotated bounds, state acquired after queuing for all nine published kinds and an empty-state recovery control. Synthetic model evidence grants no native recovery authority; runtime pins and installed builds are unchanged.
- Preserve runnable SDK settlement-model probes in an optional independent workspace. Seven scenarios exercise callback timing, expiry, historical entity IDs, failure, undo and world scope without changing the MCP runtime dependency.
- Exercise populated factory observations through the production adapter, published SDK component codec and queued scheduler pass. Cover all six states, four-choice arrays, numeric extremes and unknown state/flags while preserving bytes and uncertainty. This is synthetic SDK regression coverage; populated live factory and production semantics remain unverified.
- Preserve document-level phase clearance evidence across the complete cell union. Check up to 32 bounded absolute volumes through native pure previews and report exact later-phase conflicts. Refuse per-module clearance that was previously silently discarded. Reports remain geometry-only, with world occupancy and walkability unverified.
- Exercise the production client selection adapter through published typed event delivery on both client and dedicated-server sides. Preserve bounded raw category choices, server/client isolation, unknown current selection and session clearing. This is source regression coverage, not live category/crafting acceptance; installed runtime and MCP behavior are unchanged.

## Server 0.41.0 (multi-output recipe planning)

- Accept mutually exclusive legacy item/count or one to 16 explicit outputs per supplied recipe. Group items by selected producer, aggregate consumer demand and stock once, share batch rounding and report every output and surplus. Choices extend to 512 items; all existing catalogue/arithmetic gates remain.
- Preserve explicit output-producer allocation: coproducts assigned elsewhere remain surplus. Preserve legacy response shape; bundle steps and reports expose complete outputs and allocation. Verify with a qualified synthetic legacy response and 1,296 independent batch-search/inventory-conservation cases. Matched 0.41.0/0.18.0 cold deployment and 1,296 native synthetic comparisons now pass, including exact step yields/costs and complete balances, branching/explicit-overlap sequences, 144 choices/16 outputs, thirteen refusals and all fixed reads with source/hash/ledger preservation. Game coproduct/eligibility/crafting semantics remain unverified.

## Server 0.40.0 (native factory-station observations)

- Add read-only factory_station_read with one nonzero session-local server entity selector. Resolve the published typed FactoryStation handle optionally and queue its read on the game thread.
- Preserve running/next recipe IDs and bounded raw category choices, signed raw times, stop/use-all flags and SDK missing-requirement names. Unknown differs from not observed. Report no complete queue, current client selection, crafting eligibility, time conversion or safe-removal proof. Private/public source checks pass. Matched 0.40.0/0.18.0 deployment verifies native handle resolution, repeated unobserved reads and twelve strict refusals, with all fixed read probes and runtime preservation. Present factory-state and independent semantic acceptance remain unverified.

## Companion 0.18.0 (recipe workshop reference contract)

- Cross-check published RecipeLayout workshop reference offsets, record/array layout and GUID/u32 kinds against canonical recipe reflection before parsing. Refuse disagreement rather than returning plausible fields from an incompatible description.
- Add workshop_reference_layout_checked to successful client_recipes observations. Keep all raw values, including zero GUIDs, unchanged. The check establishes agreement between two published interface descriptions, not value validity, effective definitions, crafting applicability or why a reference is zero. Private/public 338 Rust executions (including 18 shared module checks), 54 Python tests, Clippy, formatting and release builds pass. Matched 0.39.0/0.18.0 native checks pass through both selection routes for six unchanged records, with full preservation and fixed read probes. Zero-reference cause and value-level semantics remain unverified.

## Companion 0.17.0 (workshop definition lookup)

- Add client_workshops with raw workshop_id and optional limit (one to eight). Read the published vanilla registry on the native frame callback and preserve workshop GUID, name tag, declared station item ID/GUID and NPC-workshop flag.
- Keep zero IDs, collisions, total matches and truncation explicit. Bound the complete array and every field in game-free policy; reject malformed records and non-boolean flag bytes. Definitions never establish current station availability, unlocks or crafting eligibility. Whole SDK resource allocation still precedes parser caps. Private/public 336 Rust executions (including 18 shared module checks), 53 Python tests, Clippy, formatting and release builds pass. Matched 0.39.0/0.17.0 deployment resolves four observed nonzero workshop IDs and independently correlates three declared station identities, with fixed probes and full preservation checks. Raw recipe GUID references are zero; their agreement, current availability and crafting applicability remain unverified.

## Companion 0.16.0 (recipe stack and workshop identities)

- Add the raw item_id to every client_recipes input/output stack and the raw workshop_id beside workshop_guid. Copy the published archive fields; preserve zero and full u32 values without deriving IDs from GUIDs.
- Keep matching, collision/truncation reporting, all parser limits and unknown recipe/category/eligibility semantics. Stack item IDs can be correlated through item lookup; workshop definition IDs are not inventory item or entity selectors. Extend the fixed client probe with strict u32 identity checks. Private/public 331 Rust executions (including 18 shared module checks), 52 Python tests, Clippy, formatting and release builds pass. Matched 0.39.0/0.16.0 cold deployment and six-record/eleven-ID native identity correlation pass, with both fixed read probes and complete preservation checks. Category/scaling interpretation, eligible stock and crafting feasibility remain unverified.

## Construction adapter regression coverage

- Add an isolated SDK fake-host integration fixture for the production construction and journal adapters. A native Applied receipt with unchanged voxels must remain a verification conflict and cannot authorize acceptance, progression or multi-cell undo. Replay stays once-only, failed guards preserve audit bytes/recovery credit, and lifecycle changes invalidate retained handles.
- Preserve generic journal reservation coverage in the caller module. The SDK fake host does not mutate its voxel store, so this fixture is not successful placement or reverse-undo evidence. Runtime behavior, SDK pin and installed 0.39.0 DLLs are unchanged; source verification and live staged recovery remain separate.

## Server 0.39.0 (same-read inventory item counts)

- Extend inventory_read with optional item_ids (one to eight unique nonzero IDs). Count each physical slot once over the same SDK slot read, independently of the returned page. Existing calls retain their page response.
- Refuse more than 65,536 returned records and conflicting aliases. Keep large totals exact. Report unknown inventory completeness and crafting eligibility, including when observed counts are zero; this is not eligible stock, a reservation or a live crafting claim. The SDK traversal itself is not capped by this post-read bound. Private/public 310 Rust/51 Python tests, Clippy, formatting and release builds pass. Matched 0.39.0 cold installation and live empty-count/admission checks pass, preserving complete captures, immutable ledgers and installed hashes. Nonempty counts, native alias cases and eligible stock remain unverified.

## Server 0.38.0 (explicit recipe alternatives)

- Allow bounded alternative single-output producers in recipe_plan with optional explicit item/recipe choices. Refuse ambiguous, duplicate, unknown or mismatched selections; retain existing single-producer calls.
- Analyze only selected dependencies and requirements while validating every supplied recipe and the full catalogue bound. Preserve batch rounding, shared stock accounting and deterministic results under catalogue reordering. No automatic search, optimality, live stock validation or crafting execution is added. Private/public 305 Rust/50 Python tests, Clippy, formatting and release builds pass. Matched 0.38.0 cold installation and live synthetic alternative transport/read-only preservation checks pass. Connected production-chain, live eligible stock and native staged recovery acceptance remain unverified.

## Server 0.37.0 (read-only construction budget)

- Add construction_budget to observe journal bytes, protected recovery allowances, reservation count, unreserved bytes and over-capacity state without owner IDs. Serialize the observation with shared appends; create no journal and change no reservation.
- Keep scope explicit: no World reads, native undo-storage observation, capacity reservation or write authorization. Reject undeclared arguments. Extend the fixed read-only validator with strict count/type/arithmetic checks. Matched cold installation and zero-reservation live metadata/read-only acceptance pass; nonzero reservation and native staged recovery remain unverified.

## Read-only phase planning script

- Add scripts/phase_plan.py to decompose up to 32 named primitive modules into exact disjoint phases using only pure building_plan calls. Preserve existing construction limits, bound total cells/phases and retain module provenance.
- Revalidate each phase and the exact final union. Reports authorize no execution, world observation, build order or recovery. Independent live pure-preview comparison matches the complete workshop geometry without changing world or ledgers. Private/public 297 Rust/49 Python tests, Clippy, formatting and release builds pass.

## Server 0.36.0 (retained recovery journal reservations)

- Reserve bounded construction undo audit space before native placement. Acceptance retains the allowance; later construction and all shared journal append paths preserve it. Recovery can spend only its own credit after a successful synced append.
- Serialize admission, append and accounting by mod directory. Refuse unknown owners, overflow and insufficient aggregate space. Recheck/replenish complete undo audit headroom before a deliberate undo; dry checks do not change reservations. Release only after exact baseline-verified undo or world lifecycle invalidation.
- Preserve record formats, existing caps, write/session gates, durable identities and fail-stop native semantics. Private/public 297 Rust/40 Python tests, Clippy, formatting and release builds pass. Matched 0.36.0 cold installation, isolated server read-only readiness and fresh companion Menu observation now pass; native staged recovery and live storage-fault acceptance remain pending.

## Retained recovery policy foundation (not runtime integrated)

- Add checked aggregate journal headroom arithmetic and a reproduction of later-cycle admission consuming older retained recovery space. Preserve the existing 8 MiB cap and zero-reservation behavior; refuse overflow and boundary excess.
- Runtime reservation ownership/lifecycle and all shared journal writer integration remain pending. No live protection or product behavior change is claimed by this foundation alone.

## Workshop planning milestone (documentation)

- Independently check a revised two-storey enclosure with eleven requested clearance volumes and a bounded support survey. Decompose 947 distinct foundation/enclosure targets into 16 admitted non-overlapping phases; verify exact full-brick reverse restoration in a read-only model.
- Keep native staged recovery, rendered contact, traversal and functional door/storage/workstation acceptance open. No construction, gameplay mutation or product behavior change is claimed.

## Server 0.35.0 (explicit planned-volume clearance)

- Extend pure building_plan with optional absolute half-metre-grid clearance boxes. Check one to eight world-bounded boxes and at most 1,024 requested cells; refuse malformed and unknown nested fields.
- Report exact planned solid-cell intersections after rotations and compound union. Half-open touching boundaries pass. Omitted clearance preserves existing output. Results observe no world state and prove no avatar clearance, traversal, support or shelter; construction authority is unchanged. Matched cold installation and isolated server transport now pass with independent cramped/raised-roof cell oracles and unchanged world/ledgers. Companion relaunch/connected checks of the rebuilt bundle remain unverified.

## Rendered compound geometry acceptance (documentation)

- Verify separate baseline/applied/undone client views of the floor, three stair rises and stepped roof after native positioning. Preserve independent full-capture, replay, conflict refusal, strict undo and cold-recovery checks.
- Record geometric stair-to-roof gaps of 1.5 m, 1.0 m and 0.5 m. Queue deliberate headroom, landing and grounded workshop design; keep avatar clearance, traversal, shelter and support unverified. No product behavior changes.

## Isolated compound geometry recovery (documentation)

- Verify an independent 132-cell floor/stair/solid-gable oracle across four bricks against a complete 2,048-cell capture. Reordered replay preserves the transaction; conflicting part identity refuses; guarded undo restores the full baseline.
- Verify connected dry lifecycle refusal, clean logout, restored setting, independently observed native shutdown, a 103-member cold archive, complete prior capture recovery, ledger preservation, installed hashes and durable-identity refusal after restart. Native yaw/pitch visibly change the camera; fixture rendering and functional workshop usability remain unverified. No product behavior changes.

## Isolated lifecycle acceptance on server 0.34.0 (documentation)

- Verify deliberate native save acceptance, unchanged replay, ordered graceful-shutdown/save events and an independently observed exit code 0 in a fresh server session. Preserve the earlier observer timeout as an incomplete result; never repeat a request because an observer timed out.
- Verify 91/93-member cold archives, complete capture/soil recovery, persisted restored setting, exact server/client ledgers, prior-boot refusal, 12 installed hashes and absent flags after restart. Keep native receipts causally unverified and retain connected/loading/fault and public-namespace gaps. No product behavior changes.

## Companion 0.15.0 (guarded supported native client commands)

- Add client_command, read-only client_command_status and deliberate client_command_cancel over the published SDK queue/status/cancel APIs. Support in-game screen open/close, tab selection, recipe highlight and bounded look/move; add no arbitrary input or server World access.
- Default dry. Require a fresh nonzero native session, action-specific HUD/tab context, exact preview, companion write gate and synced bounded audit before once-only submission. Keep 64 exclusive durable identities; refuse conflicts/prior-boot reuse and retain outcomes after reply/audit failure without redispatch.
- Poll native outcomes separately and bound cancellation to one deliberate attempt. SDK observed never implies rendered proof or arrival. Reject movement that vanishes at native float precision. Matched isolated alpha.17 deployment and bounded native screen/tab/recipe/yaw/movement/cancellation acceptance now pass. Client reload refuses prior identities; full captures, ledgers, settings and installed hashes are preserved. Broader targets, pitch/fault/disconnect behavior and public namespace remain unverified.

## Companion 0.14.0 (copied native client snapshot)

- Add read-only client_snapshot through the published SDK's any-thread copied state API. Preserve native state/age/session and per-part read flags, actual inner menu tab, held actions, local client identity and highlighted recipe.
- Distinguish unread parts from observed absence; map player zero and the SDK recipe-zero sentinel to null. Keep client IDs out of server selectors. No commands, world reads, crafting eligibility or rendered acceptance are claimed.
- Extend the fixed read-only probe with strict scope/availability checks and sanitized summaries that omit identities, selection and held input. Matched deployment and live acceptance remain pending.

## 0.34.0 / companion 0.13.0 (official alpha.17 source foundation)

- Pin the unmodified official alpha.17 release, revision 6a151a9550ed7a6ade99c23fa817a91e25b71bc9, ABI 12.1, directly through Cargo Git. Retire the local source override and metadata patch; adapt the upstream placed-template Guid while preserving observation semantics.
- Rebuild both consumers and update provenance checks together. The release adds supported native client commands upstream; this compatibility increment exposes no new MCP executor.
- Verify the official archive checksum and both framework release signatures, including tamper rejection. Source checks and targeted SDK checks pass; installed alpha.16 files remain unchanged. Matched deployment and live alpha.17 acceptance remain unverified.

## 0.33.0 (replay-safe caller-identified inventory grants)

- Require give_item operation identity, default dry and exact current actor/item/session preview for real calls. Legacy identity-free requests fail closed; preserve strict counts/selectors and unique actor resolution.
- Sync bounded exclusive grant identities and intent/outcome audit headroom before once-only native dispatch. Reconcile retained same-payload status without another grant; refuse conflicts, stale sessions/previews and reopened durable IDs.
- Return submission status and retain native callback outcomes independently of HTTP replies, flag removal or later world generations. Keep audit failure explicit and inventory_readback_verified false. No automatic retry, native undo or live acceptance claim.

## 0.32.1 (strict legacy grant admission)

- Fix give_item treating a negative count as an omitted value and silently substituting one. Only an absent count defaults to one; supplied counts must be integers from 1 to 100.
- Centralize grant admission in game-free policy, bound player/item selectors to 1..128 UTF-8 bytes without controls, and reject malformed fields before lookup or queueing. Advertise the same strict schema.
- Preserve valid grants, legacy real-by-default behavior, fresh write gating, queued cancellation and callback auditing. Internal audit IDs still provide no caller replay protection; durable identity/session migration and live acceptance remain separate work.

## 0.32.0 (pure bounded compound geometry previews)

- Extend building_plan with named primitive unions using the existing construction compiler, without live source/world context. Keep primitive inputs/responses unchanged and material labels unvalidated.
- Report canonical geometry, complete footprint, unique cells and separate candidate/contribution/omission/overlap counts. Preserve all candidate, part, extent, unique-cell and brick limits, strict branch admission and deterministic lexical ownership.
- Pure layouts grant no execution or source authority. Live material, rights, persistence, rendering, traversal, props, workshop usability and recovery remain separate deferred checks.

## Server 0.31.1 / companion 0.12.1 (strict write-flag expiry)

- Refuse shared write authorization when the flag is exactly 30 minutes old, matching the documented less-than-30-minute requirement. The previous comparison incorrectly admitted the exact boundary.
- Cover one nanosecond before expiry, exact expiry and one nanosecond after, including real writes, non-reporting dry requests and safe report-only dry calls. Keep flag metadata, confirmation and all other gates unchanged.
- Correct the marker reply-loss test to acquire its preview through a fresh production frame after an explicitly stale cached frame; keep production freshness and timeout limits unchanged. Give marker test folders a counter suffix so equal clock readings cannot collide.
- Pin fixed read-only probes to the new server/companion source versions and explicitly refuse prior capability versions. No installed runtime or client use changed; connected/live acceptance remains deferred.

## 0.31.0 (deliberate native save requests; live acceptance deferred)

- Add dry-by-default server_save through published World::save on the game thread, with exact preview session, empty character roster, fresh write gate and confirmation. Recheck admission after synced durable intent.
- Bound identities to 64 exclusive save-prefixed files, retain same-session replay without redispatch, refuse reopened/stale identities and audit failures, and cancel queued timeouts. Keep native errors and lost outcomes explicit; never automatically retry or remove identities.
- Separate native request acceptance from completed disk save and causal content proof. No backup, force, save holds, native undo, shutdown or client control; live save/restart/full-content acceptance remains deferred.

## 0.30.0 (bounded solid gable-roof geometry; live acceptance deferred)

- Add symmetric solid gable roofs to the existing pure preview and primitive/composite compiler. Equal paired half-metre column runs ascend toward a flat central ridge along local z; the attic volume is explicitly filled.
- Preserve candidate/world/unique-cell/brick/composite limits, source-voxel authority, canonical identity and all write/recovery gates. Report outside-gable omissions separately; omission never erases occupants.
- Independent paired-column/count, rotation, invalid-slope, preview and construction-admission checks cover source behavior. No game-native roof, shelter, support, rendering or workshop-usability claim; live placement/undo/reload remain deferred.

## 0.29.0 (bounded solid staircase geometry; live acceptance deferred)

- Add staircase geometry to the existing pure preview and primitive/composite construction compiler. Fixed half-metre rises and equal positive treads ascend local +z, using existing quarter-turn rotation and source-voxel authority.
- Preserve pre-omission candidate limits, world bounds, compact composite extents, stable keys/canonical identity, source-overlap refusal and every write/brick/recovery gate. Distinguish above-step voids from doorway openings; no omission erases occupants.
- Independent stepped-column/count tests, rotations, invalid ratios, preview metadata and primitive/composite conversion checks pass. Geometry is not a native stair blueprint or verified traversal. Placement/rendering/undo/reload and workshop usability remain deferred.

## Development tooling (independent shutdown-exit evidence)

- Correct the FakeHost marker acceptance test to reconcile a permitted started-timeout through retained status, native markers and exact journal data. Add deterministic reply-loss coverage without retrying or changing production deadlines.
- Add a read-only Windows process observer with PID, creation-FILETIME and executable matching on a pinned handle. Report readiness, exit/code or bounded timeout without starting/stopping a target or claiming save/causal completion.
- Test normal/nonzero/259 exits, timeout, identity mismatch and invalid arguments using disposable child processes only. Keep game-client access and live shutdown/restart acceptance deferred.

## 0.28.1 (bounded native shutdown/save observations; live acceptance pending)

- Add optional published lifecycle callbacks with independent world-load generation binding. Audit at most one shutdown-start and one later save event for an attempted operation; ignore early, duplicate and stale-generation events.
- Preserve the original native request outcome and explicit process/final-save/causal verification limits. Callback availability and journal failures stay visible. Passive audit callbacks do not touch the World or retry shutdown.
- Require room for intent, outcome and two possible event records before dispatch, retaining the existing shared per-record journal limit. Offline policy/native tests cover ordering, lifecycle invalidation, deduplication, audit failure and insufficient headroom.

## 0.28.0 (native dedicated-server shutdown; live acceptance pending)

- Add a dry-by-default, session-bound `server_shutdown` request through published `World::stop`, scheduled on the game thread. Recheck the empty SDK player-character roster, write gate and deadline after durable intent; a roster is not a connection lock.
- Bound and sync exclusive operation identities and journal intent/outcome. Cancel pre-start queued timeouts, retain uncertain native outcomes and prevent replay or a second native attempt in the active session. No automatic restart, save hold, process-kill wrapper or client logout is added.
- Keep native acceptance separate from process exit and final-save evidence. Game-free and FakeHost tests cover admission, cancellation, actual stop request, replay, session changes, durable identity, capacity and failure paths. Fixed read-only probes never invoke shutdown.

## Companion 0.12.0 (recipe lookup by archive ID; isolated comparison verified)

- The isolated 0.27.0/0.12.0 pair passes a matched cold deployment and live comparison: two observed archive IDs return complete unchanged output-query metadata at Menu and during a healthy connected session. Zero/max-ID queries and invalid selectors are exercised. Clean logout, restored settings, complete captures, archived ledgers and eight installed binary hashes pass. Live duplicate/zero/max-ID records, actual selected-UI applicability and public-namespace installation remain unverified.
- Extend `client_recipes` with exactly one output item name or bounded integer archive recipe ID. Preserve zero/max IDs, all raw metadata, duplicate matches and truncation. Report ID ambiguity explicitly; existing output-item responses remain unchanged.
- Keep archive reads on the existing bounded frame queue. Argument and parser tests first fail, then pass; observed IDs do not establish current UI selection, effective definitions, eligibility or a crafting executor.
- Cover delayed native clock application: later status can observe the target while the original receipt remains uncertain, with no redispatch or extra journal record.

## 0.27.0 (forward native clock requests; bounded isolated fixture observed)

- The isolated 0.27.0/0.11.0 pair now passes a bounded clock fixture after a verified cold archive and server-only replacement. One native request was accepted; immediate read-back retained the old hour and correctly stayed `readback_unverified`. A later independent observation reached the requested hour, without promoting the retained receipt. Dry preview, missing-gate and invalid-target refusals, same-request replay, altered-request refusal and operation status pass. A healthy connected client separately rendered daylight consistent with its native daytime observation; connected preview was refused. Clean logout, restored settings, complete voxel captures, archived ledgers and installed binary hashes pass. This does not verify a before/after lighting transition, elapsed world progression, unrelated gameplay effects or public-namespace installation.
- Add `clock_advance` with a bounded integer minute-of-day target, dry preview, active-session and recent-clock checks, an empty SDK player-character roster and the existing explicit write gate. Targets must be one to twelve observed game hours forward at paused or normal speed. The roster is not a connection lock; operators must coordinate access.
- Schedule the published native setter on the game thread. Cancel queued timeout work and recheck admission after durable intent. Exclusive bounded operation identities and synced intent/outcome journals prevent replay, including uncertain native results and journal failures.
- Report near-target read-back separately from native acceptance. Missing, mismatched or other-session observations remain unverified. Forward progression has no undo; returning to an earlier hour advances another cycle. Clock speed remains read-only.
- Derive probe test versions from package manifests, while keeping production compatibility pins explicit. Clock writes are excluded from fixed read-only probes. Source verification does not establish live gameplay or rendered lighting.

## 0.26.0 / Companion 0.11.0 (ABI 12 qualified; research cold readiness verified)

- Pin official alpha.16 revision 5436ffdd, ABI 12.0, in an isolated SDK checkout. Adapt published typed GUID boundaries while preserving MCP file-order hexadecimal identities, metre-valued marker behavior and explicit nanosecond clock tests. Update native test fixtures and compiled framework provenance.
- Port the existing item placed-template metadata extension and retain absent/present identity, other metadata and PIDE independence checks. Keep generated Luau definitions consistent with the public SDK field. No native layout, function-table slot, game offset or semantic client executor is added.
- Serialize the SDK's two shared-directory compile fixtures through a scoped nextest group after reproducing their race. Strengthen prompt-target metadata coverage with a mixed-byte native GUID fixture and exact file-order output.
- Preserve the previously qualified ABI 11.30 rollback pair. Matching development artifact provenance is documented separately from its absent release signatures. Native lifecycle/crash readiness remains a roadmap gap; an answering HTTP listener alone does not prove fresh UI frames.
- Source qualification passes 215 Rust tests, 35 Python tests, Clippy, formatting and both release builds. The patched SDK full check passes 4,050 main tests, both serialized compile fixtures, full/stable doctests, documentation/link checks and cargo-deny; nine upstream main tests remain skipped. The research pair passes matched cold readiness and preservation after a verified 47-file archive and eight rollback copies. Connected gameplay and public-namespace installation remain unverified.
- Fix fixed-probe discovery and capability checks still pinned to the previous mod versions/framework. Current-version fixtures first fail, then pass; prior approved ABI/version values remain explicitly refused. All 21 server and nine companion fixed checks pass live on the cold pair, retaining an incomplete semantic verdict. Probes remain read-only and retain no raw identities.
- A bounded connected-client fixture verifies healthy fresh frames, complete recipe identity preservation, scoped Flags 1/0 versus unknown, independent target/item/template identity and rendered prompt text. Temporary pacification restores exactly; logout and complete capture/ledger/binary preservation pass. Client screen-host template/position remain unavailable, so correspondence is unproven. Broader ABI 12 writes, UI eligibility and public-namespace installation remain unverified.

## Companion 0.10.0 (native archive recipe IDs; isolated identity fixture verified)

- Add `recipe_id` from the published RecipeLayout.id field for bounded selected archive recipe rows. Preserve raw zero and full u32 values, existing row limits and archive/effective-definition uncertainty. The value is read from the record rather than derived from its name or GUID.
- A record-ID expectation first failed, then passed. Synthetic mismatched-GUID IDs, zero/max values, capped output and invalid field spans pass; fixed probes reject malformed IDs while retaining no raw recipe data. Isolated runtime 0.25.0/0.10.0 returns two direct record IDs that preserve the complete prior archive response and feed scoped live knowledge observations. Full-health checks, clean logout and complete preservation pass. UI selection/eligibility, modified definitions, live zero/max IDs and public-namespace installation remain unverified. No crafting executor or eligibility claim is added.


## 0.25.0 (native scoped knowledge reads; isolated player fixture verified)

- Add read-only `knowledge_read` for one to eight unique nonzero native IDs. Omit player for world queries, or select exactly one connected player by SDK-compatible name/exact account key. All native reads run on the scheduled server game thread; responses preserve requested order, raw type/scope/value/expected flags, and explicit unknown/null observations.
- Observed zero differs from unavailable; player queries can report shared world values. No grants, revocations, quest completion, progression mutation or crafting eligibility claim is added. Fake native fixtures verify scope and zero/flags/counter boundaries without grants; strict admission and fixed read probes cover shape/refusals. Isolated runtime 0.25.0/0.9.0 distinguishes raw player Flags 1/0 from unknown/null and wrong-scope world reads in a healthy fixture. Complete preservation and clean logout pass. Recipe UI eligibility/flag semantics, observed world bool/counter values, persistence and public-namespace installation remain unverified.


## 0.24.1 (exact prompt-target item; isolated identity fixture verified)

- Include nullable native `item_id` for the exact observed prompt target through the published UsedItem getter on the scheduled game thread. Preserve observed zero separately from unavailable metadata; no spatial query, archive interpretation or effective-definition claim is added. State the template GUID's file-order hexadecimal encoding explicitly.
- A target-item expectation first failed, then passed. Fake native component fixtures cover positive, zero and absent item values while preserving prompt-versus-combat selection and no-target/unknown boundaries. Isolated runtime 0.24.1/0.9.0 agrees across direct target ID, independent entity query and archive/item placed-template observations. Full-health checks, clean logout and complete preservation pass. Live zero/absent item values, non-altar targets and public-namespace installation remain unverified.


## Companion 0.9.0 (localized interaction verb; isolated English fixture verified)

- Preserve the native prompt verb ID and add bounded localized verb text through the published client localization API on the frame callback. Missing localization stays null/unavailable; present empty text remains observed. Report truncation, requested language and the documented English fallback source.
- A native-frame expectation first failed, then passed. Fake frames and bounded UTF-8/missing-text tests preserve explicit observation limits. Text observation adds no interaction, menu, movement or camera executor. Isolated runtime 0.24.0/0.9.0 matches one rendered English altar prompt across two fresh healthy-client observations. Complete preservation, clean logout and restored settings pass. Live alternate-language/fallback behavior, other prompt types and public-namespace installation remain unverified.


## 0.24.0 (server prompt target; isolated altar fixture verified)

- Add read-only `player_interaction_target` for exactly one connected player by SDK-compatible name or exact account key. Read the published native prompt target on the game thread; return session-only server entity and nullable position/template metadata. Missing reader/read failure remains unknown, separately from no target.
- No terrain hit, camera raycast, collision/contact proof, client entity mapping or action executor is claimed. Strict arguments reject write fields and arbitrary entity selectors. Source tests distinguish combat and prompt slots and preserve unavailable metadata. Isolated runtime 0.24.0/0.8.0 returns a nonempty target agreeing with a visible Flame Altar and separately scoped client host position/template. Full health, logout, restored settings and complete capture/ledger/binary preservation pass. Document that native prop discovery intentionally excludes flame altars and creatures/survivors; the altar identity chain agrees through entity and item/template observations despite an empty prop query. Other target types, live no-target behavior and public-namespace installation remain unverified.


## 0.23.2 (foundation stop metadata; isolated fixture verified)

- Include `terrain_stop.material` and `terrain_stop.fill` in admitted foundation previews, selected from the same copied capture as the existing stop-cell coordinate. Preserve partial fill, existing Air-only plan geometry and false rendered-contact/terrain-overwrite flags. No surface-height estimator or relaxed write admission.
- A metadata expectation first failed, then passed; partial/full-fill fixtures retain their exact values and unchanged plans. Isolated runtime is server 0.23.2/companion 0.8.0. A partial-terrain fixture matches the complete capture and independent native sample, with unchanged plan geometry. Complete prior captures, operation journals and protected binaries remain preserved; rendered surface contact and public-namespace installation remain unverified.

## 0.23.1 (site selector consistency; isolated fixture verified)

- Match building_access player names with the pinned SDK's case-insensitive lowercase comparison, while keeping account-key matching exact. Preserve exact-one admission: case-variant duplicate names and name/account collisions refuse rather than selecting one actor.
- A Unicode/case regression first failed, then passed with explicit ambiguous-roster and exact account-key checks. Isolated runtime is server 0.23.1/companion 0.8.0. Account key, exact name and changed-case name returned identical actor/rule observations in a healthy sole-player fixture. Live ambiguous/Unicode rosters and public-namespace installation remain unverified.

## 0.23.0 (site access observations; isolated fixture verified)

- Add read-only `building_access` for explicit ordered boxes with each side greater than zero and at most 64 metres. Optional player name/account key must match exactly one connected actor. Native build-rights and persistence reads run on the scheduled game thread; omitted player leaves rights unrequested.
- Preserve unavailable/unchecked results as unknown, independently from observed refusal or false persistence. This is the native rule for the caller's box, not an item's full footprint, collision/contact test, save/reload proof or write authorization. No world mutation or new write gate is introduced.
- Argument/result tests and bounded isolated persistence/actor-rights fixtures pass on server 0.23.0/companion 0.8.0. Missing actor and invalid-box/write-field refusals pass. Construction dry-preview comparison refused an unloaded source region and remains unverified. Full captures, operation ledgers and installed hashes remain preserved after logout; public-namespace installation is unverified.

## Companion 0.8.0 (native map marker submission verified)

- Add `client_map_markers`: one transient mod-owned yellow annotation through published `Ui::set_markers`, queued on the native frame callback. Empty markers clear; no arguments read the submitted cache. Personal/saved markers and server world state are outside its scope.
- Strict bounded arguments, dry preview, observed UI context/revision, fresh companion-folder write flag, exact legacy confirmation and durable correlated intent/outcome guard real calls. Pending timeouts cancel before execution. Observed leave attempts empty-set cleanup once, even after flag removal; uncertain cleanup is never automatically retried.
- All 201 Rust/32 Python tests, Clippy, formatting and release builds pass in the research source. Isolated companion 0.8.0 accepted show, clear, show again and frame-driven logout cleanup with eight audit records. Rendered visibility remains unverified: fallback map shortcuts did not open the map. Cached/native acceptance never claims rendered read-back. Public-namespace installation remains unverified.

## 0.22.0 (native health restoration; isolated fixture verified)

- Add `player_restore_health`: dry-by-default administrative restoration of the sole matching living player to its observed maximum. Preview binds session, actor, lifecycle generation and health; real calls require the existing fresh flag/confirmation. No damage, revive, item consumption, invulnerability, undo or replay protection.
- Sync correlated bounded intent before mutation and outcome after independent native read-back. Recheck actor context and gate immediately before SDK submission. Queued timeout cancels before execution; started outcomes require observation before another request.
- The isolated research pair runs server 0.22.0/companion 0.7.0. A bounded living-player fixture restored native health from 17/100 to 100/100, observed a full rendered bar, appended exactly two correlated records and removed the flag. Full world baselines and prior operation ledgers remained exact after logout. Broader gameplay safety and public-namespace installation remain unverified.


## 0.21.3

- Correct server `capabilities.compiled_framework_revision` to the pinned official `acd5550c607a38c983433f9807cc95564244fe0b`. The previous response incorrectly retained an earlier revision. Compiled provenance is not live loader verification.
- Fixed read probes now require the expected framework revision, ABI and package version. Mutation regressions reject incorrect values; the stricter probe reproduced the installed 0.21.2 mismatch.


## 0.21.2 (unique grant recipient fix; source verified, not deployed)

- Refuse `give_item` selectors matching multiple connected players instead of selecting the first roster entry. Existing name/account-key matching remains; duplicate names and a player's name matching another account key are ambiguous. Refusal precedes grant intent and native submission. Numeric account selectors are accepted only when uniquely matching.
- A game-free first-match regression first failed, then passed with exact-one admission. Fake-host checks cover duplicate names, name/key collisions, uniquely selected accounts and existing timeout/journal behavior. All 188 Rust/31 Python tests, Clippy, formatting and both releases pass. Runtime remains server 0.21.0/companion 0.6.1 with writes disabled; source 0.21.2 is not installed or live-verified.


## 0.21.1 (queued grant fix; source verified, not deployed)

- Fix `give_item` admission after queued lookup. Resolve the actor and dispatch on the same scheduled game-thread handler; recheck fresh flag/confirmation immediately before SDK submission. A revoked-flag regression first reproduced native dispatch and now refuses it. Pending handlers cancel after the five-second wait; started work reports an unknown outcome.
- Sync bounded correlated intent before submission and record native outcomes from the callback even after the HTTP receiver times out. Journal failure before intent prevents dispatch; outcome failure reports possible commitment. Audit request IDs do not provide replay protection. Removing the flag does not cancel an already submitted SDK request, and a callback lost to mod shutdown can leave intent unresolved.
- All 187 Rust/31 Python tests, Clippy, formatting and both release builds pass. Fake-host tests cover dry behavior, blocked intent journals, post-timeout callback journaling and cancellation before a late tick. No live grant or deployment occurred; installed server remains 0.21.0 with writes disabled. Native acceptance of this fix remains pending.


## Companion 0.7.0 (source verified; live deployment pending)

- Add `requirement_observations` to returned recipe rows: ordered declared `required_prop_ids` and `requires_shelter`, through canonical reflected recipe/array/typedef metadata. At most 32 prop entries; spans, strides, counts and boolean bytes are admitted before reads. Empty lists, false flags, zeros and duplicate IDs are preserved. Prop type IDs are not item/entity selectors; full requirements and current eligibility stay unverified.
- Share the existing recipe reflection helpers between category and requirement readers. Read requirements only for returned matches. All 184 Rust/31 Python tests, Clippy, formatting and both release builds pass. Installed isolated pair stays server 0.21.0/companion 0.6.1. This source increment is not deployed or live-verified; neutral namespace binaries remain untested live.


## 0.21.0 (isolated native clock read verified)

- Extend `status` with a published server-clock observation: hours, day/night, day/night boundary hours, configured durations in seconds and current speed. Unavailable clocks remain explicit and distinct from midnight. Reads run on the existing scheduled server game thread; rendered lighting is unverified.
- All 180 Rust/31 Python tests, Clippy, formatting and both release builds pass. Isolated server-only deployment passes nineteen server/nine companion probes, a live daytime clock read and exact 6,144-cell/ledger preservation after a verified 44-file cold archive. Companion remains 0.6.1; official loader/framework unchanged. The carried-forward prop query returned empty as before, so nonempty template/label row output remains unverified. Neutral namespace binaries are not live-installed.


## Companion 0.6.1 (isolated category membership verified)

- Extend `client_recipes` ingredient observations with ordered vanilla recipe-category members, item IDs, raw scaling values/bits, finite status, total count and explicit truncation. Resolve canonical reflected registry/field/array types, including typedef IDs. Admit at most 4,096 category rows and 1,024 members per row; decode only the first 32 members for each returned ingredient. Missing/zero categories remain explicit. Scaling interpretation, UI selection, effective costs and eligibility remain unverified.
- Synthetic tests cover membership/order, caps before decoding, malformed spans, duplicate category IDs, canonical qualified types and typedef resolution. The first 0.6.0 isolated recipe probe failed on a typedef; its failure is retained. Corrected 0.6.1 passes all 179 Rust/30 Python tests, Clippy, formatting and both releases. Isolated companion-only deployment passes all 27 fixed client/server probes, fresh native Menu, six Workbench category member ID lookups and exact 6,144-cell/ledger preservation after a verified cold archive. Neutral publication binaries are not live-installed.


## Companion 0.5.2 (isolated read probes verified)

- Report `cursor_gap` when a client selection-event cursor predates history cleared on observed session exit, including before new events arrive. Preserve monotonic sequences, session generation and overwrite-only `dropped_history`; a caught-up cursor remains gap-free. No native executor or gameplay write is added.
- A regression first reproduced the missing gap, then passed. Isolated companion-only deployment passes nine companion and eighteen server read probes, fresh native Menu and exact 6,144-cell/ledger preservation after a verified cold archive. Live join/logout acceptance remains pending; the neutral publication namespace has not been live-installed.


## 0.20.0 / companion 0.5.1

- Add bounded prop template and nullable label-presence observations through published APIs, enriching only the returned 500 rows. Preserve session-only identity and unknown label text/other held state. This read increment is not live-installed and does not establish safe removal or undo.
- Add optional bounded Air-only foundation preview to building_survey. The isolated 0.19.0 preview matches an independent two-cell oracle, refusing unknown nested fields and excessive depth. Terrain fill is not interpreted as rendered surface height; contact, collision and support remain unverified.
- Add narrow gated enemy_pacification reads/changes, durable intent/outcome and explicit expected/session guards. Isolated boolean change/restoration and read-back pass; no invulnerability, automatic restart restoration or full preset-restoration claim.
- Pin official acd5550c, ABI 11.30, containing the local held-input sampler fix; retain the SDK-only metadata extension. No native menu/movement executor is introduced.
- Extend bounded native recipe reads with separate typed item/category branches; fix bare-name resolution using the canonical reflected type hash after retaining the failed isolated 0.5.0 probe. All nine companion probes pass with 0.5.1, but membership, eligibility and feasibility remain unknown.
- Pass 173 Rust tests, thirty Python tests, Clippy, formatting and release builds. Latest isolated research pair is server 0.19.0/companion 0.5.1; public namespace live validation remains pending.

## 0.16.0

- Add nullable native health, stamina and mana to existing player rows. Preserve raw SDK integer values, observed zero and independent unavailable pools; UI unit conversion remains unverified.
- Add neutral original branding, setup, tool reference, architecture, research boundaries and contribution documentation.
- Pass 146 Rust tests, 30 Python tests, Clippy, formatting and both release builds in source and sanitized publication. Isolated meter reads passed. The final write-gate correction also passed refusal-only research checks after a verified cold archive; no valid inventory grant was performed.
- Fix `give_item` write-flag admission to use the common metadata validator: directory flags and future timestamps are refused. A regression first reproduced directory admission, then passed with the fix. This changes write admission only. Live invalid-flag refusals preserved all 4096 baseline cells and ledger bytes; no inventory grant was performed.

## 0.15.0

- Add dry-by-default `player_reposition`, one bounded server administrative teleport request for a sole connected living actor. Bind preview to exact current server actor/session/generation/position and target; require fresh write gates again before dispatch. The 16 m gate limits the requested point, not the engine's best-spawn correction.
- Synchronize bounded durable identity/intent and journal before a retained once marker and one native request attempt. Canonical same-ID replay does not redispatch; altered requests and stale durable identities refuse. Submission stays `issued_unknown`, not an arrival receipt. Returning needs another separately gated request; no automatic retry, chain, native undo, walking, camera or menu executor.
- Pass 143 Rust tests, 30 Python tests, Clippy, formatting and release build. Correct an outdated tool-advertisement fixture; the read-only validator advertises the new write without ever invoking it. Independent scoped security review found no blocking issue, without asserting complete safety.
- Validate one isolated two-command fixture through six explicit phases, independent rendered/position observations, replay with unchanged journals, exact 4,096-cell construction preservation and cold-reload stale refusal after a verified 40-file archive. Observed positions differed from requested points. A zero-player old-preview/new-identity refusal establishes actor refusal only. Exact arrival, precise return, arrival causality and complete player-state recovery remain unverified.

## 0.14.0 and companion 0.3.0

- Pin official development revision `acd5550c607a38c983433f9807cc95564244fe0b`, ABI 11.30, in a dedicated sibling checkout. Retire the private client-event ABI from the active build and retain only a three-file existing-field SDK metadata extension with no native ABI change.
- Update two registry resource lookups to the current typed API. Report compiled framework revision/ABI separately from live acceptance and add nullable native screen-host template observations. Fake-host tests cover absent, present and cleared templates without inventing a screen-change event.
- Pass 133 Rust tests, 30 Python tests, Clippy, formatting and both release builds. The SDK's prescribed nextest run passed 538 tests. Matched isolated installation followed a verified 38-file cold archive and rollback hashes for all eight replaced binaries.
- Pass eighteen server and nine companion read probes with broader acceptance incomplete. Preserve the exact 4,096-cell shell baseline and stale old operation. Fresh Menu/InGame observations passed; null held input, a non-null live host template and named event delivery remain unverified. The interaction fixture was stopped after an enemy attack; logout completed and writes stayed disabled.
- Keep input/control APIs observational. No vanilla movement, camera or menu actuator, current tab, passive recipe selection, safe prop lifecycle or functional workshop completion is claimed.

## 0.13.0

- Extend read-only `item_info` with distinct observed placed-template, known-no-template and unavailable states. Preserve existing metadata and return explicit null metadata when the runtime definition is unavailable. Template GUIDs use lowercase file-byte-order hexadecimal encoding.
- Expose an existing native ABI field through the pinned compatibility SDK; no ABI, loader or companion change. The combined 23-file patch reproduces against the exact pinned base while retaining the original 20 event-extension files unchanged.
- Pass 132 Rust tests, 28 Python tests, Clippy, formatting and release build. The isolated research-ID deployment passed all 18 settled read-only probes with broader acceptance incomplete. The immediate startup run passed 17 and failed one construction-readiness check; listener availability alone is not readiness.
- Keep functional prop identity, full placement bounds, state-aware recovery and workshop usability unverified. A nonzero template mapping does not authorize spawning.

## 0.12.0

- Extend `building_apply_plan` with up to eight absolute-position primitive parts. Bound the global admitted footprint to 16 m per axis and the candidate sum to 1024 before cell expansion. Union/deduplication assigns overlaps to the first lexical part ID and preserves stable canonical requests under part reordering. Keep the existing 256-cell, eight-brick and 64-cell-per-brick limits; pure `building_plan` is unchanged.
- Validate one six-cell compound union from eight raw part cells and two overlaps. Sixteen isolated proof checks covered independent whole-capture verification, rendered appearance, reordered replay without another transaction, renamed-part refusal and exact undo. Post-undo cold reload matched all 512 baseline cells; stale accept/undo, old-preview and same-identity replay were refused without changing ledger hashes.
- Pass 131 Rust tests, 27 Python tests, Clippy, formatting and release build. All 18 fixed read-only probes passed with broader acceptance still incomplete. Applied-compound persistence, larger modules and workshop functionality remain unverified.

## 0.11.0

- Add gated, dry-by-default `building_accept`. Require a current Applied transaction, exact captures, the sole original actor, rights, persistence and a healthy journal. Sync the acceptance record before releasing placement exclusivity; retain the typed session-only undo receipt. Duplicate acceptance writes no additional record.
- Require strict expected postimages for accepted undo, including one-cell receipts, and block undo while another operation is unsettled. Queued undo revokes acceptance/settlement until exact baseline verification.
- Pass 125 Rust tests, 27 Python tests, Clippy, formatting and release build. The isolated research-ID installation passed all 18 settled read-only probes; one isolated acceptance/layering fixture and its post-undo cold reload passed.
- Validate a five-cell frame plus one-cell gap filler in six phases, with exact 512-cell capture comparisons throughout. Acceptance/replay guards, wrong-order undo refusal without losing the receipt, reverse-order undo and rendered gap closure/reopening/disappearance passed.
- Verify post-undo cold reload against all 512 baseline cells. Both native receipts stayed stale, old session replay was refused, no new reservation appeared, ledger hashes stayed unchanged and the session flag was absent. This is not persistence-while-applied or stateful recovery evidence.
- Record narrow rendered 0.10 observations without claiming incremental gap fill, collision/traversal safety, stateful recovery or workshop completion.

## 0.10.0

- Expand bounded administrative plans to 256 cells, eight aligned bricks and 64 cells per brick. Preserve exact per-brick captures, all-brick verification, guarded undo and existing session/replay/journal rules. One native receipt does not imply atomicity or resumable jobs.
- A separate 0.9.1 floor experiment verified persistence while applied: all 512 captured cells matched after cold reload. Preserve stale native IDs and the applied fixture.
- Fix parallel test temporary-directory collisions with a per-process nonce.
- Pass 122 Rust tests, 25 Python tests, Clippy, formatting and release build. The isolated installation passed 18 settled read-only probes; the 32-cell two-brick proof passed placement/read-back/replay/undo with exact captures; its final post-undo cold reload passed with all 1024 baseline cells equal.

## 0.9.1

- Add `building_apply_plan` for floor, wall and doorway geometry bounded to 64 requested cells in one aligned voxel brick. Preserve logical element keys, exact source material/fill, one native transaction and full-brick verification.
- Require an exact expected whole-voxel postimage before multi-cell undo. This mitigates a framework application-window foreign-edit gap without claiming concurrent safety, compare-and-swap or atomicity.
- Retain the single-cell recovery path, unchanged-props/persistence checks and native undo conflict guard. The 16-cell floor live proof and post-undo cold reload passed.

## 0.8.0

- Add gated `building_place_cell`, `building_operation` and `building_undo_cell` with session-bound dry previews, durable intent, replay tombstones, typed native receipts, exact capture verification and idempotent undo requests.
- Validate one isolated cell placement/read-back/undo and cold reload of all 512 baseline voxels. This proves the reverted fixture, not applied-structure persistence or stateful object recovery.

## 0.7.0

- Add read-only `building_survey`, bounded to 8 m per side and 4096 complete cells, with exact runtime material/fill, whole-box persistence and optional connected-player build rights. Gate arguments, allocation, full-cell bounds and selectors before scheduling onto the game thread. Unloaded and unavailable states remain explicit.
- Keep construction execution, material mapping, collision, survival accounting and recovery acceptance open. Surveying does not authorize writes.

## Research snapshot

- Server 0.6.0 provides fifteen tools, including bounded inspection, curated evidence, pure recipe/geometry previews and gated `give_item`.
- Optional companion 0.2.0 provides six native observation tools and bounded named client-event history through a separate local framework extension.
- Transport remains loopback-only. Write confirmation, fresh flag admission, game-thread scheduling and journaling remain required.
- This publication uses neutral `enshrouded.mcp` and `enshrouded.mcp-client` mod IDs and corresponding install folders. Earlier private installations must not be left active as duplicate mods.
- Administrative construction now has a narrow reversible cell proof and bounded plan implementation. Larger bounded plans, reusable modules, stateful recovery and the advanced workshop benchmark remain open.

Detailed private deployment history is deliberately excluded. Public semantic evidence and limitations are recorded in `plans/research/live-validation.md`.
