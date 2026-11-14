# EnshroudedMCP

An experimental [enshwick](https://github.com/enshwick/enshwick) mod that exposes a loopback MCP endpoint for inspecting Enshrouded. Server reads run on the game thread. Request admission, argument bounds and write gates are implemented in a game-free policy crate.

Server version 0.15.0 exposes twenty-two tools: inspection, pure planning, gated `give_item`, bounded administrative construction, and one gated administrative reposition request. Construction copies an observed building voxel rather than inventing material mappings. Recipe planning does not establish live crafting feasibility.

The project is unofficial. Keen's Terms of Use forbid mods (3.3(f)); assess the terms before use. This project contains no game binaries, saves, generated catalogues or credentials and does not modify the Steam DRM stub.

## Endpoints and tools

The server endpoint is `POST http://127.0.0.1:8751/mcp`. Its mod ID is `enshrouded.mcp`, side `server`. Joining players do not need this server-only mod.

| Tools | Scope |
| --- | --- |
| `status`, `players` | Hosting state and connected players |
| `voxel_sample`, `props_in`, `entities_near`, `entity_components` | Bounded spatial inspection; component names require a locally generated catalogue |
| `capabilities`, `inventory_read`, `item_info`, `item_lookup`, `entity_health` | Available APIs, linked inventory pages, vanilla metadata with explicit placed-template states, and typed health observations |
| `knowledge_search` | Curated evidence with provenance and applicability |
| `recipe_plan` | Pure single-output recipe arithmetic over supplied recipes and stock |
| `building_plan` | Floor, wall and doorway previews on a half-metre grid; no placement |
| `building_survey` | At most 4096 complete half-metre cells with exact material/fill, persistence and optional connected-player build rights; no edits |
| `building_place_cell`, `building_apply_plan` | Gated exact-voxel copy into one air cell or up to 256 primitive or composite floor/wall/doorway cells across eight aligned bricks, at most 64 per brick |
| `building_operation`, `building_undo_cell` | Session-bound native transaction reconciliation and guarded undo for cells or plans |
| `building_accept` | Gated acceptance of an exact applied capture to allow another bounded operation; retains session-only undo |
| `player_reposition` | Dry-by-default once-only administrative teleport request for the sole connected living actor; a requested point within 16 m, not guaranteed arrival or bounded engine correction |
| `give_item` | One explicitly gated inventory transaction |

The optional research companion 0.3.0 uses `POST http://127.0.0.1:8754/mcp` and mod ID `enshrouded.mcp-client`. It exposes `client_capabilities`, `client_state`, `client_events`, `client_item`, `client_recipes` and `client_selection_events`. The framework requires side `both`: matching companion installations are needed on the isolated research server and its clients. The dedicated-server half is a no-op. The companion copies native UI snapshots, nullable screen-host templates and named client events. Compiled framework revision/ABI are reported separately from live verification. It provides no desktop automation or gameplay actions.

Current selected tab and passively highlighted recipe remain unknown. The outer Backpack screen is insufficient to identify the Crafting tab. Observed event delivery does not prove current selection or successful crafting.

## Build

Requires Windows MSVC build tools, Rust 1.99.1 and Python 3 for helper tests. Rust is pinned in `rust-toolchain.toml`.

The active source pin is official enshwick development commit `5f647f46d7f987e93d397e00596014b23f3c1112`, ABI 11.30. Its package label remains alpha.15, while the alpha.15 release tag uses ABI 11.28. Prepare the dedicated sibling checkout `../enshwick-nightly` and apply only the three-file SDK metadata patch as [patches/README.md](patches/README.md) describes. Use the matching official development loader artifact and rebuild server and companion together.

The old private alpha.14 ABI 11.20 event extension is historical and inactive. Its function-table layout collides with later official appends; a higher version number does not make those consumers compatible. Do not apply the historical event patch to this pinned official revision or mix SDK/loader lineages.

```text
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
python -B -m unittest discover -s scripts -p "test_*.py"
cargo build --workspace --release
```

## Install and configure

Back up the loader, mods and a cold isolated-world save before testing. Exit the client and coordinate disconnected players before replacing loaded DLLs. An empty-player query alone can race with reconnecting players. Do not test new writes on a shared world.

Install the exact matching official development loader according to the patch instructions. Do not use `--with-loader`, which can substitute the ABI-incompatible alpha.15 release loader. Then install the mod into a chosen game or dedicated-server folder:

```text
cargo enshwick install -p mcp --game <game-folder>
```

Install `mcp-client` separately on both sides only for the optional research companion. This publication uses neutral module IDs and install folders, `mods/enshrouded.mcp/` and `mods/enshrouded.mcp-client/`. Do not install duplicate copies of an earlier differently named research mod.

| Variable | Default | Purpose |
| --- | --- | --- |
| `ENSHROUDED_MCP_PORT` | `8751` | Server loopback port |
| `ENSHROUDED_MCP_TOKEN` | unset | Optional required bearer token |
| `ENSHROUDED_MCP_CLIENT_PORT` | `8754` | Companion loopback port |
| `ENSHROUDED_MCP_CLIENT_TOKEN` | unset | Optional companion bearer token |

Remote access uses an SSH tunnel chosen by the operator. No non-loopback bind setting exists. Never put tokens, server passwords or host details in repository files.

## Safety and limits

Browser Origin requests, non-loopback Host headers, undeclared arguments, oversized bodies and out-of-bounds coordinates are refused. A gameplay write requires `ALLOW_WRITES` in the mod folder, less than 30 minutes old, and `confirm: "server-write"`. `dry: true` resolves and reports without changing the world when the tool supports it. Writes append `journal.jsonl`; keep that private. Remove the write flag after the session.

Entity IDs and slot references are session-scoped. Output caps do not necessarily bound upstream world discovery or linked inventory traversal. Read timeouts do not cancel already queued game work. Existing `give_item` timeout handling does not provide safe automatic retries after an unknown outcome.

Recipe outputs are conditional on supplied assumptions. Alternatives, catalysts, coproducts, real inventory reservation and survival-cost debits are unsupported. Building previews do not check material mapping, collision, traversal, build rights, appearance or persistence.

Construction uses a dry preview session token, validated operation identity and stable logical cell keys. Durable intent is synced before commit; native transaction IDs remain typed in memory. Repeating the same canonical operation polls its receipt rather than committing again. A prior-process tombstone prevents replay but cannot reconstruct native IDs or provide stateful recovery. A lost undo response is reconciled without requeueing undo. Only one unsettled proof is allowed. Each touched brick has a complete baseline and expected capture. Captures are journaled as separate synced records, each at most 64 KiB; full-cycle headroom is reserved under the 8 MiB ledger cap. One native transaction is not a resumable job system. Version 0.12.0 accepts a primitive or a composite of at most eight absolute-position primitive parts. The entire admitted footprint stays within 16 m on every axis, and the sum of candidate cells is capped at 1024 before expansion. Parts form a union: duplicate cells are written once, the first lexical part ID owns an overlap's stable logical key, and doorway omissions do not subtract cells contributed by another part. Part order and omitted default yaw normalize to the same canonical request; renaming a part changes its identity. Existing 256-cell/eight-brick/64-cell-per-brick limits remain. Pure `building_plan` previews are unchanged. `building_accept` is dry by default and does not undo or edit voxels. It requires a current native Applied result, exact current captures, the sole original actor, build rights, persistence and a healthy journal. A non-dry acceptance record is synced before the operation becomes accepted/settled and releases the one-unsettled-operation interlock. The typed native ID stays available in the current session. Duplicate acceptance adds no journal record. Undo of an accepted operation requires the strict expected postimage even for one cell, is blocked by any other unsettled operation, and revokes acceptance/settlement when queued until the exact baseline is verified. Acceptance does not prove save/reload persistence. One isolated five-cell frame plus one-cell filler passed staged acceptance/layering, guarded reverse-order undo and post-undo cold reload; this narrow proof does not establish general concurrent safety or applied persistence for the layers.

Plan placement requires every target cell to be air, one connected actor, build rights and persistence across the full target bounds. Each native change writes exactly one requested cell. Keep the isolated site quiescent through placement and undo; do not run other writes during a proof. Native placement has no compare-and-swap and no atomicity guarantee. The framework can absorb a foreign voxel edit during multi-change application into its recorded postimage. Version 0.9.1 therefore refuses multi-cell undo unless the entire expected voxel postimage still matches; props, persistence and the native undo conflict check remain required. This guard is not a general concurrent-edit safety guarantee. No inventory debit, collision, traversal or stateful prop recovery is established. Applied voxel persistence has only the narrow isolated floor evidence described below.

## Validation and roadmap

Version 0.7.0 adds bounded read-only site surveys. Supply `x`, `y`, `z`, `width`, `height`, `depth` on the 0.5 m grid, with sides at most 8 m. Optional `player` selects one connected name/account key for rights. Voxel output uses x-fastest, then y, then z order. Unloaded reads return errors; unavailable persistence/rights stay unknown. Runtime voxel material IDs are not inventory item IDs or a validated palette. The zone check is not save/reload proof, and an allowed observation does not authorize a later write. Props, collision, traversal, survival costs and complete altar boundaries remain unchecked.

The focused framework contribution is [enshwick PR #1079](https://github.com/enshwick/enshwick/pull/1079). It contains no MCP code and does not replace this snapshot's pinned compatibility patch automatically.

The 0.12.0 source passed 131 Rust tests, 27 Python helper tests, Clippy with warnings denied, formatting and a release build. The original research-ID isolated 0.12.0 installation passed all 18 settled fixed read-only probes. Neutral publication IDs have not been exercised live.

A single-cell proof passed exact whole-capture placement/read-back/undo, idempotent receipt replay and cold reload with all 512 baseline voxels equal. The 16-cell floor plan passed isolated placement, exact read-back, replay, undo and post-undo cold reload. A separate floor experiment verified all 512 applied-capture voxels after cold reload while preserving the placed fixture. Old native receipts remained stale. The 32-cell, two-brick live proof passed exact placement/read-back/replay/undo, with all 1024 capture cells verified and 992 neighbors unchanged. Its final post-undo cold reload passed with all 1024 baseline cells equal; applied floor persistence does not establish stateful workshop recovery or usability.

On 2026-11-14, six staged frame/filler phases each matched the expected whole 512-cell capture. Acceptance/replay guards passed; wrong-order undo was refused without losing the receipt, and reverse-order undo restored the baseline. The rendered gap closed, reopened and disappeared as expected. After a verified cold archive and reload, all 512 baseline cells matched, both old native IDs stayed stale, old-session requests were refused without a fresh reservation, ledger hashes stayed unchanged and the session flag was absent. This is post-undo persistence for the small fixture, separate from the applied-floor evidence.

One isolated 0.12.0 compound fixture passed sixteen proof checks: six unique union cells from eight raw part cells with two overlaps, independent whole-512-cell verification, rendered appearance, reordered real-call replay with one transaction, renamed-part refusal and exact undo. Post-undo cold reload matched all 512 baseline cells, refused stale receipts/tokens and same-identity replay, and left ledger hashes unchanged. This does not prove applied-compound persistence, maximum-cap performance or a functional workshop.

The source has limited isolated live evidence. Known inventory splits, one durability comparison, one enemy health decrease and cold file restoration have been observed. Rendered experiments also showed one cell appear and disappear on undo, a separate five-cell gap versus six-cell solid comparison, and an air-cell placement above terrain contacting grass visually before exact baseline undo. Those separate trials do not demonstrate incremental gap filling or a usable structure. Desktop control used during research is external tooling and is not included here. These findings do not establish general mechanics or complete object-state recovery. See [public evidence](plans/research/live-validation.md), [the construction roadmap](plans/goal.md) and [the queue](plans/next-up.md).

`python -B scripts/smoke.py` checks baseline reads. `scripts/investigate.py`, `scripts/validate_tools.py` and `scripts/client_probe.py` perform fixed read-only probes and save private reports under ignored `gamedata/`. Passing transport probes does not satisfy fixture acceptance. Run `--help` for options.

Generated type catalogues and component-name lists must be produced locally through enshwick's published tooling and remain in ignored `gamedata/`. No generated game catalogue is included.

Version 0.13.0 adds read-only `item_info.placement`: `observed` with a placed `template_guid`, `known_no_template` for a known item without one, or `unavailable` when runtime item metadata cannot be read. GUIDs use 32 lowercase hexadecimal characters in file-byte order. An unavailable runtime definition returns `metadata: null`. This mapping is separate from the inventory item entity template; it does not establish placement eligibility, footprint, rights, collision or spawning authority. The SDK exposes an existing native ABI field, so this increment does not replace the loader or companion.

The historical 0.13 compatibility patch retains its 23-file private alpha.14 experiment. Version 0.14 does not apply that patch. The only active extension is `enshwick-nightly-item-template.patch`: three SDK/test files expose existing item metadata without changing official ABI 11.30.

## Official development migration evidence

Version 0.14.0 and companion 0.3.0 were rebuilt against the exact official development revision. The source workspace passed 133 Rust tests, 30 Python tests, Clippy, formatting and both release builds; the SDK passed 538 prescribed nextest tests. A matched isolated installation replaced eight backed-up binaries only after a verified 38-file cold archive. Installed hashes matched the selected official artifact or rebuilt mods.

All eighteen server and nine companion read-only probes passed with broader acceptance incomplete. The prior shell's entire 4,096-cell reverted baseline remained exact and its old construction receipt stayed stale. Fresh native Menu and joined InGame frames were observed. Local held-input metadata remained null. A non-null host-template fixture and named event delivery after migration remain unverified; the altar interaction trial stopped when an enemy attacked, and the client subsequently logged out with the server empty and write flags absent.

Published input masks and control bindings observe actions; they do not synthesize vanilla walking, camera input, lobby actions or menu navigation. No selected-tab or passive-recipe getter is claimed. Version 0.15 adds the separate bounded server reposition request described below; it is not a vanilla walking or menu actuator. No screenshots, runtime identities or controller patches are included.

## Bounded native administrative reposition

`player_reposition` accepts `operation`, `player`, `x`, `y`, `z`, and optional `dry`, `preview`, `confirm`. It defaults to report-only `dry: true`. Use the returned preview unchanged in a separately gated real request. The exact preview binds the current server session, account, character, actor generation, observed position and requested point. Joined, left and death events invalidate actor generations. A sole connected matching actor with observed positive health is required; client UI entity IDs are not player selectors.

The 16 m Euclidean gate bounds the requested point from the observed current position. The published native route uses engine best-spawn search with its preserve-orientation default. The adjustment radius is not published, so actual displacement, collision safety, hazards and precise arrival are not guaranteed. Operator authority remains the trusted loopback caller, fresh `ALLOW_WRITES` flag and exact `confirm: "server-write"`; no gameplay role is changed.

Before one dispatch attempt, the adapter synchronizes an exclusive durable identity/intent and journal, then retains a once marker in memory. Repeating the same canonical identity returns the stored outcome without dispatching again. Altered requests and old-session durable identities refuse. Partial storage/native failure preserves the blocked identity and uncertainty. Never change identity to conceal an unknown outcome or automate retries.

Submission remains `issued_unknown`, with `arrival_verified: false`, even when `native_request_accepted` is true. That flag means request acceptance, not completion. Observe later server positions and rendered state independently. Returning toward the original point requires a new preview, identity and separately gated command; it is not undo and does not restore health, velocity, camera, input or complete player state. No automatic chaining, walking, camera, menu or lobby executor is added.

The isolated two-command fixture passed six separate phases, rendered/position observations, replay without journal append, exact preservation of all 4,096 construction cells, and cold-reload stale refusal after a verified 40-file archive. The engine-selected observations differed from both requested points. The zero-player old-preview/new-identity refusal proves actor refusal only, not live preview-session-specific rejection. Exact arrival/return, arrival causality, player-position persistence and stateful recovery remain unverified.

Back up `reposition-operations/` together with `construction-operations/`, `journal.jsonl` and the save. These records contain private runtime context and remain outside publication inputs.

## License

MIT. The framework patch retains its upstream license in [patches/enshwick-LICENSE](patches/enshwick-LICENSE). This project is not affiliated with Keen Games or enshwick.
