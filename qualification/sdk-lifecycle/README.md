# Native lifecycle and provider qualification

## Current server 0.51.4 source migration

The inactive [0.51.4 migration patch](consumer-sdk1216-source0514.patch) applies only to `acd5b81fa1033767bf1425ae3edc0efdf424738e`. Its [descriptor](source0514-sdk1216.json) records SDK `33cf4dc149f88d1abe3d3c11a12053d0ab94967f` / ABI 12.16, exact Rust/Cargo 1.100.0, patch hash and reconstructed tree. Candidate and clean replay each pass 491 Rust/131 Python checks and strict lint/format/releases/contracts; the optional lifecycle qualifier also passes fourteen debug and optimized cases in each copy. All 393 source files and the tree match. See [decision0181](../../plans/decisions/0181-current-source0514-sdk1216-replay.md) for the 17 stale diagnostic assertions and preserved production gates.

```powershell
$migrationPatch = (Resolve-Path qualification/sdk-lifecycle/consumer-sdk1216-source0514.patch).Path
git worktree add --detach ../enshrouded-sdk1216-source0514-check acd5b81fa1033767bf1425ae3edc0efdf424738e
git -C ../enshrouded-sdk1216-source0514-check apply --check $migrationPatch
git -C ../enshrouded-sdk1216-source0514-check apply $migrationPatch
```

In that checkout, verify `rustc +stable --version` and `cargo +stable --version` both report exactly 1.100.0 before running the workspace test, strict Clippy, formatting, release-build and contract commands with that compiler. No installation or native operation follows from these instructions. Default signed SDK 12.1 and existing prepared native candidates are unchanged. Older probe and migration sections below retain their exact historical bases and scopes.


