# Agent operations

Follow the user's request, this file, then settled choices in `plans/decisions/` and the queue in `plans/next-up.md`.

## Non-negotiable rules

- Build, tests and lint must pass: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, and `python -B -m unittest discover -s scripts -p "test_*.py"`.
- Check `plans/decisions/` before structural changes. Project knowledge belongs in `plans/`, not agent memory.
- Preserve existing uncommitted changes. Never force-push or rewrite shared history without permission.
- Every security rule belongs in game-free `crates/policy` with behavior tests.
- Bind loopback only. Unknown tool arguments are refused.
- Every world read or write runs on the game thread through `Scheduler::next_tick`. Listener threads never access the world.
- The companion copies native UI state on its owning callback and observes explicitly client-scoped events. It grants no server authority; client IDs are never server selectors.
- Entity IDs last one session. Durable building identity requires tested labels or plan keys.
- Writes require the existing fresh `ALLOW_WRITES` gate and explicit confirmation, meaningful dry runs, journaling and recovery. Building writes retain typed session transaction IDs for undo and reconcile unknown outcomes without replay. Keep proof sites quiescent; multi-cell undo requires the exact expected voxel image.
- Do not commit credentials, private host details, player identities, generated game data, logs, saves or build output.
- Use published framework APIs. Do not disassemble, dump or patch game code or touch the Steam DRM stub.
- Coordinate disconnected players before restarting a shared server. An empty-player check alone is insufficient.
- Mark all missing live acceptance explicitly. Passing offline tests is not live validation.

## Context recovery

Read `plans/README.md`, `plans/next-up.md`, `plans/open-questions.md`, the newest decision, `CHANGELOG.md`, and `git log --oneline -20`. Then read `plans/goal.md`.

## Before done

Run the checks above. Update public documentation and changelog for behavior changes. Record decisions and remaining validation in `plans/`. Keep private deployment details outside the repository.

## Structure

Rust 2024 workspace: `crates/policy` holds rules, `crates/mcp` the server adapter, `crates/transport` loopback HTTP/MCP, and `crates/client` the optional native research companion. `scripts` contains Python read-only probes and helper tests. `patches` contains the reproducible pinned framework extension. Read its instructions before building.

The current SDK pin is official commit `5f647f46d7f987e93d397e00596014b23f3c1112`, ABI 11.30, through sibling `enshwick-nightly`. Apply only the active metadata patch; the old private event patch is historical. Read `patches/README.md` before changing dependency or loader lineage.

`player_reposition` is a once-command request, not a construction transaction or arrival receipt. Preserve exact actor/session previews, positive-health/sole-actor admission, fresh pre-dispatch gates, synced durable intent and cached outcomes. The requested-distance gate does not bound native spawn correction. Never redispatch a reserved identity or automate retries; a return is a new separately gated request, not undo. Keep both operation namespaces and journals private.
