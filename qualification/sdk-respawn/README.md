# Read-only respawn-target qualification

This independent, inactive workspace consumes the published safe SDK at exact revision `daffae3937398f29939783adc5247d6e5bb75bbf` (ABI 12.13, Rust 1.100.0). It is an rlib, not an installable mod or registered MCP tool. Product dependencies and installed files remain unchanged.

The consumer calls `World::respawn_point` once. It reports the future respawn target, not current position, health, ground contact, arrival, teleportation or revival. No setter is called. The caller must provide a fresh server World and current Player on the server game thread. Future product integration requires the existing Scheduler route and explicit session/selector admission; this prototype supplies neither a durable selector nor a world-session identifier.

Missing observations remain null with explicit unresolved/fallback/departed-player semantics. A legitimate zero coordinate remains an observation. Finite coordinates and quaternion components are copied without clipping to write bounds or normalization. Zero entity identifiers and non-finite components are refused. Entity IDs are session-only. Known kinds retain distinct labels; unknown or future kinds map to `other`, because the safe enum does not expose the original unknown numeric code. Native wrong-thread and unavailable errors remain typed errors.

Seven behavior tests use the published FakeHost: exact observations, unresolved and departed players, kinds and zero origin, thread/availability errors, reading without entity commands, and malformed copied observations. The malformed-value test edits copied SDK values; it does not establish native provider behavior under corruption. Synthetic success does not qualify provider table fidelity, real player observations, native scheduling or persistence.

Run from the repository root with Rust stable 1.100.0 and a separate build directory:

```powershell
$env:CARGO_BUILD_JOBS='2'
$env:RUST_TEST_THREADS='1'
$env:CARGO_TARGET_DIR="$PWD/target/sdk-respawn"
cargo +stable test --locked --manifest-path qualification/sdk-respawn/Cargo.toml
cargo +stable test --locked --release --manifest-path qualification/sdk-respawn/Cargo.toml
cargo +stable clippy --locked --manifest-path qualification/sdk-respawn/Cargo.toml --all-targets -- -D warnings
cargo +stable fmt --manifest-path qualification/sdk-respawn/Cargo.toml --check
```

These seven cases are separate from the product's 401 Rust executions. The staged SDK 12.11 Menu review cannot qualify this ABI 12.13 API. A matching provider and independently observed current-player target are prerequisites before proposing a product read route. No automatic movement or respawn mutation is authorized by this qualification.

Sources: [published player API](https://github.com/enshwick/enshwick/blob/daffae3937398f29939783adc5247d6e5bb75bbf/crates/internal/world/src/world/players.rs), [upstream contract](https://github.com/enshwick/enshwick/blob/daffae3937398f29939783adc5247d6e5bb75bbf/crates/enshwick/changes/respawn-point.md), and [upstream tests](https://github.com/enshwick/enshwick/blob/daffae3937398f29939783adc5247d6e5bb75bbf/crates/enshwick/tests/respawn_point.rs).