This independent experimental workspace pins official commit `6a5966ee6683fde480ac9dd3654f91e73da6e2a5` from merged [PR #1150](https://github.com/enshwick/enshwick/pull/1150), ABI 12.11. It uses only the published SDK fake. It does not add an MCP endpoint, resolve a real profile, read a credential store, connect to a server or change the product SDK/installed files.

The revision requires Rust 1.100. The qualification used installed `stable` at exactly Rust/Cargo 1.100.0, with Windows MSVC tools. Verify that version before reproducing; `stable` is not a permanent compiler pin. From the project root:

```powershell
$env:RUST_TEST_THREADS='1'
cargo +stable test --locked --manifest-path qualification/sdk-lifecycle/Cargo.toml
cargo +stable test --locked --release --manifest-path qualification/sdk-lifecycle/Cargo.toml
cargo +stable clippy --locked --manifest-path qualification/sdk-lifecycle/Cargo.toml --all-targets -- -D warnings
cargo +stable fmt --manifest-path qualification/sdk-lifecycle/Cargo.toml -- --check
```

Eight consumer tests exercise ABI shape and SDK Debug redaction; queued/menu/connecting/loading/in-world phases; missing/matching/different copied peers; all nine named failures plus unknown failure/phase preservation; cancellation without implicit replacement or exit acceptance; and join/leave overlap/world admission. The observer omits addresses and passwords, retains unknown peer identity and never promotes a native state into independent roster, character or rendered acceptance. These constant proof limits describe the prototype's scope, not implemented workflow admission or a durable operation store. A failed copied-snapshot getter retains the typed snapshot_read_error and null peer while preserving command state/phase/age; an available missing peer instead has null read error. The published fake supplies Unavailable in the previous ninth total case (seven consumer plus two provider/report cases). This cfg(test)-only correction does not fix or retry the independently failed native Join; see [decision0125](../../plans/decisions/0125-preserve-unavailable-lifecycle-snapshot.md).

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

## SDK 12.16 feature-source migration qualification

The separate [SDK 12.16 feature patch](consumer-sdk1216-feature-candidate.patch) applies only to public base `27ffaa1481e7e1fe0e5c55aa8a38703f49f52f70` (server0.49.1 / companion0.22.0). It pins official main revision `33cf4dc149f88d1abe3d3c11a12053d0ab94967f`, ABI12.16, with Rust/Cargo1.100.0. This is an inactive source candidate, not SDK/provider adoption, a deployment procedure or permission to retry a native lifecycle request. Product main and the feature branch retain SDK 12.1/Rust 1.99.1.

Sixteen paths update exact dependency/compiler provenance, source profiles and fixed probes; five snapshot fixtures explicitly establish sampling/frame prerequisites. Two further test fixtures account for SDK 12.16 Input construction starting snapshot and held-input sampling: the scripted getter returns an empty initialization observation, and cached diagnostics preserve the resulting requested flag. The request still requires exactly two scripted reads, including the post-intent guard, and cached reads submit no commands. Runtime admission, write gates, deadlines, session checks and once-only rules are unchanged.

The initial ABI assertion failure, the scripted initialization callback abort and the requested-flag mismatch are distinct retained qualification failures. Correcting fixture assumptions does not diagnose an earlier native Join failure or establish that the new provider fixes it.

Candidate and independently patched replay each pass 424 Rust executions, 107 Python tests, strict Clippy, formatting, release builds and source contracts. All 325 tracked files match byte for byte, and both indices reconstruct tree `049da547eb234955b055fb7e401e7817ce2e48fe`. Patch SHA-256: `5f2f8d1bc156c014fee4fac96099abce8c0eae08803141906619c8f3b83b6669`. Resolve the patch before creating a clean checkout of its exact base:

```powershell
$sdkPatch = (Resolve-Path qualification/sdk-lifecycle/consumer-sdk1216-feature-candidate.patch).Path
git worktree add --detach ../enshrouded-mcp-sdk1216-check 27ffaa1481e7e1fe0e5c55aa8a38703f49f52f70
git -C ../enshrouded-mcp-sdk1216-check apply --check $sdkPatch
git -C ../enshrouded-mcp-sdk1216-check apply $sdkPatch
Push-Location ../enshrouded-mcp-sdk1216-check
rustc +stable --version
cargo +stable --version
# Proceed only when both report exactly 1.100.0.
$env:CARGO_BUILD_JOBS='2'
$env:RUST_TEST_THREADS='1'
cargo +stable test --locked --workspace
python -B -m unittest discover -s scripts -p test_*.py
cargo +stable clippy --locked --workspace --all-targets -- -D warnings
cargo +stable fmt --check
cargo +stable build --locked --release -p mcp -p mcp-client
python -B scripts/check_contracts.py
Pop-Location
```

The latest published release remains alpha.17; the candidate uses a later exact main commit. Upstream includes published Join-state and joined-terrain corrections, but no native provider, intended-peer/character/roster, Join/Leave, credential-resolution, rendered or persistence acceptance follows from these source checks. No candidate DLL is installed, no current process is controlled and no native command is submitted.

## Lifecycle snapshot freshness and SDK 12.16 comparison

The inactive consumer now preserves snapshot_metadata with the SDK state, nullable age_ms and session separately from copied-target matching and command status. A failed getter retains snapshot_read_error and null metadata; it does not fabricate a Never/zero-session observation. Matching target text remains possible for Fresh, Stale, Off or Never copies. None authenticates a current peer, proves a character/roster match or accepts a workflow. The observer still emits no address or credential and never retries a command.

One new regression fails before the correction and passes afterward. The ABI12.11 cohort, exact SDK33cf4dc149f88d1abe3d3c11a12053d0ab94967f/ABI12.16 cohort and independent patched replay each pass ten debug and ten release cases, strict Clippy and formatting using exact Rust/Cargo1.100.0. Their consumer sources agree after normalizing only pinned revision/ABI expectations. Product workspace checks remain a separate cohort; no product SDK pin or MCP surface changes.

The [inactive optional-consumer patch](consumer-sdk1216-observer.patch) applies only to public base `ff0759f625594c92fa9e53bed41bd1db224be7b0`. It changes four optional-workspace paths and reconstructs tree `5f7844bab7502b258dceefa39c04681bad022e61`. SHA-256: `473c9721d401748fbbd39f4bc7d089c5b6e55e2427126046823e917dfdaa3b8a`. It includes the freshness correction and changes the optional dependency lock/provenance to the later exact SDK; it does not apply the separate whole-product migration.

```powershell
$observerPatch = (Resolve-Path qualification/sdk-lifecycle/consumer-sdk1216-observer.patch).Path
git worktree add --detach ../enshrouded-lifecycle-observer-check ff0759f625594c92fa9e53bed41bd1db224be7b0
git -C ../enshrouded-lifecycle-observer-check apply --check $observerPatch
git -C ../enshrouded-lifecycle-observer-check apply $observerPatch
Push-Location ../enshrouded-lifecycle-observer-check
rustc +stable --version
cargo +stable --version
# Proceed only when both report exactly 1.100.0.
$env:CARGO_BUILD_JOBS='2'
$env:RUST_TEST_THREADS='1'
cargo +stable test --locked --manifest-path qualification/sdk-lifecycle/Cargo.toml
cargo +stable test --locked --release --manifest-path qualification/sdk-lifecycle/Cargo.toml
cargo +stable clippy --locked --manifest-path qualification/sdk-lifecycle/Cargo.toml --all-targets -- -D warnings
cargo +stable fmt --manifest-path qualification/sdk-lifecycle/Cargo.toml -- --check
Pop-Location
```

These SDK fakes supply lifecycle transitions. They do not qualify provider fidelity, an actual connection, cancellation/Leave effects, credential provisioning, current-peer identity or native crash causes. No candidate is installed. Nested qualification target directories are ignored to keep ordinary local build output outside publication; source files remain visible.

## Current feature source against exact SDK 12.16

Decision0165 adds consumer-sdk1216-current-feature.patch against exact feature base `291ae31292c429c573552f12664657891dbff29b` (server 0.51.3 / companion 0.22.0). This inactive migration affecting 29 files reconstructs 373 source files. Candidate and clean replay each pass 472 Rust tests and 125 Python tests and required checks. The two extra scheduling discriminators and stopped-world regression are source/fake evidence only. Existing signed product SDK 12.1 and its lifecycle tests remain unchanged; no provider or Join/Leave workflow is adopted.

SDK 12.16 passes the scheduler before WorldLoaded and discards tasks from an ended game. Cross-game tests use the published end_game boundary and assert no returned old-world data. Same-game shutdown tests retain positive SDK observations and exact active-lifecycle refusal. These do not establish the same claim. The transport error alone supplies no native cancellation or replay authority. Source compatibility cannot replace actual header, peer, character, roster, private credential and Leave acceptance.

```powershell
$migrationPatch = (Resolve-Path qualification/sdk-lifecycle/consumer-sdk1216-current-feature.patch).Path
git worktree add --detach ../enshrouded-sdk1216-current-check 291ae31292c429c573552f12664657891dbff29b
git -C ../enshrouded-sdk1216-current-check apply --check $migrationPatch
git -C ../enshrouded-sdk1216-current-check apply $migrationPatch
Push-Location ../enshrouded-sdk1216-current-check
rustc +stable --version
cargo +stable --version
# Proceed only when both report exactly 1.100.0.
$env:CARGO_BUILD_JOBS='2'
$env:RUST_TEST_THREADS='1'
cargo +stable test --locked --workspace
python -B -m unittest discover -s scripts -p 'test_*.py'
cargo +stable clippy --locked --workspace --all-targets -- -D warnings
cargo +stable fmt --check
cargo +stable build --locked --release -p mcp -p mcp-client
python -B scripts/check_contracts.py
Pop-Location
```

The replay can reuse a dependency artifact cache while Cargo rebuilds its distinct workspace packages; compiler-artifact package identities and metadata are checked independently. The exact checked sources and DLLs are private qualification artifacts, not an installed provider. Official matching development providers lack detached signatures and need the project's fresh signed-provider exception review before any temporary native test. Six rollback files are verified; no installation or test approval is implied by source checks.
