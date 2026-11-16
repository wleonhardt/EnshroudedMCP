# 0049: Clear command age when the SDK status poll fails

Date: 2026-11-16. Status: source fix; native fault behavior remains unverified.

A successful SDK poll stored native_age_ms. A later failed poll changed native_state to unknown and marked polling finished, but retained that earlier age beside a fresh status_observation. This mixed two different observations. Native age describes the command age sampled by the current successful poll, not a historical value from a prior poll.

Companion 0.20.1 explicitly sets native_age_ms to null on the error branch. Preserve the error, terminal uncertainty, fresh poll timestamp and no-retry identity. Do not erase valid native age from an SDK response whose state is Unavailable; that is still a successful SDK read. Initial/replay receipts, write gates, durable reservations, SDK pin and thread routing remain unchanged apart from the corrected retained result field.

The existing fake-host regression now observes Applied with a numeric native age, then models loss of that SDK identity. It first fails on the retained numeric age. It now checks explicit null in individual status, inventory and replay, with no redispatch or additional terminal journal row. The SDK error is simulated by an unavailable native identity; this does not prove a real disconnect, native status eviction or current-boot frame failure.

A new probe regression first accepts a poll-error row containing stale numeric age. The corrected read-only validator requires an unknown state, a string SDK error and explicit null native age, while saving only aggregate counts. Source probe version/discovery expectations advance with the companion patch version. Native current-boot error acceptance requires a controlled joined fixture; do not force an undocumented SDK fault.

Keep server 0.42.0, official 6a151a95 / ABI 12.1 and the existing installed 0.42.0/0.20.0 pair until a cold matched installation and read-only regressions pass. No native executor or unsupported join/logout is introduced.
