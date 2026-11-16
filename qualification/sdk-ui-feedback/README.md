# Inactive ABI 12.4 UI feedback qualification

This optional test workspace pins published SDK main dc0cf4ce584c1f8ac7d4aee0cd0cc837cfc8bf65, ABI 12.4 and Rust 1.100. It is outside the four-member product workspace and supplies no MCP tool, mod DLL or installed provider. The product remains official release 6a151a95 / ABI 12.1 / Rust 1.99.1, installed server 0.42.0 / companion 0.20.0.

The checked release list still starts with alpha.17 at 6a151a95. Main adds terrain-material registration at slot 207, interaction prompts at 208 and item-tooltip registration at 209. Existing client command actions are unchanged. Frame::hovered_slot observes the game's inventory hover; Frame::item_slot draws a mod-owned slot and reports its input. Neither supplies vanilla item use, crafting, server join or logout. Load::extend_item_tooltip observes entity-backed items and permits mod lines; this prototype adds no lines.

## Consumer contracts

Four tests pass in debug and optimized builds, strict Clippy and formatting. The consumer copy functions first fail because they are absent, then preserve client inventory/item identities, raw slot reference, index, item ID/count, zero values, optional item entity and unknown category. Raw slot references use fixed hexadecimal strings so even synthetic u64 extremes do not lose JSON numeric precision. A missing/zero-reference hover clears the copied value. Inventory category remains the SDK's INVALID value rather than an inferred backpack category.

Tooltip copies preserve only supplied item identity. Count is unknown. A callback is historical evidence, not proof the tooltip remains visible or an item is selected. Entity-free items produce no callback in this model. Server registration refuses. No widget, tooltip line or native client command is submitted by these observers.

SDK limitations stay explicit: None conflates no hover with unsupported/not-observed native data. The fake allows supplied hover data on any named screen, so these tests do not prove native screen restrictions. Its open_tooltip helper invokes callbacks directly and does not model the documented native once-per-item-change frequency. This workspace implements no freshness/session cache, item-use/crafting action or MCP endpoint. Native provider compatibility, rendering and joined-session behavior remain unverified. Future MCP integration must clear on context changes, stamp observations, preserve client/server identity separation and retain unknown availability.

Run from the product root:

```powershell
cargo +1.100.0 test --manifest-path qualification/sdk-ui-feedback/Cargo.toml
cargo +1.100.0 test --release --manifest-path qualification/sdk-ui-feedback/Cargo.toml
cargo +1.100.0 clippy --all-targets --manifest-path qualification/sdk-ui-feedback/Cargo.toml -- -D warnings
cargo +1.100.0 fmt --check --manifest-path qualification/sdk-ui-feedback/Cargo.toml
```

The checks used an installed stable compiler reporting exact Rust 1.100.0. Named toolchain installation is separate from these source tests. The lockfile freezes dependency resolution.

## Isolated consumer migration candidate

The [consumer patch](consumer-candidate.patch) applies to exact product publication base 2dbb43a43a708bd27269800f15b2f57aa4ffb9b3. Apply it only in an isolated source checkout, preserving the active product. It changes the SDK/toolchain pin, truthful compiled provenance, expected probe provenance and fake snapshot fixtures. Product source 6a151a95 remains untouched.

Compared with the product SDK, the newer fake starts copied snapshot sampling only after the first ask and a subsequent interface step. Four candidate client tests initially fail on Never. Candidate-only fixtures request sampling and draw one fake interface step before fresh-state expectations, including negative-context tests so they cannot pass merely because sampling never started. No production gate or deadline is weakened.

All 363 candidate Rust executions, 67 Python tests, strict Clippy, formatting and release builds pass. The exact-base patch applies to a temporary Git index; every one of its eleven changed files is accounted for. A build-source copy must include the existing plans/knowledge/claims.json include_str asset. No AGENTS/plans scaffold is required.

After applying the patch in that isolated checkout, run the normal workspace checks with Rust 1.100.0 and both release builds. Stop at source qualification: no installer is part of this procedure. A matching native ABI 12.4 provider, its source/binary provenance, guarded-header compatibility, hosted-world thread-role changes and native client feedback must be qualified before migrating the product or loading candidate binaries. Numeric ABI assertions and model success do not supply that proof.

The older prop-placement proposal targets ABI 12.2 against 70a468bd and now collides with the official terrain slot. It remains inactive and needs an independently verified rebase, proposed ABI 12.5/slot 210, before submission. No upstream issue, PR or message has been sent.

## Provider artifact checkpoint

Official dev-build run 43860821079 succeeds for dc0cf4ce. Artifact 14224176074 matches its GitHub-reported SHA-256 digest. Its nested SDK package contains LICENSE and two DLLs, with no detached signature sidecars. The SDK signature-verifier source is unchanged from the trusted release, but verification cannot complete without signatures. Files are staged privately only. No native ABI query or provider installation occurred. This supplies provenance for a dev candidate without detached SDK signatures, not accepted runtime compatibility or the signed-provider trust level of the installed release. Review the supported distribution/trust path and qualify native behavior before any migration; do not silently bypass signature checks.
