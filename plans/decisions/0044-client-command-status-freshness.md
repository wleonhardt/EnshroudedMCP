# 0044: Distinguish copied command status from fresh client evidence

Date: 2026-11-16. Status: source verified; native prior-boot freshness acceptance passes; current-boot polling remains open.

A retained pending/applied result previously had no timestamp for its last SDK poll. If frame callbacks stopped while the listener remained responsive, a status read could repeatedly return the same old result without exposing its age. The SDK native_age_ms is the command age sampled by that poll; it is not the time since MCP observed the status.

Store an in-memory monotonic timestamp when each client-frame command-status poll completes, including error results. Add status_observation to explicit status reads and inventory rows. It reports source, age_ms, stale, polling_finished and current_context_verified=false. More than 1,000 whole milliseconds or an unknown age is stale. Never-polled operations have null age. Prior-boot records have no reconstructed SDK identity and no known polling state. Listener reads only compute elapsed time; they never call the SDK or refresh the timestamp.

Finished polling preserves a historical terminal outcome. Its age can grow stale without changing observed/cancelled/unavailable/unknown into a new result. A recently polled SDK error remains unknown, not success. None establishes present UI, rendering, arrival, crafting or permission to repeat an uncertain action. Initial submission and command-replay receipts retain their old shape. Monotonic timestamps and derived freshness metadata are not persisted or added to journal records, so durable identity and audit semantics stay unchanged.

The behavior regression fails on the missing freshness field before the encoder is implemented. It covers never-polled state, available poll age, forced stale pending status, read-only inventory, historical terminal status without renewed polling/journaling, prior-boot unknowns and an unavailable SDK identity. Policy tests cover unknown/zero/1,000/1,001-millisecond boundaries. Probe validators refuse missing or inconsistent metadata and save only aggregate counts/states, never operation identities, payloads or error strings. Synthetic polls do not establish live callback-stall behavior.

Keep official SDK 6a151a95 / ABI 12.1 and installed 0.41.0/0.19.0 unchanged until a matched cold deployment. The inactive SDK placement proposal and pending upstream-post authorization are unrelated.

Private/public checks pass 360 Rust executions (including 19 repeated shared-module checks), 60 Python tests, strict Clippy, formatting and both release builds. Installed 12 hashes, immutable ledgers, absent flags, fresh native Menu and empty roster are preserved. Source 0.20.0 is not installed; native callback-stall behavior remains unverified.

## Native qualification

Matched server 0.41.0 / companion 0.20.0 verifies 18 prior-boot records with null age, stale true, polling_finished null and current_context_verified false. Individual status reads and repeat inventory agree. Five strict selectors and six Menu dry actions refuse with no client dispatch, reservation or audit changes. Fixed probe summaries contain only 18 unavailable/unpolled/stale counts and prior-boot states, not identities or raw error strings.

All 13 client and 26 server probes, complete world baselines, runtime source closure, 12 installed hashes and immutable ledgers pass. Four exact rollback DLLs and a verified 137-member cold archive preserve recovery. Eight official framework/signature files are unchanged. One independently observed native shutdown exits 0; exactly four new audit rows and one marker under two archive aliases preserve every old prefix and other immutable byte. Exit is not causal final-save proof. Explicit process port environments avoid the previous launch mistake. Windows MCP/capture remain off.

Fresh native Menu does not exercise a current-boot SDK command-status poll. Native positive poll age, callback-stall timing, joined disconnect behavior and rendered outcomes remain unverified. Retain those acceptance gates rather than treating prior-boot null metadata as proof of the whole new branch.
