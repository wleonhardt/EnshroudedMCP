# Native lifecycle and provider qualification

This independent experimental workspace pins official commit `6a5966ee6683fde480ac9dd3654f91e73da6e2a5` from merged [PR #1150](https://github.com/enshwick/enshwick/pull/1150), ABI 12.11. It uses only the published SDK fake. It does not add an MCP endpoint, resolve a real profile, read a credential store, connect to a server or change the product SDK/installed files.

The revision requires Rust 1.100. The qualification used installed `stable` at exactly Rust/Cargo 1.100.0, with Windows MSVC tools. Verify that version before reproducing; `stable` is not a permanent compiler pin. From the project root:

```powershell
$env:RUST_TEST_THREADS='1'
cargo +stable test --locked --manifest-path qualification/sdk-lifecycle/Cargo.toml
cargo +stable test --locked --release --manifest-path qualification/sdk-lifecycle/Cargo.toml
cargo +stable clippy --locked --manifest-path qualification/sdk-lifecycle/Cargo.toml --all-targets -- -D warnings
cargo +stable fmt --manifest-path qualification/sdk-lifecycle/Cargo.toml -- --check
```

Six consumer tests exercise ABI shape and SDK Debug redaction; queued/menu/connecting/loading/in-world phases; missing/matching/different copied peers; all nine named failures plus unknown failure/phase preservation; cancellation without implicit replacement or exit acceptance; and join/leave overlap/world admission. The observer omits addresses and passwords, retains unknown peer identity and never promotes a native state into independent roster, character or rendered acceptance. These constant proof limits describe the prototype's scope, not implemented workflow admission or a durable operation store.

A copied snapshot requests sampling on first read; the fake needs an interface frame before supplied peer values become available. The fixture explicitly checks the initial absence and drives that frame before positive peer checks. It does not weaken expected matching/different results. Fake phase/failure setters inject provider replies; passing cases do not prove actual connection transitions, timing, password hashing, cancellation returning to Menu or native provider fidelity.

Published source supports address-based Join with an optional SDK Password, Leave, command-ID cancellation and a copied server string for loader-initiated joins. It does not expose the proposed game-saved profiles or password retrieval; manual/other joins can have no copied server. Debug redaction alone proves neither secure credential storage nor memory zeroization. Before any product route, separately qualify exact provider provenance/ABI and whole-product compatibility, design an allowlisted local profile plus privately owned credential resolver, preserve once-only identities and pre-world context generations, then prove intended peer/character/roster agreement and clean leave on an isolated client. No real secret is needed or accessed by these tests.

The copied server string describes the loader-initiated join target. Equality with a configured address is not independent authentication of the remote peer or proof of the selected character. Require separate connection/roster evidence; never treat a missing target as a different peer.

## Whole-product migration candidate

The inactive [consumer migration patch](consumer-candidate.patch) applies to sanitized public base `813fe58f7b4795ce3cc616d4d12dc37406ed8ecb` (server 0.46.1 / companion 0.20.2). It changes thirteen files: SDK/compiler pins and lock graph, compiled provenance/probe expectations, source-profile documentation and five snapshot warm-up fixtures. It preserves the corrected terrain-caption regression and all production admission, wait, session, write and recovery rules. It adds no Join/Leave MCP route or credential resolver.

Apply it to a clean copy of that exact base, using a patch path outside the copy. For example, from a checkout that contains this patch:

```powershell
$migrationPatch = (Resolve-Path qualification/sdk-lifecycle/consumer-candidate.patch).Path
git worktree add --detach ../enshrouded-mcp-lifecycle-check 813fe58f7b4795ce3cc616d4d12dc37406ed8ecb
git -C ../enshrouded-mcp-lifecycle-check apply --check $migrationPatch
git -C ../enshrouded-mcp-lifecycle-check apply $migrationPatch
Push-Location ../enshrouded-mcp-lifecycle-check
$env:CARGO_BUILD_JOBS='2'
$env:RUST_TEST_THREADS='1'
cargo +stable test --locked --workspace
cargo +stable clippy --locked --workspace --all-targets -- -D warnings
cargo +stable fmt --check
python -B -m unittest discover -s scripts -p test_*.py
cargo +stable build --locked --release -p mcp -p mcp-client
python -B scripts/check_contracts.py
Pop-Location
```

Use the compiler version described above. Source compilation and fake-model results cannot certify the provider, actual connection outcomes or a safe upgrade. The active product dependency and installed signed provider remain unchanged.

Candidate and clean replay each pass 391 Rust executions, 94 Python tests, strict Clippy, formatting, releases and offline contract checks. All 237 tracked files match after newline normalization and independently reconstruct Git tree `b727eccea31031d012f608d7bf4f9f3ce74a9e0b`. Patch SHA-256: `a9978fd8d605d28579165e9caf22fbc4cb1716f5733ff32a689cc4e29ff2c3ec`. See [decision 0077](../../plans/decisions/0077-replayable-lifecycle-sdk-candidate.md) for evidence limits.

## Native provider-header probe

The crate also builds `sdk_lifecycle_qualification.dll`, an opt-in diagnostic mod with side `both`. Its initialization copies the published immutable `Host::api().header` into one exclusively created `provider-observation.json` in its own mod directory. It records the actual provider ABI/table size separately from compiled SDK values. It acquires no World or Input handles and submits no command. This introduces no World-thread exception or MCP endpoint.

Two additional behavior tests distinguish supplied header values from compiled constants and verify exclusive/missing-directory file behavior. The suite now has eight tests: six lifecycle consumer models plus these two probe-helper cases. Native entrypoint execution is qualified separately in a disposable server; it is not driven by those helper tests.

```powershell
cargo +stable build --locked --release --manifest-path qualification/sdk-lifecycle/Cargo.toml
```

Stage that DLL only with an explicitly paired provider in a new disposable runtime. An existing diagnostic refuses overwrite and disables this probe's new initialization rather than retrying. See [the native procedure and bounded result](NATIVE.md). The native server result does not qualify client callbacks or real Join/Leave.
