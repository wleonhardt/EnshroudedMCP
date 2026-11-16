# SDK settlement and prop-recovery qualification

Optional, independent test workspace for official enshwick revision `70a468bd4d0001f5e7237bf09ff3b3b0ffc04fc2`. This is not the MCP runtime dependency. The main project retains its qualified SDK and Rust pins.

The tests use the published fake host only. They do not launch, connect to or modify a real game/server. Three tests check the callback signature and numeric ABI, seven callback cases, and prop recovery contracts. The prop fixture checks two placement orientations, nine held-state kinds at queued write and undo boundaries, and an empty-state undo control. Run with Rust 1.100.0 and the Windows MSVC tools:

```powershell
cargo +1.100.0 test --manifest-path qualification/sdk-settlement/Cargo.toml
cargo +1.100.0 test --release --manifest-path qualification/sdk-settlement/Cargo.toml
cargo +1.100.0 clippy --manifest-path qualification/sdk-settlement/Cargo.toml --all-targets -- -D warnings
cargo +1.100.0 fmt --manifest-path qualification/sdk-settlement/Cargo.toml -- --check
```

The same compiler version was exercised through an installed stable toolchain. This crate is excluded from the MCP workspace by its own `[workspace]` boundary and changes no installed dependency or product toolchain.

The scenarios preserve these proof boundaries: a bound is not cancellation; an Applied receipt can deliver past its bound; expired undo data can still yield a standing ID; another transaction can remove that entity before callback delivery; failure is not success; Undoing is not completed undo; and a game-scoped wait can disappear without callback on world exit. Tests assert SDK-model behavior, not native provider fidelity, rendering, complete postimages or recoverability.

See [the qualification decision](../../plans/decisions/0039-sdk-settlement-qualification.md). Native loader compatibility and the complete MCP/game acceptance remain separate gates.

The prop tests use synthetic item metadata and held-state flags. They verify that a rotated long placement box can be denied away from its allowed anchor, and that state acquired after queuing prevents the entire mixed prop write or undo. All nine published held-state kinds are covered, including CustomPlayerString. The empty-state control completes undo. These tests do not read native component state, establish exact label ownership, expose a placement-box getter, prove physical geometry or establish atomicity in the real loader. In particular, adding a label can itself introduce a held-state recovery obstacle; this suite grants no ownership exception.
