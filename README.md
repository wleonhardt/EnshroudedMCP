<p align="center"><img src="docs/assets/enshrouded-mcp-banner.svg" alt="Enshrouded MCP — Observe. Plan. Build. Verify." width="1200"></p>

# Enshrouded MCP

Inspect a running Enshrouded world, preview bounded building plans, and carry out deliberate administrative edits through MCP.

This unofficial research project connects an MCP client to [enshwick](https://github.com/enshwick/enshwick)'s published native API. It combines loopback transport, game-thread execution, tested write gates and journaled operation identities. Current development versions are **server 0.23.1** and **optional companion 0.8.0**.

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

Companion source 0.5.2 fixes selection-event cursor gap reporting after session-clear. Cursors older than cleared observations report a gap before any new event. Overwrite counts retain their existing meaning. Isolated research-namespace installation passes fixed native probes; actual join/logout loss reporting remains unverified.

## Companion 0.6.1 recipe-category observations

`client_recipes` now adds `category_membership` to each returned `input_observations` row. State is `observed`, `not_found`, or `no_category_id`; unresolved states have `members: null`. Observed categories contain ordered `members` with `item_id`, nullable `scaling`, `scaling_finite`, and exact `scaling_bits`, plus `total_members`, `truncated`, and `scaling_semantics_verified: false`. Member entries are archive observations; no unique-member, effective-cost, eligibility or UI-selection claim is made. Non-finite scaling remains null in JSON with its bits preserved.

The parser admits a 16 MiB registry, at most 4,096 category rows, 1,024 member records per category and reflected strides from 1 to 4,096 bytes. Every category header/member span is admitted before indexing; duplicate category IDs refuse. Only the first 32 members of categories referenced by returned recipe inputs are decoded. Whole upstream registry allocation still precedes admission. All array/field offsets derive from published reflection, including typedef resolution; no game offsets or new hooks are used.

Original research-namespace companion 0.6.1 is installed isolated-only; server remains 0.19.0 and official framework acd5550c/ABI 11.30 is unchanged. Nine companion and eighteen installed-version server probes pass. A Workbench category returns six members whose item IDs independently resolve through the server vanilla item registry. This is limited static membership evidence; scaling interpretation, displayed choices and full crafting feasibility remain unverified. Both complete world captures and archived ledgers match; native client stays at Menu, zero players and writes disabled. Public namespace binaries are not live-installed.

## Native world clock

Server 0.21.0 adds `status.clock`: `state` (`observed`/`unavailable`), `source: "published_server_clock"`, nullable `observation`, and `rendered_lighting_verified: false`. Observed fields are `hours`, `is_day`, `day_starts_hours`, `night_starts_hours`, `day_duration_seconds`, `night_duration_seconds`, and `speed`. SDK hours and speed are preserved; SDK durations are expressed in seconds, including fractions. Unavailable observations are null, not midnight or a zero speed. This supplies experiment timing evidence without asserting current rendered lighting or a safe gameplay condition.

The isolated original research namespace now runs server 0.21.0/companion 0.6.1, official acd5550c/ABI 11.30. All nineteen server/nine companion fixed probes pass, including a real daytime clock read; complete world captures and archived ledgers match after a verified cold archive. Native client stays at Menu with zero players and writes disabled. Nonempty prop template/label rows, connected-client rendering and neutral-namespace live installation remain unverified.

Companion source 0.7.0 adds bounded declared prop/shelter requirement observations. It passes offline checks but is not live-installed; current eligibility/full requirements remain unknown. See [tool reference](docs/tool-reference.md).

Server source 0.21.1 fixes queued give admission and post-timeout callback journaling. This patch is not live-installed; audit IDs do not provide replay protection. See [grant contract](docs/tool-reference.md).

Server source 0.21.2 additionally refuses ambiguous grant recipients; no roster-order tie breaking. This source patch is not live-installed.

Server source 0.21.3 corrects compiled framework provenance in `capabilities`; fixed probes check the pinned revision, ABI and framework package version. Compiled metadata does not verify the installed loader.

`player_restore_health` (server 0.22.0) accepts `player`, optional `dry` (default true), `preview` and `confirm`. Dry returns a preview bound to the sole living server actor and observed health. A real request requires that exact preview, `dry:false`, a fresh `ALLOW_WRITES` flag and `confirm:"server-write"`; it restores health to the observed maximum through the published SDK. Read-back, journaling and pending cancellation do not provide invulnerability, undo or replay protection. Inspect fresh `players.vitals` and journal after unknown outcomes; never blindly retry. One bounded isolated research-namespace fixture restored native health from 17/100 to 100/100 with independent read-back, a full rendered bar and clean logout. This does not establish general combat safety or public-namespace live installation.

## Native map control checkpoint

Companion 0.8.0 implements bounded transient map-marker submission and explicit clearing. The isolated joined-client fixture accepted show/clear/show and automatic observed-leave cleanup, preserving empty submitted state after logout. Rendered visibility remains pending because fallback map shortcuts did not open the map. Prioritize supported native join/private entry, map/menu opening and logout executors; do not equate cached observations with control. See [tool reference](docs/tool-reference.md).

### Rendered map follow-up

A separate isolated fixture opened World/map through the authorized desktop fallback. Native marker show and clear were accepted, but no distinct annotation label/icon was observed at the near-player offset, including after reopening the map. Existing world/player icons remained visible. The small offset may overlap existing icons at this scale; the cause is unresolved. Next use a clearly separated bounded annotation and observed hover before diagnosing the native render path. No fast travel or saved-marker editing was exercised.

Explicit clear preceded logout; research settings were restored and flags removed. Complete baseline captures, operation-ledger membership/hashes and installed binaries remained exact. This adds four companion audit records and four server research-setting records. SDK submission remains verified separately from rendered visibility, which is still unaccepted. Supported native map opening and rendered feedback remain roadmap priorities.

Site-access source increment: server 0.23.0 adds read-only `building_access` for bounded explicit boxes and optional unique actors. It reports native rights/persistence without claiming item footprints, collision or verified saves. Deployment/live acceptance pending; isolated runtime remains 0.22.0/0.8.0.

## Site access live checkpoint

The isolated research pair now runs server 0.23.0/companion 0.8.0. A verified 47-file cold archive and two rollback copies preceded server-only replacement; six companion/framework/loader hashes remained unchanged. Nineteen server and nine companion fixed probes pass, with incomplete verdicts preserving their semantic limits.

Read-only building_access observed the native persistence rule true for a known fixture box and false for a distant box. Missing actor, unexpected write fields and reversed corners refused. A bounded joined fixture with native full health observed allowed build rights and persistence true for an explicit half-metre box. The separate construction dry preview could not load its source region, so exact preview comparison remains unverified. No construction write or reservation occurred.

After logout and explicit restoration of the research setting, complete prior captures, operation ledgers and installed binary hashes remain exact. The prior server journal prefix has exactly four expected setting records; the companion journal is unchanged. Flags are absent and zero players remain. This validates limited native rule observations, not item footprints, collision/contact, complete base-boundary behavior, save/reload effects or public-namespace installation.

Server source 0.23.1 aligns site-access name matching with the pinned SDK while preserving unique-actor admission. Deployment remains pending; isolated runtime stays 0.23.0/0.8.0.
