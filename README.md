# EnshroudedMCP

An experimental [enshwick](https://github.com/enshwick/enshwick) mod that exposes a loopback MCP endpoint for inspecting Enshrouded. Server reads run on the game thread. Request admission, argument bounds and write gates are implemented in a game-free policy crate.

This research prototype has read tools, pure planning tools and one gated gameplay write, `give_item`. Administrative construction is the next implementation priority; plan placement, idempotent jobs and building undo are not implemented. Recipe planning does not establish live crafting feasibility.

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

## Validation and roadmap

The source has offline tests and limited isolated live evidence. Known inventory splits, one durability comparison, one enemy health decrease and cold file restoration have been observed. These findings do not establish general mechanics or complete object-state recovery. See [public evidence](plans/research/live-validation.md), [the construction roadmap](plans/goal.md) and [the queue](plans/next-up.md).

`python -B scripts/smoke.py` checks baseline reads. `scripts/investigate.py`, `scripts/validate_tools.py` and `scripts/client_probe.py` perform fixed read-only probes and save private reports under ignored `gamedata/`. Passing transport probes does not satisfy fixture acceptance. Run `--help` for options.

Generated type catalogues and component-name lists must be produced locally through enshwick's published tooling and remain in ignored `gamedata/`. No generated game catalogue is included.

## License

MIT. The framework patch retains its upstream license in [patches/enshwick-LICENSE](patches/enshwick-LICENSE). This project is not affiliated with Keen Games or enshwick.
