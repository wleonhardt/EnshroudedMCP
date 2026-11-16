# 0024: Official alpha.17 native client-control foundation

Status: source qualified; matched deployment and live acceptance pending.

Pin official release 6a151a9550ed7a6ade99c23fa817a91e25b71bc9 (alpha.17, ABI 12.1), which retains Rust 1.99.1. Use Cargo's exact Git dependency without a local source override. The placed-template getter is upstream and returns Option<Guid>; historical metadata patches must not be applied.

Server 0.34.0 and companion 0.13.0 update compiled provenance together. Required source tests, lint, formatting and release builds pass. Targeted isolated SDK tests cover 33 client/window cases, 51 unstable action cases and six signature cases; this is not a full upstream workspace run. Official release archive SHA256 is 066a8c149570abf60157126b7258516a706c9687544e8a136d5835e75169eac5. Both framework signatures verify and tampered binaries fail verification. Installed files remain unchanged.

This foundation exposes no new MCP executor. Next implement bounded native snapshot and command adapters, with game-free admission, fresh session/screen binding, deliberate client write gates, audit and once-only outcome reconciliation. Native commands execute later in the framework's client phase. They grant no server World authority. Observed movement completion does not prove arrival or rendered results.

The public API does not provide main-menu server join, protected credential entry, logout, process lifecycle or screenshots. Keep these gaps explicit. Qualify one matched cold deployment after adapters, preserving binary/signature rollback and isolated world data. Never infer live loader identity from compiled provenance.
