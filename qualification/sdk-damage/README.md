# Read-only damage-hook coverage qualification

This independent inactive rlib pins published SDK `e8dc967ce7acb0a82bce9840c75aef8c54687cc9`, ABI 12.14, table 1776 and Rust 1.100.0. It supplies no mod entry point, MCP tool or installed provider. Product dependencies and signed installation remain unchanged. The separate SDK 12.11 Menu review 0107 is consumed after a stopped attempt and exact rollback; see decision 0113.

`read()` calls only the published `damage::causes_off()` getter. Its list names causes no registered damage filter receives because hooks were not armed, sites were unavailable or sites faulted. With no filters registered, all named causes are normally off except impact when another mod's init-time stats use arms it. An empty observed list does not prove any filter prevents damage, that the player is healthy, that enemies are pacified or that conditions are quiet. Both protection and quiet-condition verification remain false. This consumer never registers a filter; its test-only fixture can register a pass-through filter.

Published `Off` remains an error before initialization finishes, without a loader or when the getter is unavailable. It never becomes a successful empty list. Successful copies retain exact integer ABI codes and explicit known labels, sort by code for deterministic output, preserve duplicate entries and label unknown codes `other`. There is no entity, session, health, threat or environmental observation. No live freshness cache or server game-thread route is implemented. Future product integration must use the server scheduler, qualify a matching provider and keep hook diagnostics separate from gameplay safety.

Seven behavior cases cover no-loader and init-time refusal, all named causes without a filter, empty off-list with a pass-through filter, missing fog/breathing groups, all named and unknown numeric codes, and unordered duplicate copies. Unknown-code and duplicate cases exercise copied SDK values, not malformed native responses. FakeHost results do not establish actual hook linkage or effects in a running game. Debug, optimized, strict lint and formatting results are recorded in decision 0112.

Run from the repository root after verifying `cargo +stable --version` and `rustc +stable --version` are exactly 1.100.0:

```powershell
$env:CARGO_BUILD_JOBS='2'
$env:RUST_TEST_THREADS='1'
$env:CARGO_TARGET_DIR="$PWD/target/sdk-damage"
cargo +stable test --locked --manifest-path qualification/sdk-damage/Cargo.toml
cargo +stable test --locked --release --manifest-path qualification/sdk-damage/Cargo.toml
cargo +stable clippy --locked --manifest-path qualification/sdk-damage/Cargo.toml --all-targets -- -D warnings
cargo +stable fmt --manifest-path qualification/sdk-damage/Cargo.toml --check
```

The lockfile fixes dependency resolution. `unstable` is explicitly required by this published API; this does not adopt it into the product. Native provider provenance, thread behavior, actual missing-site correspondence and filter delivery require separate qualification. Never test those claims by exposing a character to harm automatically.

Sources: [published getter](https://github.com/enshwick/enshwick/blob/e8dc967ce7acb0a82bce9840c75aef8c54687cc9/crates/internal/inventory/src/damage.rs), [upstream contract](https://github.com/enshwick/enshwick/blob/e8dc967ce7acb0a82bce9840c75aef8c54687cc9/book/src/damage.md), and [upstream fake cases](https://github.com/enshwick/enshwick/blob/e8dc967ce7acb0a82bce9840c75aef8c54687cc9/crates/enshwick/tests/damage_causes_off.rs).
