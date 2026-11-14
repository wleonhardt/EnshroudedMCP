# EnshroudedMCP

An experimental [enshwick](https://github.com/enshwick/enshwick) mod that exposes a loopback MCP endpoint for inspecting Enshrouded. Server reads run on the game thread. Request admission, argument bounds and write gates are implemented in a game-free policy crate.

Server version 0.9.1 exposes twenty tools: inspection, pure planning, gated `give_item`, and bounded administrative construction. Construction copies an observed building voxel rather than inventing material mappings. Recipe planning does not establish live crafting feasibility.

The project is unofficial. Keen's Terms of Use forbid mods (3.3(f)); assess the terms before use. This project contains no game binaries, saves, generated catalogues or credentials and does not modify the Steam DRM stub.

## Endpoints and tools

The server endpoint is `POST http://127.0.0.1:8751/mcp`. Its mod ID is `enshrouded.mcp`, side `server`. Joining players do not need this server-only mod.

| Tools | Scope |
| --- | --- |
| `status`, `players` | Hosting state and connected players |
| `voxel_sample`, `props_in`, `entities_near`, `entity_components` | Bounded spatial inspection; component names require a locally generated catalogue |
| `capabilities`, `inventory_read`, `item_info`, `item_lookup`, `entity_health` | Available APIs, linked inventory pages, vanilla metadata and typed health observations |
| `knowledge_search` | Curated evidence with provenance and applicability |
| `recipe_plan` | Pure single-output recipe arithmetic over supplied recipes and stock |
| `building_plan` | Floor, wall and doorway previews on a half-metre grid; no placement |
| `building_survey` | At most 4096 complete half-metre cells with exact material/fill, persistence and optional connected-player build rights; no edits |
| `building_place_cell`, `building_apply_plan` | Gated exact-voxel copy into one air cell or up to 64 floor/wall/doorway cells in one aligned 8-cube brick |
| `building_operation`, `building_undo_cell` | Session-bound native transaction reconciliation and guarded undo for cells or plans |
| `give_item` | One explicitly gated inventory transaction |

The optional research companion uses `POST http://127.0.0.1:8754/mcp` and mod ID `enshrouded.mcp-client`. It exposes `client_capabilities`, `client_state`, `client_events`, `client_item`, `client_recipes` and `client_selection_events`. The framework requires side `both`: matching companion installations are needed on the isolated research server and its clients. The dedicated-server half is a no-op. The companion copies native UI snapshots and named client events; it provides no desktop automation or gameplay actions.

Current selected tab and passively highlighted recipe remain unknown. The outer Backpack screen is insufficient to identify the Crafting tab. Observed event delivery does not prove current selection or successful crafting.

## Build

Requires Windows MSVC build tools, Rust 1.99.1 and Python 3 for helper tests. Rust is pinned in `rust-toolchain.toml`.

This snapshot uses enshwick alpha.14 with a reproducible local framework extension, ABI 11.20. Prepare the sibling SDK checkout exactly as [patches/README.md](patches/README.md) describes. Cargo's path patch deliberately requires that sibling checkout. The upstream PR is separate from this project; no released framework support is claimed.

The private 11.20 extension and current-main 11.28 extension have different ABI lineages. Do not load a private-11.20 consumer under a newer official/main loader merely because its version number is higher. Rebuild consumers against the matching SDK before changing loader lineages.

```text
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
python -B -m unittest discover -s scripts -p "test_*.py"
cargo build --workspace --release
```

## Install and configure

Back up the loader, mods and a cold isolated-world save before testing. Exit the client and coordinate disconnected players before replacing loaded DLLs. An empty-player query alone can race with reconnecting players. Do not test new writes on a shared world.

Install the matching extended framework loader according to its instructions. Do not use `--with-loader`, which can replace it with an incompatible stock loader. Then install the mod into a chosen game or dedicated-server folder:

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

Construction uses a dry preview session token, validated operation identity and stable logical cell keys. Durable intent is synced before commit; native transaction IDs remain typed in memory. Repeating the same canonical operation polls its receipt rather than committing again. A prior-process tombstone prevents replay but cannot reconstruct native IDs or provide stateful recovery. A lost undo response is reconciled without requeueing undo. Only one unsettled proof is allowed.

Plan placement requires every target cell to be air, one connected actor, build rights and persistence across the full target bounds. Each native change writes exactly one requested cell. Keep the isolated site quiescent through placement and undo; do not run other writes during a proof. Native placement has no compare-and-swap and no atomicity guarantee. The framework can absorb a foreign voxel edit during multi-change application into its recorded postimage. Version 0.9.1 therefore refuses multi-cell undo unless the entire expected voxel postimage still matches; props, persistence and the native undo conflict check remain required. This guard is not a general concurrent-edit safety guarantee. No inventory debit, collision, traversal, stateful prop recovery or applied-structure persistence is established.

## Validation and roadmap

Version 0.7.0 adds bounded read-only site surveys. Supply `x`, `y`, `z`, `width`, `height`, `depth` on the 0.5 m grid, with sides at most 8 m. Optional `player` selects one connected name/account key for rights. Voxel output uses x-fastest, then y, then z order. Unloaded reads return errors; unavailable persistence/rights stay unknown. Runtime voxel material IDs are not inventory item IDs or a validated palette. The zone check is not save/reload proof, and an allowed observation does not authorize a later write. Props, collision, traversal, survival costs and complete altar boundaries remain unchecked.

The focused framework contribution is [enshwick PR #1079](https://github.com/enshwick/enshwick/pull/1079). It contains no MCP code and does not replace this snapshot's pinned compatibility patch automatically.

The 0.9.1 source passed 119 Rust tests, 24 Python helper tests, Clippy with warnings denied, formatting and a release build. The original research-ID isolated installation passed all 18 fixed read-only probes. Neutral publication IDs have not been exercised live.

A single-cell proof passed exact whole-capture placement/read-back/undo, idempotent receipt replay and cold reload with all 512 baseline voxels equal. The 16-cell plan live proof remains pending. Undo-then-reload baseline equality does not prove persistence of an applied structure.

The source has limited isolated live evidence. Known inventory splits, one durability comparison, one enemy health decrease and cold file restoration have been observed. These findings do not establish general mechanics or complete object-state recovery. See [public evidence](plans/research/live-validation.md), [the construction roadmap](plans/goal.md) and [the queue](plans/next-up.md).

`python -B scripts/smoke.py` checks baseline reads. `scripts/investigate.py`, `scripts/validate_tools.py` and `scripts/client_probe.py` perform fixed read-only probes and save private reports under ignored `gamedata/`. Passing transport probes does not satisfy fixture acceptance. Run `--help` for options.

Generated type catalogues and component-name lists must be produced locally through enshwick's published tooling and remain in ignored `gamedata/`. No generated game catalogue is included.

## License

MIT. The framework patch retains its upstream license in [patches/enshwick-LICENSE](patches/enshwick-LICENSE). This project is not affiliated with Keen Games or enshwick.
