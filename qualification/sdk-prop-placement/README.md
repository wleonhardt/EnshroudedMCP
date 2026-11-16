# Inactive SDK prop placement proposal

This is a source proposal, not an official SDK API, an active MCP patch or an installed loader. The MCP keeps its qualified SDK revision 6a151a9550ed7a6ade99c23fa817a91e25b71bc9, ABI 12.1 and Rust 1.99.1. Installed server 0.41.0 / companion 0.18.0 are unchanged.

The [patch](../../patches/enshwick-prop-placement-read-proposal.patch) applies only to the qualified source baseline [70a468bd](https://github.com/enshwick/enshwick/tree/70a468bd4d0001f5e7237bf09ff3b3b0ffc04fc2). It proposes ABI 12.2 with one appended host entry and a headed 52-byte record. Every old host slot stays fixed. The SDK uses Rust 1.100.0; this does not change the product toolchain.

The proposed World::prop_placement_info(item) returns a copied item/template and optional PropPlacementBox with declared min/max local corners. The native adapter reuses the existing item resolver without new engine offsets or signatures. It creates no prop or transaction. Missing, invalid or unreadable bounds remain unknown. Corners are not normalized, and no fallback permission box is substituted. Rust, C header and generated Luau declarations are coordinated.

These are placement metadata, not proof of complete physical geometry, collision, permission, persistence, exact label ownership or safe recovery. A future MCP adapter must still run its game read on the server game thread. The SDK registry pin does not relax MCP thread or write policy. No new MCP tool or executor is exposed here.

## Qualification

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

The patch retains the SDK's [MIT attribution](../../patches/enshwick-LICENSE). The [upstream request and PR text](UPSTREAM.md) are review drafts only. No issue, PR, maintainer message, runtime installation or consumer migration has been submitted or performed.
