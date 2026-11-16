# Inactive SDK prop placement proposal

This is a source proposal, not an official SDK API, an active MCP patch or an installed loader. The MCP keeps its qualified SDK revision 6a151a9550ed7a6ade99c23fa817a91e25b71bc9, ABI 12.1 and Rust 1.99.1. The original source-qualification checkpoint used server 0.41.0 / companion 0.18.0; consult the product README for the current installed pair.

## Submitted current proposal

API request [#1139](https://github.com/enshwick/enshwick/issues/1139) and PR [#1141](https://github.com/enshwick/enshwick/pull/1141) are open. The [current patch](../../patches/enshwick-prop-placement-read-abi128-proposal.patch) applies only to exact [b86d94d1](https://github.com/enshwick/enshwick/tree/b86d94d1c40dfba5661321276a29b989d80b8ba1).

Placement PR [#1141](https://github.com/enshwick/enshwick/pull/1141) remains open and ready at signed 1b647ee7. A normal merge preserves its published history and official main b86d94d1, including statistics and transaction settlement. The proposal appends ABI 12.8/slot 212 after settlement slot 211; the host table is 1,728 bytes and the metadata record remains 52 bytes. Exact 29-file source/commit audits, fresh apply/tree proof and hooks pass. The upstream check passes 4,268 model tests with nine skips, lint, formatting, interfaces, affected doctests and fresh local-target documentation links. Broad trybuild and stable-surface checks remain with CI; 61 external book links and native provider/value behavior remain unverified. No provider was installed. Product SDK 6a151a95/ABI 12.1 and installed 0.42.0/0.20.1 remain unchanged.

Reproduce with `cargo +stable xtask check --base b86d94d1c40dfba5661321276a29b989d80b8ba1` on Rust 1.100.0 after applying the exact-base patch. The [ABI 12.6 patch](../../patches/enshwick-prop-placement-read-abi126-proposal.patch) remains a historical artifact for exact 73c18f31; its old slot must not be applied to current main.

Use the workspace's own `target` directory for these checks: the SDK link checker currently expects `target/doc`. No consumer pin or MCP execution policy changes. A future MCP adapter must still use the server game-thread scheduler.

## Historical exact-base reference

The [patch](../../patches/enshwick-prop-placement-read-proposal.patch) applies only to the qualified source baseline [70a468bd](https://github.com/enshwick/enshwick/tree/70a468bd4d0001f5e7237bf09ff3b3b0ffc04fc2). It proposes ABI 12.2 with one appended host entry and a headed 52-byte record. Every old host slot stays fixed. The SDK uses Rust 1.100.0; this does not change the product toolchain.

The proposed World::prop_placement_info(item) returns a copied item/template and optional PropPlacementBox with declared min/max local corners. The native adapter reuses the existing item resolver without new engine offsets or signatures. It creates no prop or transaction. Missing, invalid or unreadable bounds remain unknown. Corners are not normalized, and no fallback permission box is substituted. Rust, C header and generated Luau declarations are coordinated.

These are placement metadata, not proof of complete physical geometry, collision, permission, persistence, exact label ownership or safe recovery. A future MCP adapter must still run its game read on the server game thread. The SDK registry pin does not relax MCP thread or write policy. No new MCP tool or executor is exposed here.

## Historical qualification

The new SDK regression first failed because the getter did not exist. It now checks copied values, repeated reads, unknown boxes, non-prop/unknown-item refusal, world-stop unavailability, raw inverted corners and independence from entity-command availability. Props and transaction lists remain unchanged. Generic guarded-header tests cover the new record.

The full upstream check passes against the exact baseline: 4,136 affected tests passed and nine were marked skipped. C/C++ header checks, Clippy, formatting, doctests, rustdoc, stable-surface checks and generated declaration checks pass. The tool did not check 61 pre-existing external book links without a separate mods clone. Optional game-input environment variables were absent; game-dependent cases that return early are not native-provider proof. Native values, provider compatibility and real prop behavior remain unverified.

Use a clean, disposable SDK checkout of the exact baseline. Keep this outside the product dependency and installed framework. Resolve the patch path before changing directories:

```powershell
$proposalPatch = (Resolve-Path patches/enshwick-prop-placement-read-proposal.patch).Path
$sdkCheckout = 'C:\path\to\sdk-checkout'
git -C $sdkCheckout checkout --detach 70a468bd4d0001f5e7237bf09ff3b3b0ffc04fc2
git -C $sdkCheckout apply --check $proposalPatch
git -C $sdkCheckout apply $proposalPatch
Set-Location $sdkCheckout
Remove-Item -LiteralPath Env:ENSH_GAMEDATA,Env:ENSH_SERVER_EXE,Env:ENSH_CLIENT_EXE -ErrorAction SilentlyContinue
cargo +1.100.0 test -p enshwick --features unstable,test --test main world_prop_placement
cargo +1.100.0 xtask check --base 70a468bd4d0001f5e7237bf09ff3b3b0ffc04fc2
```

Follow the upstream tool prerequisites, including LLVM clang, ast-grep and nextest. Empty environment values still opt into some game checks; remove the variables for model-only qualification. These checks used an installed stable compiler reporting Rust 1.100.0.

The patch retains the SDK's [MIT attribution](../../patches/enshwick-LICENSE). The [submission record](UPSTREAM.md) links the authorized current request and PR. This historical patch was not submitted or installed.
