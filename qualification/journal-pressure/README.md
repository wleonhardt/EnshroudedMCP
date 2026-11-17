# Journal pressure qualification

This optional Windows-only workspace compiles the unchanged production journal adapter directly. It uses the current product policy dependency and a disposable local file; it never loads the SDK, opens an MCP endpoint, starts a game or edits installed files. Product versions and runtime dependencies remain unchanged.

Run from the project root with the pinned Rust toolchain:

```powershell
cargo run --locked --manifest-path qualification/journal-pressure/Cargo.toml
cargo run --locked --release --manifest-path qualification/journal-pressure/Cargo.toml
cargo clippy --locked --manifest-path qualification/journal-pressure/Cargo.toml --all-targets -- -D warnings
rustfmt --edition 2024 --check --config skip_children=true qualification/journal-pressure/src/main.rs
```

The executable asserts behavior; a successful compilation alone is not a pass. A barrier releases 128 unrelated one-byte appenders with only 64 unreserved bytes available. Exactly 64 must succeed. Two owners retain 100 bytes each. A real Windows sharing lock refuses an owner append without spending recovery credit or changing bytes. After unlocking, a 75-byte owner append spends only that credit; release makes its remaining 25 bytes available to a generic writer while preserving the other owner's 100-byte recovery append. The final file must equal the exact journal cap, with the entire original prefix and every appended suffix checked byte for byte.

Reports live under ignored `gamedata/journal-pressure-qualification/`, separately for debug and release. The suite refuses an existing report or fixture rather than overwriting evidence or repeating an ambiguous run. Successful runs remove only the explicitly named disposable journal and empty fixture directory. Preserve reports before deliberately preparing another run.

This proves actual adapter serialization, capacity accounting, open-refusal behavior and filesystem byte preservation on the tested Windows filesystem. It does not inject partial writes or sync failures, establish power-loss durability, prove native undo capacity, test game callbacks or establish recovery across process restart. Those remain separate acceptance gates. The inherited adapter's existing tests can also compile under Clippy; they are not counted as newly added tests.
