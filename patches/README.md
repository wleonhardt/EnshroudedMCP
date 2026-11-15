# Pinned official framework and SDK metadata patch

This branch targets server 0.26.0 / companion 0.11.0 against official enshwick commit `5436ffddcc623439c339a283a4dc2b45f924fe5b`, package alpha.16, ABI 12.0. The matching research namespace pair passes cold readiness and preservation; connected gameplay and public-namespace installation remain unverified. Preserve the earlier ABI 11.30 source and binary rollback pair.

## Reproduce the SDK source

Use a new dedicated sibling checkout named `enshwick-abi12`; preserve existing dirty framework checkouts. From this project folder:

```text
git clone --no-checkout https://github.com/enshwick/enshwick ../enshwick-abi12
git -C ../enshwick-abi12 checkout --detach 5436ffddcc623439c339a283a4dc2b45f924fe5b
git -C ../enshwick-abi12 apply <absolute-project-path>/patches/enshwick-abi12-item-template.patch
```

Cargo's override expects `../enshwick-abi12/crates/enshwick`. The patch exposes the existing published placed-template item field through Rust metadata, preserving GUID file order and distinguishing known absence from unavailable metadata. It changes the inventory reader, public metadata and inventory behavior tests. Because the public field participates in the SDK's Script derive, its generated Luau snapshot and three bundled example type files must also match. These generated SDK types contain no game catalogue or runtime data.

No native ABI slot, native layout, loader hook or game offset changes. The regression first fails with missing-field E0609, then passes after the wrapper is adapted to typed Guid. It verifies absent/present templates, unchanged item metadata and independence from PIDE. Full SDK checks also cover generated definitions; source checks do not establish live game behavior.

The full suite exposed two compile-fixture processes racing over the same trybuild directory. A scoped nextest group serializes only those fixtures while preserving separate processes and every compile assertion. Both fixtures pass with this configuration. This test-harness fix adds no runtime control.

Active eight-file patch SHA-256: `eb00f6d6cc41baefdc1ef721ada9ffc19c65c9c2ea085316f29aa8bb1c5cf331`.

The untouched upstream `cargo xtask check --all` baseline passed, including 4,049 main nextest tests and two compile fixtures. The final patched full command also passed: 4,050 main tests, both serialized compile fixtures, full/stable doctests and Clippy, documentation/link checks and cargo-deny. Nine main tests remain skipped by the upstream harness. Product checks pass 215 Rust tests, 35 Python tests, Clippy, formatting and both release builds. These are source results; candidate cold/live acceptance remains pending.

## Matching official development loader

Use the official artifact from [development run 43701282408](https://github.com/enshwick/enshwick/actions/runs/43701282408), built for the exact pinned revision. The outer ZIP SHA-256 is `cfbf51fe1e4d8905df22b255e1664237428c80f261ea516b3d4209dca31e81f5`, matching [GitHub artifact metadata](https://api.github.com/repos/enshwick/enshwick/actions/artifacts/14157390106). Its inner enshwick.zip SHA-256 is `3621b3eaa043e314395d702992c0673092c3fd9dc1c7c357d5a4193726733693`. Both ZIP CRC/member checks passed. Artifacts can expire; these hashes identify inspected files rather than promise permanent availability.

The development archive contains the framework DLL and loader stub but no release `.sig` files. It is an official CI development artifact, not a release-signature-verified distribution. Do not fabricate signatures or silently substitute a release loader. Preserve this distinction when staging or reporting installation.

The published loader source runs unsigned builds while marking them unofficial and logging E1011. That source behavior has not yet been observed live for this candidate; qualification requires no signature bypass.

Rebuild server and companion against the prepared SDK. Stop both game processes, coordinate disconnected players and prevent reconnection, preserve a verified cold save/ledger archive, and record rollback hashes before replacing any loaded file. Install matching framework, loader and consumers together. Never combine ABI 12 loader files with ABI 11 consumers. Do not use `--with-loader` to substitute an unrelated release loader. The research migration preserved a verified 47-file cold archive and all eight rollback binaries before replacement; eight installed hashes, complete prior captures/ledgers, fresh native Menu, zero players and disabled writes pass after restart.

## Historical inactive patches

`enshwick-nightly-item-template.patch` belongs to the earlier acd5550c ABI 11.30 SDK and must not be applied to this candidate. `enshwick-client-events.patch` belongs to historical private alpha.14 experiments at base 8fa382949551f0256a6505ed161cb28e5a0ea8f5, ABI 11.20. Neither is active here. Keep earlier source and rollback pairs separate; package strings alone do not prove compatibility.

## Observation limits

Input masks and control bindings observe actions; they do not synthesize vanilla walking, camera, menu, lobby or crafting input. Current inner tab and passive recipe selection remain unknown. Client IDs do not grant server authority. An answering companion endpoint with absent/stale frames does not prove a healthy rendered client; external crash/startup dialogs remain a documented desktop fallback.

The metadata patch provides no complete placement-box or runtime-label getter. A mapped template does not establish placement eligibility, safe spawning, stateful recovery, workshop usability or crafting eligibility. Preserve separate source, probe and live acceptance evidence.

Upstream enshwick is MIT licensed; its license remains in `enshwick-LICENSE`. No game data, build tables, binaries, saves, journals, screenshots or generated game catalogues belong here.
