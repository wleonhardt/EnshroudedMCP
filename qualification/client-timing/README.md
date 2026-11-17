# Client command timing qualification

This optional patch instruments a disposable source copy of public commit `c3e650caa59bc79215884bab78b07cdd044876a6`. The product remains unchanged. **Never install binaries built from this qualification copy or apply the patch to a deployment checkout.** It starts no game, opens no endpoint and uses only synthetic SDK fixtures and disposable files.

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
