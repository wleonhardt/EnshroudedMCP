# Client command timing qualification

The historical `consumer-probe.patch` instruments a disposable source copy of public commit `c3e650caa59bc79215884bab78b07cdd044876a6`. The product remains unchanged. **Never install binaries built from this qualification copy or apply the patch to a deployment checkout.** It starts no game, opens no endpoint and uses only synthetic SDK fixtures and disposable files.

Apply `consumer-probe.patch` in a separate clean checkout of that exact commit. Use a separate `CARGO_TARGET_DIR`; shared output without a forced matching rebuild is not source correspondence proof. The pinned SDK and Rust toolchain remain unchanged.

```powershell
$probePatch = 'C:/path/to/consumer-probe.patch'
git apply --check --whitespace=error-all $probePatch
git apply --whitespace=error-all $probePatch
$env:CARGO_BUILD_JOBS = '2'
$env:RUST_TEST_THREADS = '1'
$env:CARGO_TARGET_DIR = 'C:/path/to/separate-build-directory'
cargo test -p mcp-client timed_post_sync_wait_refuses_before_native_and_retains_once_only_identity -- --nocapture
cargo test -p mcp-client native_dispatch_replay_cancel_reload_and_audit_failures_never_duplicate_input -- --nocapture
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

The controlled fixture waits once for the existing deadline plus 50 ms **after** the exclusive marker sync. It then requires the exact pre-submission deadline refusal, zero native commands, two journal rows, identical replay/reopen bytes and one marker sync. The injected wait is a separate trace stage; it does not simulate a failed or stalled filesystem sync. It never changes production deadlines or grants retry authority.

`CLIENT_TIMING` reports contain at most 128 rows, with an explicit dropped count. Rows distinguish durations from queued ages and name admission, namespace open/create, journal metadata, marker/journal open/write/sync, snapshot getter, native submission, reservation total and request total. The fixture emits its report after execution. Measurements include OS descheduling and observer overhead; nested durations must not be summed. Directory enumeration beyond opening the iterator and other uninstrumented work remain within the total. Direct consumer fixtures do not establish native frame-queue behavior.

The controlled run retains all 32 rows, isolates the deliberate wait and makes no native submission. An ordinary existing fixture also passes without injection, but its first 128 rows omit later work; do not infer complete counts or absence from that truncated report. Recorded reservation work can consume a substantial part of the real deadline. These observations do not identify the cause of historical failures or distinguish intrinsic storage latency from descheduling during an I/O call.

Preserve failed cohorts and bounded reports before a deliberate new run. A passing rerun does not fix a historical failure. This qualification supplies no production MCP tool, runtime tracing option, new SDK capability, game behavior or native interruption acceptance.


## Current final-admission consumer

The separate [current probe](consumer-admission-probe.patch) applies only to public `010b5932b690c4049ff0b907d78ca65af77cb478`, server 0.47.0 / companion 0.21.1, original SDK `6a151a95` / ABI 12.1 and Rust/Cargo 1.99.1. Preserve the historical artifact above. **Never install this instrumented consumer.** Use a separate disposable checkout and target directory, then the test/lint/format commands above with `--locked` on test and Clippy. Also select `changed_context_after_durable_intent_refuses_without_dispatch_or_replay -- --nocapture` to inspect final-getter delay and permission revocation. Python helper tests and offline contracts pass independently.

Each test-build State::run clears its bounded row buffer and emits one report when its run-total scope ends. The deliberately armed wait survives this reset. Each report declares scope, a fixture-local request sequence, limit 128 and dropped count. The sequence resets with each fixture. The final-getter test's valid case submits once; five refusal cases add replay/reopen, giving 16 reports. Work outside State::run, native frame scheduling and a fatal abort without scope completion are outside this observation.

Retained selections contain 42 complete intervals and 420 rows, maximum 26 per request, with zero drops: four post-sync, 22 ordinary and 16 getter/context requests. Queued ages before/after final admission distinguish the delayed getter from post-sync expiry. Existing refusal and once-only assertions remain unchanged. The ordinary maximum reservation elapsed time is 10,522 microseconds in this selection; no organic deadline failure or historical cause is reproduced. Durations include scheduling/observer overhead and do not measure intrinsic storage service time.

The instrumented copy passes 400 Rust executions, 96 Python tests, strict Clippy, formatting and contracts. Clean patch application with whitespace errors refused matches all 262 normalized sources and reconstructs tree `3984a2c5d55e04dd57f6fd6ea86da665cc0497d5`. Replay was not separately compiled. Patch SHA-256: `be76b5689bbc1d1e207362e3f3b280cde299cf44dae12e85068bbfb6e8a502aa`. See [decision 0094](../../plans/decisions/0094-request-scoped-client-timing.md). Product and installed runtime remain unchanged; native qualification is separate.

## Attributed slow-only probe for current 0.48.0 source

The distinct [attributed probe](consumer-attributed-slow-probe.patch) applies only to public `855e6b5deabd61df14f5f2acc71e65910b06a024`, server 0.48.0 / companion 0.21.1, SDK 6a151a95 / ABI 12.1 and Rust 1.99.1. Preserve older artifacts. **Never install this consumer.** Use the separate-checkout/target commands above and all required workspace checks.

Each report has the synthetic `operation`, generated `fixture_tag`, optional `thread_name` and fixture-local `request_sequence`. Missing origin stays null. These identify diagnostic intervals, not players, authenticated peers or persistent objects. Do not associate reports using merged test-harness label order. Scope is one instrumented command interval, including direct synthetic recorder checks.

Rows remain bounded to 128 with explicit `dropped`. JSON formatting/output is deferred unless a retained duration/queued age reaches the existing two-second threshold or truncation occurs. Fast complete intervals are unprinted; that does not count requests or prove effects. Timings include scheduling/observer overhead, and nested durations are not additive storage service time. Correlate outcomes separately using retained synthetic evidence.

The controlled post-sync report matches an independent marker/fixture anchor. Unnamed-thread/missing-directory origin stays null. A synthetic overflow emits 128 zero-duration rows plus three explicit drops. Full source passes 408 Rust executions, 97 Python tests and strict lint/format/releases/contracts. Six bounded selected-test trials in the earlier generated fixture area pass without measuring an organic expiry. The private junction is restored and twelve prior evidence files remain exact. These are not a timing fix or native acceptance.

Patch SHA-256: `a087f6bd3ff7192cb5119ae548f6da10d009c025200aa0599140872792319933`. Strict clean application matches all 277 normalized tracked files and tree `049e2747d8f2a5d6e52b1eb2148d5f8538e87c34`. Replay is not separately compiled. See [decision 0102](../../plans/decisions/0102-attributed-slow-command-intervals.md). The active product and signed installation remain unchanged.
