# Contributing

Use small changes with an explicit behavior and acceptance check. The project is MIT licensed; preserve the project license, the enshwick attribution and patch provenance. Do not include game files, decompiled material, generated reflection catalogues, saves, private server details, credentials or runtime screenshots in a contribution.

## Build prerequisites

Follow [setup](setup.md) for the exact official Git-pinned SDK and matching signed loader. No SDK patch is active. Rust is pinned to 1.99.1. Windows builds need MSVC and the Windows SDK. The optional companion must be rebuilt against the same SDK lineage.

## Required checks

```text
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
python -B -m unittest discover -s scripts -p "test_*.py"
cargo build --release -p mcp -p mcp-client
```

A running game is unnecessary for source tests. Python 3.11 or newer supplies the standard-library TOML parser for contract checks. Run `python -B scripts/check_contracts.py` for offline index/profile consistency; add explicit server/client ports to compare an existing installation's metadata discovery. Test counts and dated native evidence are recorded in the capability matrix and decisions.

For framework SDK changes, use the exact candidate's documented harness and focused regressions without altering the dependency cache. Historical qualification patches remain inactive. Broad framework contributions have additional checks in its own `CONTRIBUTING.md`.

## Design boundaries

Security-relevant admission belongs in the game-free `policy` crate. World access belongs on the server game thread through the existing scheduler. The transport must remain loopback-only. Reject unknown tool arguments, preserve dry behavior and never infer mutation authority from client IDs, UI events or input bits.

For writes, retain the fresh flag, explicit confirmation, journal and session/replay contracts. Construction must preserve whole-brick read-back and native transaction identity. Repositioning remains a once-command interface with no automatic retry or arrival guarantee. Avoid adding generic job infrastructure before a measured workload requires it.

## Evidence and documentation

Write a failing reproduction before fixing a bug. Test observable behavior and boundaries rather than duplicating SDK internals. Record source-only, synthetic and live checks separately. A passing read probe is not a live fixture acceptance result.

Update the changelog, tool reference and relevant plans for behavior changes. Before a live experiment, define a tiny disposable fixture, an independent expected result, recovery boundaries and a clear pause for observation. Never experiment on a shared world by treating an empty-player snapshot as exclusive access.

Original SVG branding is code-native artwork under the project MIT license. Keep it self-contained, accessible and free of game imagery, external scripts and embedded identifiers.

The optional [journal pressure qualification](../qualification/journal-pressure/README.md) executes real concurrent appends and Windows lock refusal against a disposable journal. Run its debug/release executables and scoped checks separately; workspace tests do not execute this independent suite.
