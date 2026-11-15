# Setup

Use an isolated server and a disposable world for first installation. Stop processes before replacing their loaded DLLs. Coordinate disconnects and prevent reconnection during maintenance; a zero-player reading alone is not an admission lock. Preserve a verified cold save and ledger archive before changing a research installation.

## Requirements

- Windows x86-64 with the MSVC build tools and Windows SDK.
- Rust 1.99.1, selected by the repository's toolchain file.
- Python 3 for the standard-library helper tests.
- A local Enshrouded dedicated server or hosted game, plus the matching enshwick loader.

## Reproduce the pinned SDK

From the project folder, use a new dedicated sibling checkout. Do not overwrite an existing dirty checkout.

```text
git clone --no-checkout https://github.com/enshwick/enshwick ../enshwick-abi12
git -C ../enshwick-abi12 checkout --detach 5436ffddcc623439c339a283a4dc2b45f924fe5b
git -C ../enshwick-abi12 apply <project>/patches/enshwick-abi12-item-template.patch
```

Replace `<project>` with this checkout's path. Cargo's source override expects `../enshwick-abi12/crates/enshwick`. The patch exposes an existing placed-template metadata field, updates generated Luau definitions and serializes shared-directory compile fixtures. It changes no native layout or ABI slot. Unknown item metadata and a known item without a placed template remain distinct.

The exact revision identifies **ABI 12.0**, package alpha.16. The previously qualified ABI 11.30 pair is incompatible. Cold research preservation and a bounded healthy connected read/pacification fixture pass; broader writes and public-namespace installation remain unverified. Use the matching official artifact and verify the identity and checksums documented in [patches/README.md](../patches/README.md). Historical private alpha.14 consumers are incompatible. Rebuild consumers before switching SDK/loader lineages. Do not use `--with-loader` to substitute a release loader for the pinned development loader.

## Build and install

Build the matching Cargo subcommand from the prepared source checkout. Its revision must match this pin; an available older command is insufficient:

```text
cargo install --locked --force --path ../enshwick-abi12/crates/cargo-enshwick
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
```

These fixed probes are read-only. A report may intentionally remain `incomplete` after every fixed check passes, because fixture and gameplay acceptance are separate. Reports stay in ignored `gamedata/`.
