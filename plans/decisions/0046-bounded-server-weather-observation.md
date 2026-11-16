# 0046: Observe bounded server weather through the published SDK

Date: 2026-11-16. Status: source verified; native qualification pending.

Workshop experiments need current server environmental observations distinct from the server clock and client rendering. The pinned official alpha.17 SDK already publishes Load::weather and Weather::at. Wrap one point sample; do not introduce offsets, signatures, a private SDK patch or a forecast executor.

Require x/y/z numbers within existing finite world bounds. Security-relevant admission and dominance usability are game-free policy with unit tests. Queue both the sample and world-key read through the existing Scheduler::next_tick route. An unavailable SDK sample stays unavailable; it does not become clear or a claim about which provider/hosting/thread condition failed. Do not call Weather::fronts: its empty result conflates absence/unavailability and its allocation is not bounded by this adapter.

Preserve finite clear/rain/snow/blizzard weights and their descriptive f64 sum without normalization. Only finite 0..1 weights with at least one positive component receive the SDK strongest-state name; ties follow the SDK's published ordering. This does not require the sum to be one or establish a probability model. Observed all-zero, non-finite and out-of-range samples remain observed but have unusable dominance and a null name. JSON null within the weight object denotes a non-finite published value, and a non-finite sum becomes null. Entire null weights mean unavailable. Keep rendered_weather_verified, shelter_verified, actor_safety_verified and world_mutated false.

The new policy and adapter tests fail before their functions exist. Regressions cover exact bounds, missing/incorrect/extra arguments, unavailable versus clear/zero, four dominant states, SDK ties, unnormalized finite values and malformed floats. SDK sampling is queued explicitly, cannot reply before the fake tick, and never submits a forecast. The fake getter does not itself enforce off-thread refusal; this is not general native thread-guard proof or a full MCP dispatch fixture. Native production-route sampling and rendered comparison remain separate acceptance gates.

Keep the official 6a151a95 SDK, ABI 12.1 and companion 0.20.0 unchanged. The server version becomes 0.42.0. A matched cold deployment must preserve all existing rollback, world and ledger invariants before native acceptance. No weather write or mutable front state is exposed.

Private/public checks pass 363 Rust executions (including 19 repeated shared-module checks), 67 Python tests, strict Clippy, formatting and both release builds. Installed runtime remains 0.41.0/0.20.0 until matched native qualification; provider fidelity, rendered weather and safety/shelter interpretation are not established by model checks.
