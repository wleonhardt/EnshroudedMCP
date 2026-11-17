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

## Current-product migration candidate

The separate [current-base patch](consumer-current-candidate.patch) applies only to public `d745bc15c0f801d5a6fc601358935368f4c8afaf`, server 0.47.0 / companion 0.21.0. It preserves the historical 0.46.1 patch above. Fourteen paths update exact SDK/compiler provenance, lock graph, five fake snapshot prerequisites, runtime-header compiled expectations, source profiles and fixed probes. No production guard, deadline, schema, tool inventory or lifecycle route changes.

Candidate and clean replay each pass 398 Rust executions, 96 Python tests, strict Clippy, formatting, releases and contract checks with Rust/Cargo 1.100.0. All 250 normalized tracked files agree. Both source indices and a separately applied canonical patch reconstruct tree `a570197af25de6616d113771f6ea50554b952c1e`. Patch SHA-256: `3d00f60209e05accfae9c76ab805c0c0c7a862e1d9655e3e516cb22e4ccd3127`.

Resolve this patch before checking out its exact base, then use the same locked check commands and explicit compiler verification above:

```powershell
$currentMigrationPatch = (Resolve-Path qualification/sdk-lifecycle/consumer-current-candidate.patch).Path
git worktree add --detach ../enshrouded-mcp-current-sdk-check d745bc15c0f801d5a6fc601358935368f4c8afaf
git -C ../enshrouded-mcp-current-sdk-check apply --check $currentMigrationPatch
git -C ../enshrouded-mcp-current-sdk-check apply $currentMigrationPatch
```

Candidate DLLs are not installed or native-qualified. Their declared mod IDs, directory coherence, endpoint exclusivity and rollback require a separate concrete runtime procedure. The active signed SDK remains ABI 12.1 and the client remains closed. Earlier bounded provider and legacy-client Menu experiments do not qualify this current consumer, real Join/Leave, credentials or selected-character/peer identity. See [decision 0085](../../plans/decisions/0085-current-base-lifecycle-sdk-replay.md).

## Native provider-header probe

The crate also builds `sdk_lifecycle_qualification.dll`, an opt-in diagnostic mod with side `both`. Its initialization copies the published immutable `Host::api().header` into one exclusively created `provider-observation.json` in its own mod directory. It records the actual provider ABI/table size separately from compiled SDK values. It acquires no World or Input handles and submits no command. This introduces no World-thread exception or MCP endpoint.

Two additional behavior tests distinguish supplied header values from compiled constants and verify exclusive/missing-directory file behavior. The suite now has eight tests: six lifecycle consumer models plus these two probe-helper cases. Native entrypoint execution is qualified separately in a disposable server; it is not driven by those helper tests.

```powershell
cargo +stable build --locked --release --manifest-path qualification/sdk-lifecycle/Cargo.toml
```

Stage that DLL only with an explicitly paired provider in a new disposable runtime. An existing diagnostic refuses overwrite and disables this probe's new initialization rather than retrying. See [the native procedure and bounded result](NATIVE.md). The native server result does not qualify client callbacks or real Join/Leave.

## Admission-preserving migration candidate

The separate [admission-current patch](consumer-admission-candidate.patch) applies only to public `8a9de10f4971903e878c721224bdf58f878bbee2`, server 0.47.0 / companion 0.21.1. It preserves both older artifacts above and the complete production control prefix, including final permission and deadline rechecks after copied-snapshot preparation. The SDK/compiler pins remain exact revision `6a5966ee6683fde480ac9dd3654f91e73da6e2a5`, ABI 12.11 and Rust/Cargo 1.100.0.

The newer scripted snapshot fixture initially aborts because SDK initialization calls its getter before the script exists. The fixture now returns `Raw::EMPTY` while unarmed and explicitly checks Never/session-zero/unknown-age before the six command cases. The failed run is retained; no game process is involved. Production gates, deadlines, versions, tools and schemas remain unchanged.

The fourteen-file canonical patch cleanly applies with whitespace errors refused. All 259 normalized tracked files match and reconstruct tree `5f304b27847ce718628cd22267aad00b40feecc5`. Patch SHA-256: `977cfd382b8d495545c2a377f8a111e9e63426265a860bf3fc61a8aec47be3b1`. Candidate and clean replay each pass 399 Rust executions, 96 Python tests, strict Clippy, formatting, both releases and offline contracts.

Resolve this patch before checking out its exact base, then run the locked check commands above with the explicitly verified compiler:

```powershell
$admissionMigrationPatch = (Resolve-Path qualification/sdk-lifecycle/consumer-admission-candidate.patch).Path
git worktree add --detach ../enshrouded-mcp-admission-sdk-check 8a9de10f4971903e878c721224bdf58f878bbee2
git -C ../enshrouded-mcp-admission-sdk-check apply --check --whitespace=error-all $admissionMigrationPatch
git -C ../enshrouded-mcp-admission-sdk-check apply --whitespace=error-all $admissionMigrationPatch
```

No candidate is installed or native-qualified. The current signed 0.47.0/0.21.1 runtime stays on ABI 12.1. The old native-review closure is stale after that signed update; prepare a fresh matching provider/consumer/rollback review before any native experiment. Source tests do not establish native timing, rendered behavior, credentials or Join/Leave acceptance. See [decision 0092](../../plans/decisions/0092-admission-preserving-sdk1211-replay.md).

## Current server 0.48.0 source candidate

The distinct [0.48.0 migration](consumer-server048-candidate.patch) applies to exact public `3f647e1d5e6f3499894a1ff1c8278cc878ba965c`, server 0.48.0 / companion 0.21.1. It pins SDK `6a5966ee6683fde480ac9dd3654f91e73da6e2a5`, ABI 12.11 and Rust 1.100.0. Preserve all older exact-base artifacts. This candidate retains complete terrain pin columns, journal process coverage and the entire final-admission production control prefix. It adds no Join/Leave route, credential resolver, provider adoption or native acceptance.

Apply the new patch outside a clean checkout of that exact base, using the reproduction procedure above and separate candidate/replay targets. Verify installed `cargo +stable` and `rustc +stable` are exactly 1.100.0 before using that moving alias. Candidate and independent clean replay each pass 404 Rust executions, 97 Python tests, strict lint/format, both releases and offline contracts. All 274 normalized tracked files and index tree `ac2172f8177d5d4984afa1bb3be85d168fb5a835` agree. Patch SHA-256: `b5d87a7de40aa64a0c32d1c5748607c64576498573eef4d9383c3f20f34abb43`.

An initial unmodified cold baseline fails two positive command fixtures with the existing deadline error and native_submitted false. Two instrumented cohorts and an unmodified warm cohort later pass; they do not fix or identify the original cause. Production waits and guards are unchanged. Merged test-harness labels are not reliable attribution for fixture-local timing sequences. Preserve failed fixture evidence and add explicit bounded request provenance before storage/timing conclusions. See [source proof and limitations](../../plans/decisions/0100-current-server048-sdk1211-replay.md).

The actual signed 0.48.0/0.21.1 installation remains on SDK 6a151a95/ABI12.1. Old native review 0093 is retired. A fresh source-matched consumer, provider/probe/rollback closure and explicit temporary exception under decision 0059 are required before an unsigned Menu test. No clean-source pass authorizes that swap or validates character, peer, Join/Leave, credentials, rendered or gameplay behavior.
