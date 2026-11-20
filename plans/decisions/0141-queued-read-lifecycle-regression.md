# 0141: Verify lifecycle changes after a player read is queued

Date: 2026-11-20. Status: deterministic source regression and negative mutation verified; native timing remains unverified.

Decision0140 guards player_shroud at admission and queued execution. Its initial source cases cover shutdown before a call, but do not independently prove the queued epoch check. Add one regression with unchanged, shutdown and reload scenarios. A listener thread submits the actual GameTools request. After observing the pinned SDK scheduler become pending, deliver documented SDK-fake lifecycle events before the server tick. Retained typed data stays available, so a refusal cannot pass merely because a component disappeared.

The isolated fixture asserts an idle queue before the request. Its only task is this read. Scheduler's public Debug pending field is used solely as a bounded enqueue barrier; it is not an executor, liveness, current-actor or native timing observation. The exact SDK remains pinned, and a future SDK change must requalify this diagnostic assumption. No fixed sleep assumes the request reached admission, and no production deadline or gate changes.

The unchanged case must return the expected timer. Shutdown must refuse through the active lifecycle check. Reload must refuse through the admission/execution epoch mismatch. An isolated exact-base mutation removes only that queued epoch comparison; the new regression then fails because it receives an observation. The product runtime prefix and all production source remain unchanged. This is source behavior under a published SDK fake, not evidence of native disconnect/WorldLoaded timing or provider fidelity.

Keep test evidence distinct from runtime acceptance. No version bump, MCP schema, SDK adoption, installed file, process action, client input or native lifecycle request is involved. Full source checks pass: 433 Rust and 109 Python cases, strict lint/format, release builds and source contracts. Exact sanitized audits are required before publication.
