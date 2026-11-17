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

## Graceful process restart

```powershell
cargo run --locked --manifest-path qualification/journal-pressure/Cargo.toml --bin restart
cargo run --locked --release --manifest-path qualification/journal-pressure/Cargo.toml --bin restart
rustfmt --edition 2024 --check --config skip_children=true qualification/journal-pressure/src/bin/restart.rs
```

This separate executable starts and awaits two real child processes against the same disposable journal. The first reserves 200 bytes and durably appends a thirteen-byte owner record, retaining 187 bytes. After that process exits successfully, the second must observe the exact twenty journal bytes with zero reservations and zero retained credit. Both observations remain report-only and confer no write authorization. The parent independently compares the complete file after each child. Reports are exclusively created under ignored `gamedata/journal-restart-qualification/`; existing reports/fixtures refuse reuse. Cleanup removes only the named journal and empty fixture directory.

This establishes graceful process-boundary behavior of the unchanged production journal adapter. Its reservations are deliberately session-local; it does not recover them or establish gameplay recovery authority from journal bytes. It does not test construction-operation replay admission, forced termination, power loss, partial writes or sync failures. The earlier pressure experiment's process-restart limit remains accurate for that experiment alone. The default executable remains the original pressure check.

## Adapter durability-error cases

`cargo test --locked --manifest-path qualification/journal-pressure/Cargo.toml` also executes the directly included production adapter tests. Its private callback seam retains the normal write-all/sync-all path while allowing two deterministic errors: after one actual appended byte, and after the full 100 bytes without successful sync. Both owners retain all 200 reserved bytes; exact file contents and conservative capacity refusals are checked. These are injected outcomes rather than actual operating-system write/sync faults, and no successful durability or game recovery is inferred. Run with `--release` for the optimized cases.
