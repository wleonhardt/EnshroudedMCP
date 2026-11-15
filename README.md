<p align="center"><img src="docs/assets/enshrouded-mcp-banner.svg" alt="Enshrouded MCP — Observe. Plan. Build. Verify." width="1200"></p>

# Enshrouded MCP

Inspect a running Enshrouded world, preview bounded building plans, and carry out deliberate administrative edits through MCP.

This unofficial research project connects an MCP client to [enshwick](https://github.com/enshwick/enshwick)'s published native API. It combines loopback transport, game-thread execution, tested write gates and journaled operation identities. Current development versions are **server 0.20.0** and **optional companion 0.5.1**.

[Setup](docs/setup.md) · [23 server tools](docs/tool-reference.md) · [Architecture and safety](docs/architecture.md) · [Evidence and roadmap](docs/research-status.md) · [Contributing](docs/contributing.md)

## What it can do

| Capability | Current scope |
| --- | --- |
| Observe | Players and nullable native vitals, inventory pages, item metadata, props, nearby entities and exact voxel surveys |
| Plan | Pure recipe arithmetic and half-metre building geometry, with explicit unknown requirements |
| Build | Copy an observed building voxel into air; combine up to eight named floor, wall and doorway parts |
| Verify and recover | Compare complete modified bricks, reconcile a retained transaction, accept a verified result and request guarded undo |
| Reposition | Submit one gated administrative teleport request for the sole connected living player; observe position separately |
| Inspect client UI | Optional companion observes native screen, prompt, host and named events without desktop automation |

Construction is bounded to **256 unique cells, eight aligned bricks and 64 cells per brick**, within **16 m per axis**. Parts form a union; doorway openings omit cells rather than erase other contributions. Material and fill come from a real source voxel. No survival inventory cost is claimed.

A composed 164-cell shell has rendered placement and exact recovery evidence within a 4096-cell baseline. That is a geometric shell, with a raised entrance—not a usable workshop with stairs, doors, storage and stations. Repositioning submits a native request; engine spawn correction has no published radius. It does not provide walking, camera or menu control.

## Get started

1. Use a disposable isolated world and coordinate access with its players.
2. Prepare the exact framework source and matching official loader using [setup](docs/setup.md). The pin is `acd5550c607a38c983433f9807cc95564244fe0b`, package alpha.15, **ABI 11.30**. The alpha.15 release tag uses a different ABI.
3. Apply the three-file [SDK metadata patch](patches/README.md), then build and test both consumers you intend to install.
4. Install the server mod with writes disabled. Connect your MCP client to `http://127.0.0.1:8751/mcp` and begin with `capabilities`, `status` and `players`.

The endpoint always binds loopback. Remote use requires an authenticated tunnel. An optional bearer token adds request authentication; it does not replace the write gate. The client companion has a separate endpoint and token.

## Deliberate writes

Real writes require a fresh `ALLOW_WRITES` file in the mod folder and `confirm: "server-write"`. Remove the flag when the controlled session ends. Construction also binds a preview to the active world session, checks the sole actor's rights and persistence, and records intent before native commit. Reusing an operation identity reconciles its outcome; it does not silently place again. Native receipts expire with the session.

Most administrative tools default to dry preview. **`give_item` does not; pass `dry: true` explicitly.** Keep agents read-only until you understand each tool's contract. Empty-player checks alone do not prevent reconnection during maintenance.

## Development and evidence

```text
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
python -B -m unittest discover -s scripts -p "test_*.py"
cargo build --release -p mcp -p mcp-client
```

The development checkpoint passed **173 Rust tests and 30 Python tests**, plus Clippy, formatting and release builds. The pinned SDK passed 592 tests under its prescribed nextest harness. Fixed live read probes and small isolated write fixtures provide additional evidence; passing transport probes does not establish gameplay semantics or complete acceptance. See the [evidence boundaries](docs/research-status.md).

## License and affiliation

Project code and original geometric SVG artwork are MIT licensed; see [LICENSE](LICENSE). Retained enshwick material has its own [MIT attribution](patches/enshwick-LICENSE). The artwork is original and contains no game artwork.

This project is not affiliated with Keen Games. Keen's Terms of Use prohibit mods in section 3.3(f); review the applicable terms before use. No game binaries, saves, credentials or generated game catalogues are included. The project relies on published framework APIs and does not modify the Steam DRM stub.

Server 0.20.0 adds bounded prop template and nullable label-presence observations; this increment has offline tests but is not live-installed. Server 0.19.0 native Air-only pier preview and companion 0.5.1 typed recipe branches have limited isolated evidence. Neither proves rendered terrain contact, category membership, prop recovery or crafting feasibility. Enemy pacification is a guarded research setting, not invulnerability. Native vanilla menu/movement/quit execution remains unavailable; desktop fallbacks do not count as native acceptance. Follow [phased goals](plans/phases.md).

Companion source 0.5.2 fixes selection-event cursor gap reporting after session-clear. Cursors older than cleared observations report a gap before any new event. Overwrite counts retain their existing meaning. This patch is not live-installed.
