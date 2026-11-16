# 0049: Clear command age when the SDK status poll fails

Date: 2026-11-16. Status: source fix; native fault behavior remains unverified.

A successful SDK poll stored native_age_ms. A later failed poll changed native_state to unknown and marked polling finished, but retained that earlier age beside a fresh status_observation. This mixed two different observations. Native age describes the command age sampled by the current successful poll, not a historical value from a prior poll.

Companion 0.20.1 explicitly sets native_age_ms to null on the error branch. Preserve the error, terminal uncertainty, fresh poll timestamp and no-retry identity. Do not erase valid native age from an SDK response whose state is Unavailable; that is still a successful SDK read. Initial/replay receipts, write gates, durable reservations, SDK pin and thread routing remain unchanged apart from the corrected retained result field.

The existing fake-host regression now observes Applied with a numeric native age, then models loss of that SDK identity. It first fails on the retained numeric age. It now checks explicit null in individual status, inventory and replay, with no redispatch or additional terminal journal row. The SDK error is simulated by an unavailable native identity; this does not prove a real disconnect, native status eviction or current-boot frame failure.

A new probe regression first accepts a poll-error row containing stale numeric age. The corrected read-only validator requires an unknown state, a string SDK error and explicit null native age, while saving only aggregate counts. Source probe version/discovery expectations advance with the companion patch version. Native current-boot error acceptance requires a controlled joined fixture; do not force an undocumented SDK fault.

Keep server 0.42.0, official 6a151a95 / ABI 12.1 and the existing installed 0.42.0/0.20.0 pair until a cold matched installation and read-only regressions pass. No native executor or unsupported join/logout is introduced.

## Native regression acceptance

Signed sanitized source e3b3ee9eea4fc43cb70e80a73b6649632616883e passes private/public 363 Rust executions and 71 Python tests, lint, formatting, releases and exact 178-file/history audits. A verified 141-member cold archive and four exact rollback DLLs precede matched installation. Only two companion destinations change; the server DLL and eight official framework/signature files remain unchanged.

One native dedicated shutdown has an independent exact-process observer attached before dispatch and exits 0. Exactly four unique audit rows and one marker under two aliases preserve all old ledger bytes/prefixes. This is not causal final-save proof. One verified fresh-Menu client closes gracefully and relaunches through the OS because the SDK has no process-lifecycle executor; Windows MCP stays off. A shell timestamp conversion guard initially refuses before any close attempt; preserving timestamp text and its bounded native precision resolves the identity comparison. Actual launch/endpoint process identities are reacquired independently.

Matched server 0.42.0 / companion 0.20.1 passes 29 server and 13 client fixed read probes, all 18 durable prior-boot identity/status comparisons, five inventory selector refusals, six Menu dry refusals and eleven archive selector refusals. Existing recipe/workshop/item correlations, full world baselines, runtime source closure, all 12 installed hashes, immutable ledgers and absent flags pass. No client command, join or gameplay write is submitted. Fresh Menu, empty roster and original pacification false remain observed.

The failing native status-poll branch is covered by the model regression only. No actual SDK identity loss, disconnect or current-boot command error is forced. Keep that gate open; transport/read regression acceptance does not replace it.
