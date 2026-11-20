# Shared-waypoint read qualification

This independent, inactive rlib consumes the published safe SDK at exact revision `33cf4dc149f88d1abe3d3c11a12053d0ab94967f` (ABI12.16, Rust1.100.0). It is not an installable mod or MCP tool. Product dependencies, compiler pins, signed providers and installed files remain unchanged.

The consumer calls `Map::waypoint` once for a caller-supplied server Player on the server game thread. It preserves the shared map waypoint in metres, not player position, client UI selection, arrival or a traversable route. Entity IDs are session-only; no world-session identifier or current-player membership proof is supplied. Future product integration requires the existing Scheduler route and fresh server-selector/session admission.

Unset or absent points stay null; a legitimate zero coordinate stays observed. Raw finite coordinates are copied without snapping, clamping or converting map extents into write authority. Non-finite copied observations refuse instead of silently becoming JSON null. SDK availability and thread errors remain typed errors. No map setter, command, event subscription, movement or gameplay write is called. Fixture component changes occur only inside the published FakeHost tests.

Eight behavior cases cover precise asymmetric coordinates and proof limits, unset/missing versus zero, a departed selector, wrong-thread/world errors, unavailable layout, successive moved/cleared observations, malformed copied coordinates and reading without entity-command support. The same formatted source is also checked independently against the product's exact SDK6a151a95/ABI12.1 revision under Rust1.100.0. These are two source cohorts of eight cases, not sixteen independent behaviors. The product still builds with its existing Rust1.99.1 toolchain.

`Map::waypoint` already exists in the product's pinned SDK. ABI12.16 moves its transport from the SDK's published reflection-backed implementation to the provider's appended `HostApi::waypoint`; this qualification does not invent a new high-level API or require an SDK upgrade before proposing a pinned read route. Native fidelity of either implementation remains unverified.

Run from the repository root with exact Rust/Cargo stable1.100.0 and a separate target directory:

```powershell
$env:CARGO_BUILD_JOBS='2'
$env:RUST_TEST_THREADS='1'
$env:CARGO_TARGET_DIR="$PWD/target/sdk-waypoint"
cargo +stable test --locked --manifest-path qualification/sdk-waypoint/Cargo.toml
cargo +stable test --locked --release --manifest-path qualification/sdk-waypoint/Cargo.toml
cargo +stable clippy --locked --manifest-path qualification/sdk-waypoint/Cargo.toml --all-targets -- -D warnings
cargo +stable fmt --manifest-path qualification/sdk-waypoint/Cargo.toml --check
```

Debug and optimized SDK-fake results do not prove a native provider, current player, visible map marker, world-session association or preservation after disconnect. Positive native acceptance needs a deliberately set shared waypoint, independently observed coordinates and clear/disconnect behavior on a qualified provider. Do not promote fake success into an executor or automatically move an avatar to a waypoint.

Sources: [current safe getter](https://github.com/enshwick/enshwick/blob/33cf4dc149f88d1abe3d3c11a12053d0ab94967f/crates/internal/world/src/map.rs), [provider transport contract](https://github.com/enshwick/enshwick/blob/33cf4dc149f88d1abe3d3c11a12053d0ab94967f/crates/enshwick/changes/waypoints.md), [published fake fixture](https://github.com/enshwick/enshwick/blob/33cf4dc149f88d1abe3d3c11a12053d0ab94967f/crates/enshwick/tests/map_events.rs) and [pinned safe getter](https://github.com/enshwick/enshwick/blob/6a151a9550ed7a6ade99c23fa817a91e25b71bc9/crates/internal/world/src/map.rs).

Synthetic fixture construction follows the published enshwick examples under its [MIT notice](../../patches/enshwick-LICENSE).
