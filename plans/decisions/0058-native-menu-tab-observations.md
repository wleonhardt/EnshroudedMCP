# 0058: Preserve menu tab observations separately from HUD screen

Date: 2026-11-16. Status: bounded native acceptance passes; rendered transitions unverified.

Qualify existing supported UI controls without moving the avatar, changing inventory or executing a recipe. Installed server 0.44.0 / companion 0.20.2 remains on official alpha.17 and ABI 12.1. At this fixture checkpoint, candidate 0.45.0 was not installed; its later matched deployment is recorded in decision 0057.

Four separately previewed, once-only commands run in a fresh nonzero client session. Each panel is held for six seconds. All four SDK terminal outcomes are Observed. Copied native snapshot fields report:

| Command | HUD screen | Menu tab |
|---|---|---|
| Open Backpack | Backpack | Backpack |
| Choose PlayerCrafting tab | Backpack | PlayerCrafting |
| Choose Backpack tab | Backpack | Backpack |
| Close screen | InGame | null |

Do not rewrite HUD Backpack into PlayerCrafting or collapse these two fields. The pinned published snapshot implementation carries HUD screen and menu tab independently. SDK Observed and changed getter fields do not independently prove a rendered panel transition, selected recipe, concrete costs, crafting eligibility or a completed craft. The operator reports not watching, so independent rendered confirmation is unavailable. No equivalent rendered retry is automatic.

Complete 19,200 baseline cell observations, an additional 4,096-cell source survey and platform/headroom/prop comparisons pass before and after. Server actor position remains within 0.02 m of its starting observation, with full health. The complete returned inventory response is equal. Twelve installed hashes, old audit prefixes and unrelated ledgers remain unchanged. Original pacification is restored and write flags are absent.

Exactly four server pacification rows, twelve client rows and four client reservations are added. Each command has one intent, submission and terminal row. All four current-boot status bodies agree with command inventory after separating advancing poll-age metadata. Those ages remain stale and monotonic, with polling_finished true and current_context_verified false. The inventory contains 26 identities: these four current-boot records and 22 unavailable prior-boot records. Dynamic status observation is absent from durable audit outcomes. All 35 installed-version server probes and 13 client probes pass.

The first private audit attempted exact equality including live poll age and failed. Its corrected check requires identical immutable bodies and validates the separately increasing age rather than discarding freshness metadata. No native command was repeated and no product bug was inferred from that audit assumption.

No product source changed for this qualification. Current 0.45.0 source retains its preceding private/public 370-Rust and 71-Python qualification, lint, formatting and release builds. This does not install or natively qualify retained shutdown status. Pending-command cancellation, callback stalls, disconnect timing, general tab-target coverage and rendered panel acceptance remain open. No further avatar movement runs automatically.
