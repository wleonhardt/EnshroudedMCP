# 0046: Observe bounded server weather through the published SDK

Date: 2026-11-16. Status: source and native point-read acceptance pass; rendered and broader provider behavior remain open.

Workshop experiments need current server environmental observations distinct from the server clock and client rendering. The pinned official alpha.17 SDK already publishes Load::weather and Weather::at. Wrap one point sample; do not introduce offsets, signatures, a private SDK patch or a forecast executor.

Require x/y/z numbers within existing finite world bounds. Security-relevant admission and dominance usability are game-free policy with unit tests. Queue both the sample and world-key read through the existing Scheduler::next_tick route. An unavailable SDK sample stays unavailable; it does not become clear or a claim about which provider/hosting/thread condition failed. Do not call Weather::fronts: its empty result conflates absence/unavailability and its allocation is not bounded by this adapter.

Preserve finite clear/rain/snow/blizzard weights and their descriptive f64 sum without normalization. Only finite 0..1 weights with at least one positive component receive the SDK strongest-state name; ties follow the SDK's published ordering. This does not require the sum to be one or establish a probability model. Observed all-zero, non-finite and out-of-range samples remain observed but have unusable dominance and a null name. JSON null within the weight object denotes a non-finite published value, and a non-finite sum becomes null. Entire null weights mean unavailable. Keep rendered_weather_verified, shelter_verified, actor_safety_verified and world_mutated false.

The new policy and adapter tests fail before their functions exist. Regressions cover exact bounds, missing/incorrect/extra arguments, unavailable versus clear/zero, four dominant states, SDK ties, unnormalized finite values and malformed floats. SDK sampling is queued explicitly, cannot reply before the fake tick, and never submits a forecast. The fake getter does not itself enforce off-thread refusal; this is not general native thread-guard proof or a full MCP dispatch fixture. Native production-route sampling and rendered comparison remain separate acceptance gates.

Keep the official 6a151a95 SDK, ABI 12.1 and companion 0.20.0 unchanged. The server version becomes 0.42.0. A matched cold deployment must preserve all existing rollback, world and ledger invariants before native acceptance. No weather write or mutable front state is exposed.

Private/public checks pass 363 Rust executions (including 19 repeated shared-module checks), 67 Python tests, strict Clippy, formatting and both release builds. Installed runtime remains 0.41.0/0.20.0 until matched native qualification; provider fidelity, rendered weather and safety/shelter interpretation are not established by model checks.

## Native acceptance

Signed sanitized source 0497e930a08cc8345b015d63c5cc95c3ef8efa4f passes its source checks and exact 169-file publication audits. Matched 0.42.0/0.20.0 native production-route qualification observes six samples at three bounded points. All have usable dominance and agree with independent decoded-weight, sum and SDK tie-order checks; each world key matches a separate current status read. Eleven invalid or mutable selectors refuse. Raw positions, world keys and weights remain private evidence, not publication artifacts.

All 29 fixed server and 13 client probes, existing recipe/workshop correlations, complete world baselines, runtime source closure, 12 installed hashes and immutable ledgers pass. A verified 139-member cold archive and four exact rollback DLLs preserve recovery; eight official framework/signature files are unchanged. One independently observed native shutdown exits 0, adding exactly four journal rows and one marker under two archive aliases while preserving every old prefix and other ledger byte. Exit is not causal final-save proof. Original settings, zero roster, fresh native Menu and absent flags are preserved. No weather write or forecast executor is added; Windows MCP/capture remain off.

Native missing/malformed samples were not forced. Their handling has model coverage only. Rendered weather, actual shelter/hazard interpretation, broader climates and future provider versions remain unverified. A joined rendered fixture is the next correlation gate; point observations do not authorize construction, movement or weather mutation.
