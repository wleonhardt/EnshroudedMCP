# Inactive native lifecycle control qualifier

This independent workspace builds a bounded qualification mod against official SDK `33cf4dc149f88d1abe3d3c11a12053d0ab94967f`, ABI 12.16 and Rust/Cargo 1.100.0. It exercises published client command/status/cancel APIs from the client frame. It never acquires a server `World`, drives OS input or captures the game.

The default MCP product remains on signed SDK 12.1. No tool is registered by this qualifier. Its native Menu observation callback has now executed in one temporary client cohort (decision 0171). The external controller aborted on an uncovered subprocess-readiness exception before publishing any native request; command execution remains unverified. Exact signed rollback is verified. A separate isolated server cohort loaded its DLL alongside matching development providers and observed immutable headers, bounded server reads and one owned shutdown/exit0 (decision 0168). Server-side DLL loading does not independently prove mod initialization or native client execution. A fresh native controller, matching-provider review and independent connection acceptance remain required. Framework metadata supports server/both; this mod declares both and initializes passively on the server, without profile, input or filesystem access there.

## Private files and admission

The native controller must create a fresh owned mod directory and `profile.json` with exactly `profile` and `query` string fields. The label uses 1–64 lowercase ASCII letters, digits, underscores or hyphens. Query must be a nonzero IPv4 loopback socket address. Do not copy existing game configuration or credential stores. There is no password field or resolver; SDK Join always receives `password: None`. This does not qualify password-protected access or saved credentials.

The controller supplies sequential `request-0001.json` through `request-0032.json`, each at most 4096 bytes. The mod never overwrites, removes or replays a request. A request contains `operation` and `action`; Join additionally requires the configured `profile` label, and Cancel requires `target_operation`. Leave accepts neither field. Addresses, passwords and character selectors are not request arguments. Operation names use the same bounded label format.

Requests default to `dry: true`. Dry produces a current preview and no native action or operation reservation. A separate real request uses the next request number, `dry: false`, the exact preview and `confirm: "server-write"`, with a fresh `ALLOW_WRITES` flag in the mod folder. Do not arm this flag outside an approved trial.

Game-free policy validates arguments, profile, actual copied UI age, ownership, native freshness/session/target-text agreement, preview, lease, deadline, files and capacity. Join requires fresh Menu and idle ownership. Leave requires this probe's observed Join, its first bound matching session and current fresh matching native snapshot. Cancel targets only this boot's retained pending Join. Neither implicitly leaves an unrelated world. SDK execution checks occur later; admission is not atomic with that phase.

At most eight real identities are guarded by memory and exclusive synced `operations/connection-<operation>.json` intents. Removing a file cannot replay a retained memory identity. Exclusive `session.json` refuses restart/reload. Native command IDs stay in memory. Authority, deadline and elapsed frame age are checked again after intent fsync. Refusal after intent burns the identity. Recording/reservation uncertainty halts native dispatch; no replacement operation or cleanup runs automatically.

Dispatch has the existing two-second request deadline. UI/native freshness use the existing one-second bound, preserving unknown age. The 180-second qualification lease starts at initialization. Expiry stops the qualifier; it does not cancel or leave. These limits do not redefine asynchronous SDK Join as a two-second action. A submitted command can continue under game ownership after observation stops.

## Evidence limits

`observations.jsonl` is the bounded synced private journal. It records submissions, status changes, session binding, replies and periodic copied metadata. It omits the query address and reports target-text agreement as true/false/unknown. Matching text is not authentication. First-session association combines copied observations and Join status non-atomically; later sessions cannot rebind that operation. Terminal observations remain historical facts beside a later unavailable status.

No record establishes independent roster, selected character, rendered behavior, provider fidelity or workflow success. Selected-character observation/locking is not supplied here; the published executor requires a selected character and the controller must independently check the result. Normal Join also changes game-owned Steam connection history. Applied, acceptance, timeout or cancellation submission alone proves no successful connection or completed cancellation.

Fifteen policy cases and fourteen optional SDK/file cases cover admission, redaction, dry/real dispatch, identities, revocation, changed session, aged frame, unknown outcomes, terminal retention, lease and passive server initialization. Four failing reproductions precede the memory/session/frame fixes and a bounded-reader fix. The reader consumes at most 4097 bytes even if a file grows after its metadata check, refusing the sentinel byte beyond the 4096-byte JSON bound. The SDK fake draws an in-game frame; Menu tests supply a synthetic screen to the same handler. They prove no native Menu callback or rendering.

## Source checks

From this directory, verify both installed stable tools report exactly 1.100.0:

```powershell
rustc +stable --version
cargo +stable --version
$env:CARGO_BUILD_JOBS='2'
$env:RUST_TEST_THREADS='1'
cargo +stable test --locked
cargo +stable test --locked --release
cargo +stable clippy --locked --all-targets -- -D warnings
cargo +stable fmt --check
cargo +stable build --locked --release
```

Default workspace checks do not execute this independent workspace. Its policy dependency is ordinary game-free product policy; its SDK pin does not change the product dependency. Do not install or launch from these instructions. Preserve the signed baseline until the project's fresh bounded native review passes.
