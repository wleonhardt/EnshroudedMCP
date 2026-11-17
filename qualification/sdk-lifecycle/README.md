# Native lifecycle consumer qualification

This inactive, independent workspace pins official commit `6a5966ee6683fde480ac9dd3654f91e73da6e2a5` from merged [PR #1150](https://github.com/enshwick/enshwick/pull/1150), ABI 12.11. It uses only the published SDK fake. It does not add an MCP endpoint, resolve a real profile, read a credential store, connect to a server or change the product SDK/installed files.

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
