# 0173: Distinguish lost reply channels from elapsed game-thread waits

Date: 2026-11-20. Status: source qualified; no native deployment or fault timing acceptance.

The SDK 12.16 source qualification in decision 0165 proves old-game task discard in its published fake host. The default bridge previously mapped both std channel disconnection and elapsed receive time to a message saying the game thread did not answer within five seconds, with an unsupported world-loaded hint. A disconnected channel need not have waited five seconds, and channel state alone does not establish a world/lifecycle cause or whether a write ran.

Extract the unchanged receive behavior into game-free policy::game_reply::receive, leaving Scheduler::next_tick, its one closure, the five-second wait and caller result types intact. Real std sync-channel tests first fail against the old mapper. Source server 0.51.4 now reports a disconnected reply channel separately from an elapsed reply deadline. Both say execution outcome is unknown and direct write recovery to retained status/journal inspection. There is no producer cancellation, native status reconstruction, scheduling change, new retry or write authority.

Four behavior cases distinguish a dropped sender from a live empty producer, preserve both successful values and buffered game refusals after sender drop, and prove a producer can still execute after the receive deadline and receiver drop. This last case is a channel/producer discriminator, not a native game-write test. The bridge invokes the tested helper; actual native interruption causes and timing remain unqualified. No diagnostic is a cancellation acknowledgement or permission to repeat a write.

Default SDK 6a151a95 / ABI 12.1, Rust 1.99.1, companion 0.22.0 and 54 registered tools remain unchanged. Source discovery/probe expectations and current profile advance to 0.51.4; installed signed server 0.48.0 / companion 0.22.0 and all historical cohorts remain separately described. This patch is not installed or exercised live. It does not modify the separately frozen SDK 12.16 client connection cohort, its helper hashes, candidates or pending approval.


Verification: the default workspace passes 488 Rust tests and 125 Python tests, strict Clippy, formatting, both release builds and offline contracts. The independent lifecycle qualifier passes fourteen debug and optimized cases plus strict lint/format and release build with the added policy module. Native queued interruption and diagnostic timing are not exercised; no installation or game action occurs. The pending exact client test remains on its separately frozen artifacts.
