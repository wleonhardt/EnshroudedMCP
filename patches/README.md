# Pinned official framework and SDK metadata patch

The active MCP 0.14.0/companion 0.3.0 build pins official enshwick commit `5f647f46d7f987e93d397e00596014b23f3c1112`, ABI 11.30. Its package string remains alpha.15, but the alpha.15 release tag is ABI 11.28. Package version strings alone are not compatibility evidence.

## Reproduce the SDK source

Use a dedicated sibling checkout named `enshwick-nightly`; do not overwrite an existing dirty framework checkout. From this project folder, prepare the exact public source and apply only the active patch:

```text
git clone --no-checkout https://github.com/enshwick/enshwick ../enshwick-nightly
git -C ../enshwick-nightly checkout --detach 5f647f46d7f987e93d397e00596014b23f3c1112
git -C ../enshwick-nightly apply <absolute-path-to-this-project>/patches/enshwick-nightly-item-template.patch
```

Cargo's override points to `../enshwick-nightly/crates/enshwick`. The active patch changes only the inventory reader, public slot metadata and inventory regression tests. It exposes the already published native placed-template field through Rust item metadata, preserving GUID file order and distinguishing known absence from unavailable metadata. It does not append an ABI function, change native layout, add a game offset or alter the loader. The focused regression first failed with missing-field E0609; the subsequent SDK nextest run passed 538 tests. Rebuild both MCP binaries against this source.

Active patch SHA-256: `a91b4ea0cef5b606b1e3c077ac1c1ad12da1b001cb7abf5795d58add3ed624ae`.

## Matching official loader

Use the official artifact from [development run 43608939268](https://github.com/enshwick/enshwick/actions/runs/43608939268), for the exact pinned commit. The official [workflow](https://github.com/enshwick/enshwick/blob/5f647f46d7f987e93d397e00596014b23f3c1112/.github/workflows/dev-build.yml) builds and uploads the commit-specific artifact. The outer artifact ZIP SHA-256 was `817b8c46e86e9670b13404fc49d3d8ecdf2e88c9f71405b9a9aa164e93b841bb`, matching [GitHub artifact metadata](https://api.github.com/repos/enshwick/enshwick/actions/artifacts/14119696639). The inner `enshwick.zip` SHA-256 was `9fed544ed004a42633a1c4198ab02f7e87acfe73e7e71bd2f3146070d4628b16`. Artifacts may expire; these checksums identify the tested artifact rather than promising permanent availability.

Install the matching official loader stub and framework together with rebuilt server and companion binaries. Do not use `--with-loader` to substitute the alpha.15 release loader. Preserve rollback files and a verified cold isolated-world save/ledger archive. Coordinate disconnected players and exit both game processes before replacing loaded files. Never mix private SDK consumers with a later official function-table layout.

## Historical inactive patch

`enshwick-client-events.patch` is retained for the earlier private alpha.14 experiments at base `8fa382949551f0256a6505ed161cb28e5a0ea8f5`, ABI 11.20. Its combined 23-file form includes the client-event experiment and earlier inventory metadata exposure. It is not applied by the current build. Official named client-event subscriptions provide that route on the pinned official revision.

The private 11.20 append collides with later official table slots. Numerically newer minor versions cannot identify that collision; rebuild consumers against the matching SDK before changing lineages. Keep historical rollback pairs separate.

## Observation limits

Current selected tab and passive recipe remain unknown. Named events are observations, not persistent UI state or crafting authority. Official input masks and control bindings observe actions; they do not synthesize vanilla walking, camera, menu or lobby control. Screen-host templates are nullable observations and do not grant server authority.

The patch provides no complete placement-box or runtime-label getter. A mapped template does not prove prop eligibility, collision bounds, safe spawning, stateful recovery or workshop usability. Source tests and partial isolated observations are recorded separately in [public evidence](../plans/research/live-validation.md).

Upstream enshwick is MIT licensed; its license remains in `enshwick-LICENSE`. No game data, build tables, binaries, saves, raw screenshots or generated game catalogues belong in this directory.
