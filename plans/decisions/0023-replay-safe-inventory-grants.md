# 0023: Replay-safe caller-identified inventory grants

Date: 2026-11-15. Status: implemented; private and sanitized-public source verified; live acceptance deferred.

Legacy give_item's internally generated audit request could not reconcile a lost reply by caller identity. Strict input admission in 0.32.1 fixed malformed quantities but deliberately did not claim replay protection. Migrate the existing tool, rather than add a competing unsafe grant route or a generic job engine. Use the already supported native inventory API and current construction lifecycle epoch.

## Compatibility and request contract

Server 0.33.0 requires operation, player and item. Operation is 1..64 lowercase ASCII letters, digits, underscores or hyphens. Count remains optional/default one and strictly 1..100. Selectors retain bounded UTF-8 admission. Dry now defaults true. Legacy requests without operation fail closed; they never receive a fabricated identity or silently submit. This is an intentional pre-1.0 contract change.

A new dry request resolves one unique current actor and vanilla item and returns a bounded opaque preview binding operation, normalized requested selector/item/count, active world session and observed account/entity/item identities. Real calls require that exact current preview, fresh ALLOW_WRITES and confirm server-write. Keep ambiguous names/name-account collisions refused; do not replace unique selection with a silent first match or require all other players to disconnect. Recheck actor/item/session after intent and write flag/deadline immediately before submission.

## Durable dispatch and reconciliation

Keep at most 64 identities in memory and give-operations/. Prefix exclusive filenames grant- so Windows device names are safe. Sync intent before native dispatch and reserve bounded intent/outcome journal headroom. Failed intent or changed admission blocks the identity without submitting. A queued five-second timeout cancels before execution; a started timeout is unknown and must be reconciled without blind retry.

Same-session, same normalized requested payload returns retained status without resolving another actor or issuing another grant. Changing selector, item or count conflicts. Dry lookup repeats the original payload and needs no write flag. Real replay still needs current write admission and a syntactically valid preview, but only returns retained status. Reopened durable identities report unknown on dry lookup and refuse real dispatch. Old in-memory session identities refuse after WorldLoaded; no entity ID is reconstructed as persistent identity.

Return submission status without waiting for native completion. The published inventory callback retains its own outcome and audit independent of the HTTP reply, including after flag removal or world-generation change. Late records retain their original session and cannot authorize work in a later one. Callback audit failure stays explicit in cached status. Native commit/rollback is reported as given true/false; submitted-but-unsettled is null. Inventory_readback_verified remains false: a native commit report is not independently observed inventory contents. No automatic retry, native undo or survival-cost claim is added. A new identity is a new deliberate request, never a hidden retry after an unresolved outcome.

## Verification and deferred acceptance

Use game-free tests for required identities/default dry, strict fields/quantities, payload/operation/actor/item/session preview changes, capacity, deadline and journal budget. Use published FakeHost scheduling/native inventory callbacks for pending status, lost-acknowledgement reconciliation, exactly one submission under replay, payload conflict, rollback, queued cancellation, reopened IDs, stale previews/sessions, late callback scope and intent/outcome audit failures. Include grant identities in cold archives and verify synthetic exact membership/hashes.

Live acceptance remains deferred under the user's client hold. Before a later matched deployment, archive all old captures, journals and identities. On a disposable isolated fixture, observe baseline inventory independently, preview one bounded grant, submit once, reconcile the same identity, compare exact native inventory state, verify replay/conflict/stale refusal and preserve recovery limits. This administrative grant has no native undo; do not destroy arbitrary inventory as cleanup or claim a geometric capture preserves inventory. Keep old source, runtime versions and missing live acceptance explicit.
