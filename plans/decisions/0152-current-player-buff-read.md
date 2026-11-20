# 0152: Current-player active buff read

Expose `player_buffs_read` using the published SDK roster and existing active-buff projection. Resolve a unique current server player on the game thread; never accept an entity or client identifier as a player selector. Validate the selected entity before calling the buff getter.

Require active MCP lifecycle at admission and execution with the same epoch. The epoch is not a native world identifier or a connection lock. Preserve nullable world keys, SDK durations, optional levels, read errors and bounded output. This grants no mutation, client mapping, complete effects or actor-safety authority.

Verification includes strict policy admission, actual dispatcher selection and projection, unavailable versus empty and refreshed state, and queued reload/shutdown refusals while SDK roster and buffs remain independently readable. Existing `entity_buffs_read` is unchanged. Matched native acceptance is pending; earlier0.50.6 native evidence cannot qualify this new tool. No installed files or processes change.

The missing-tool reproduction fails before implementation. All six dispatcher/SDK-fake cases and two policy cases pass afterward. Full source checks pass455 Rust/110 Python, strict Clippy, format, both release builds and offline contracts. The first Python run catches an outdated current-version table row; correcting it makes all110 pass. Earlier failed logs remain retained. The fixed read-only validator now has46 checks, whose synthetic success remains an incomplete verdict. Native player, rendered UI and in-flight interruption acceptance remain open.
