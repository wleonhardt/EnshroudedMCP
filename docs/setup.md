# Setup

Use an isolated server and a disposable world for first installation. Stop processes before replacing their loaded DLLs. Coordinate disconnects and prevent reconnection during maintenance; a zero-player reading alone is not an admission lock. Preserve a verified cold save and ledger archive before changing a research installation.

## Requirements

- Windows x86-64 with the MSVC build tools and Windows SDK.
- Rust 1.99.1, selected by the repository's toolchain file.
- Python 3.11 or newer for standard-library helper tests and TOML profile checks.
- A local Enshrouded dedicated server or hosted game, plus the matching enshwick loader.

## Reproduce the pinned SDK

Cargo fetches official release commit `6a151a9550ed7a6ade99c23fa817a91e25b71bc9` directly, alpha.17, ABI 12.1. No sibling SDK checkout or patch is required for consumer builds. The repository retains Rust 1.99.1; newer upstream main uses a later toolchain and is not this pin.

The placed-template metadata getter is now official. Preserve unknown item metadata versus a known item with no placed template. Historical patches are inactive and must not be applied to this release. See [framework lineage and signature verification](../patches/README.md). Rebuild both consumers and use the matching signed loader and bootstrap with their `.sig` sidecars; cold/matched deployment and live checks remain separate gates.

## Build and install

If needed, build the matching Cargo subcommand from its exact official revision:

```text
cargo install --locked --git https://github.com/enshwick/enshwick --rev 6a151a9550ed7a6ade99c23fa817a91e25b71bc9 cargo-enshwick
```

Run the [development checks](contributing.md), then build in release mode. With the selected official loader already installed:

```text
cargo enshwick install -p mcp --game <server-folder>
```

The public server mod ID is `enshrouded.mcp`; its mod side is `server`. Joining players do not need this server-only mod. Keep the mod's `ALLOW_WRITES` file absent during initial read checks.

| Setting | Default | Meaning |
| --- | --- | --- |
| `ENSHROUDED_MCP_PORT` | `8751` | Server HTTP/MCP port; loopback only |
| `ENSHROUDED_MCP_TOKEN` | Unset | Optional server bearer token |
| `ENSHROUDED_MCP_CLIENT_PORT` | `8754` | Separate companion port; loopback only |
| `ENSHROUDED_MCP_CLIENT_TOKEN` | Unset | Separate optional companion bearer token |

Configure your MCP client for `POST http://127.0.0.1:8751/mcp`. Supply the bearer token through your client's credential settings if enabled. Never commit tokens or example files containing real credentials. Use an authenticated SSH tunnel for remote access; there is no public-bind option. This endpoint is not a web page and refuses browser-origin requests.

Start with `capabilities`, `status` and `players`. Inspect `capabilities` because optional SDK reads can be unavailable on a particular game build. Missing data is not zero or empty game state.

## Optional native companion

The companion package is `mcp-client`, mod ID `enshrouded.mcp-client`. It is an opt-in research component and is not required for server reads or construction. The pinned framework exposes it as side `both`, so install matching companion builds on the research server and each participating client. Its dedicated-server half is inactive.

```text
cargo enshwick install -p mcp-client --game <research-server-folder>
cargo enshwick install -p mcp-client --game <client-game-folder>
```

Exit the game before replacing its loaded files. Keep loader, SDK and companion builds matched. The companion endpoint is `http://127.0.0.1:8754/mcp`. It observes client UI; it does not give the client authority over the server world.

## Local catalogues and probes

`entity_components` requires a locally generated `components.txt` in the mod folder. Generate catalogues only through the framework's published tools, keep outputs private and follow `scripts/component_names.py --help`. No generated game catalogue ships here.

```text
python -B scripts/validate_tools.py --port 8751
python -B scripts/client_probe.py --port 8754
python -B scripts/check_contracts.py --server-port 8751 --client-port 8754
```

These fixed probes are read-only. A report may intentionally remain `incomplete` after every fixed check passes, because fixture and gameplay acceptance are separate. Reports stay in ignored `gamedata/`.

The historical alpha.17 / 0.34.0 / 0.15.0 isolated pair passed bounded native client-control and client-reload checks. Use the [capability matrix](../plans/capability-matrix.md) for current versions and evidence scope. This does not validate a public-namespace install, every command target, server lifecycle behavior or broader administrative writes. Keep binary and signature sidecars matched and preserve client-commands ledgers.

The contract checker without port arguments runs offline index/profile checks. Requested live scopes compare discovery plus fixed compiled metadata, not gameplay or runtime-provider behavior. Exit 0 means consistent, 1 drift and 2 unavailable. Its CLI reads no credentials; authenticated use requires an already configured Client passed to the helper API.
