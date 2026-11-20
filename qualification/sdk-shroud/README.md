# Pinned SDK shroud observation qualification

This independent rlib workspace uses the existing exact official SDK6a151a9550ed7a6ade99c23fa817a91e25b71bc9 / ABI12.1 with Rust/Cargo1.99.1. It does not change product dependencies, register an MCP tool, load a provider, read credentials, connect a client or modify installed files.

The published World::shroud_time and World::in_shroud getters can be read on the server game thread. The consumer preserves their independent optional results: absent data remains null, false is observed false, and a zero timer is observed zero. Both fields present yields observed; one yields partial; neither yields unavailable. The SDK collapses missing components, unsupported layouts and unreadable data into None, so the consumer does not invent a cause or physical absence.

Timer current/max values remain exact signed SDK integer seconds. Negative values, zero maxima and values above the maximum are not clamped or interpreted. SDK units do not establish correspondence with the displayed countdown, damage timing or survival mechanics. No timer refill, write authorization, traversal/damage safety, rendered UI, current player or durable entity identity is established. The supplied server entity ID lasts one session; a future MCP route must independently resolve a current player selector and enforce admission in policy before queuing the read. No listener-thread World access is permitted.

```powershell
rustc --version
cargo --version
# Both must report exactly 1.99.1 in this product checkout.
$env:CARGO_BUILD_JOBS='2'
$env:RUST_TEST_THREADS='1'
cargo test --locked --manifest-path qualification/sdk-shroud/Cargo.toml
cargo test --locked --release --manifest-path qualification/sdk-shroud/Cargo.toml
cargo clippy --locked --manifest-path qualification/sdk-shroud/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path qualification/sdk-shroud/Cargo.toml -- --check
```

Eight debug/release SDK-fake cases pass, along with strict lint/format. They cover unknown versus false/zero, positive and signed extrema, partial field availability, malformed attribute-set headers, updated values without caching, zero-entity refusal and deferred server-tick execution. A feature specification first fails after independent SDK getter assertions prove the zero/false fixture is valid. The first full implementation run also exposes a fixture holding two installed fake hosts at once; correcting its lifetime leaves observation logic unchanged. These are synthetic published-SDK fixtures, not offsets obtained from a game process or native acceptance. No fixture manufactures a world mutation or real Shroud exposure.

Next add one bounded, read-only player observation only after reviewing fresh roster/session admission and unchanged getters. Matched native timer/flag and UI/countdown agreement remain unverified. Missing values or a false flag must never authorize movement or a deadline calculation.
