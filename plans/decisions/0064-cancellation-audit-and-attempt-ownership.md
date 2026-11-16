# 0064: Qualify cancellation audit admission and attempt ownership

Date: 2026-11-16. Status: source contract qualified; native interruption acceptance open.

Existing tests already cover deliberate cancellation and terminal replay. Add one behavior fixture for the missing audit-path refusal and attempted-but-not-yet-observed replay boundary. Reuse the published SDK fake host and existing controller fixture; no runtime abstraction, hook or dependency change is needed.

Submit one real model command after fresh-context preview and write admission. Replace only the owned fixture's journal file with an empty directory. Cancellation must refuse with client command journal must be a regular file. Independent SDK command status remains Pending, retained outcome remains unchanged, and the durable reservation bytes and single native command remain intact. This is a non-file audit-path admission fault before intent append; it does not inject append, partial-write or sync failure.

Restore the exact fixture journal bytes and deliberately request cancellation. The SDK reports Cancelled(ByMod); the reply records one accepted cancellation attempt. The original two journal rows remain a prefix and exactly two cancellation rows are appended. The companion has not polled a terminal native state yet.

To discriminate an actual repeated native cancellation from a cached reply, rearm only the SDK model's command to Pending. Replaying cancellation must leave that independent native status Pending and preserve byte-identical journal/reservation, one command and one durable identity. A reopened controller reports prior_boot_unavailable and refuses cancellation without reconstructing an ID or changing SDK state. Rearming is solely a source-test discriminator; it is not native behavior or a retry policy.

An isolated exact-source copy of public 43695084d883ff8ab071b187a65d31da8906ff1b receives only the new test and a mutation removing the cancel_attempted replay guard. The new fixture fails at the independent SDK status check: Cancelled(ByMod) instead of Pending. The real production guard passes. This proves attempt protection is exercised rather than merely matching a retained JSON flag. The mutation is never applied to working or published production code.

Private/public complete checks pass 373 Rust executions, 71 Python tests, strict Clippy, formatting and both release builds. Production source before the test module, package versions, SDK pins, deadlines, installed DLLs and game state are unchanged. Actual callback stalls, pending native cancellation/disconnect, append/sync failures, post-reservation context races and rendered/persistence effects remain separate open acceptance. The latest published release was rechecked during this increment and remains alpha.17; no provider upgrade is adopted.

## Separate native Menu refusal

The current matched 0.45.0/0.20.2 installation was also observed read-only. UI-frame observation is fresh at Menu while the published copied client snapshot remains Never. These are different observation channels; a resolved snapshot handle or fresh UI callback does not establish fresh copied client context. One explicit dry Map-open request refuses with the exact fresh/nonzero-session/readable-HUD error. No native command is dispatched or identity added. The complete command inventory, 88 ledger/reservation files and twelve installed hashes remain unchanged; server roster stays empty and all write flags stay absent. Compiled provenance remains the pinned SDK 6a151a95. No game save comparison was performed, so this does not claim full world/persistence or native cancellation acceptance.
