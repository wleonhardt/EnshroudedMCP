# 0139: Qualify independent shroud state and timer observations

Date: 2026-11-20. Status: inactive source consumer verified; no MCP registration or native acceptance.

Traversal and mechanics goals need truthful environmental observations. The pinned SDK already exposes World::shroud_time and World::in_shroud; no SDK upgrade is needed for source qualification. Keep this first consumer in an independent rlib workspace on exact SDK6a151a95/ABI12.1 and Rust1.99.1. Product pins, source versions, installed files and the staged native baseline remain unchanged.

Read the two getters independently on the server game thread. Preserve nullable values, exact signed SDK integer-second pools and known false/zero. Missing/unreadable/unsupported SDK results remain unavailable without an invented cause. Partial field availability remains explicit. A supplied entity ID does not prove a current player, client identity or durable object. The output grants no refill, world mutation, movement, damage-deadline, safe-traversal or rendered-UI authority.

Eight debug/release SDK-fake cases and strict lint/format pass. The initial feature specification fails at the unimplemented observer only after independent getter assertions confirm observed zero and false without entity health. Seven cases then pass on first implementation; a two-host fixture lifetime error is corrected before the final complete run. No runtime gate is weakened and no native failure is diagnosed from these fixtures. Exact reproduction and failed fixture receipts remain private.

See qualification/sdk-shroud/README.md for commands and limits. Next review a strict current-player selector and queued read adapter; a real tool must keep admission/projection in game-free policy. Positive native timer/flag values, displayed countdown agreement and current-context acceptance remain open. No real player is moved into the Shroud, no installed provider is changed and no client lifecycle request occurs.
