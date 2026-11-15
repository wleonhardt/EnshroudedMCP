# Changelog

## 0.26.0 / Companion 0.11.0 (ABI 12 qualified; research cold readiness verified)

- Pin official alpha.16 revision 5436ffdd, ABI 12.0, in an isolated SDK checkout. Adapt published typed GUID boundaries while preserving MCP file-order hexadecimal identities, metre-valued marker behavior and explicit nanosecond clock tests. Update native test fixtures and compiled framework provenance.
- Port the existing item placed-template metadata extension and retain absent/present identity, other metadata and PIDE independence checks. Keep generated Luau definitions consistent with the public SDK field. No native layout, function-table slot, game offset or semantic client executor is added.
- Serialize the SDK's two shared-directory compile fixtures through a scoped nextest group after reproducing their race. Strengthen prompt-target metadata coverage with a mixed-byte native GUID fixture and exact file-order output.
- Preserve the previously qualified ABI 11.30 rollback pair. Matching development artifact provenance is documented separately from its absent release signatures. Native lifecycle/crash readiness remains a roadmap gap; an answering HTTP listener alone does not prove fresh UI frames.
- Source qualification passes 215 Rust tests, 35 Python tests, Clippy, formatting and both release builds. The patched SDK full check passes 4,050 main tests, both serialized compile fixtures, full/stable doctests, documentation/link checks and cargo-deny; nine upstream main tests remain skipped. The research pair passes matched cold readiness and preservation after a verified 47-file archive and eight rollback copies. Connected gameplay and public-namespace installation remain unverified.
- Fix fixed-probe discovery and capability checks still pinned to the previous mod versions/framework. Current-version fixtures first fail, then pass; prior approved ABI/version values remain explicitly refused. All 21 server and nine companion fixed checks pass live on the cold pair, retaining an incomplete semantic verdict. Probes remain read-only and retain no raw identities.

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
