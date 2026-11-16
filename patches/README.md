# Clean official framework pin

This branch targets server 0.34.0 / companion 0.14.0 against official release `v1.0.0-alpha.17`, commit `6a151a9550ed7a6ade99c23fa817a91e25b71bc9`, ABI 12.1. Rust remains pinned to 1.99.1. Cargo uses the exact official Git revision directly; no local SDK patch or sibling checkout is required for consumer builds.

The release now includes `ItemInfo::placed_entity_template` as `Option<Guid>`. MCP preserves observed, known-no-template and unavailable metadata states without modifying the SDK. The release also provides native client snapshot and command/status/cancel APIs; this foundation increment alone exposes no new MCP executor. Native adapters and live acceptance remain separate work.

## Official loader qualification

Download `enshwick.zip` and `SHA256SUMS` from the official alpha.17 release. ZIP SHA-256: `066a8c149570abf60157126b7258516a706c9687544e8a136d5835e75169eac5`.

The ZIP contains `powrprof.dll`, `mods/enshwick/enshwick.dll`, their 64-byte `.sig` sidecars and the upstream license. Both signatures verify with the published `enshwick_gamedata::signature::verify`; altered bytes are refused. This verifies framework release signatures, not Steam files or Windows Authenticode.

Rebuild server and companion together. Stop the client and coordinate an empty isolated server before replacing loaded files. Preserve a verified cold save/ledger archive and complete binary/signature rollback accounting. Never replace the Steam DRM stub. Source checks do not prove successful loader startup, client compatibility or gameplay effects.

## Historical inactive patches

`enshwick-abi12-item-template.patch` belongs to alpha.16 commit `5436ffddcc623439c339a283a4dc2b45f924fe5b`, ABI 12.0. Its eight-file SHA-256 was `eb00f6d6cc41baefdc1ef721ada9ffc19c65c9c2ea085316f29aa8bb1c5cf331`. It is retained for historical reproduction only and must not be applied to the current release.

The earlier nightly metadata and private alpha.14 event patches are also inactive. Preserve their exact source/loader pairings if reproducing old evidence; package strings alone do not prove compatibility. No complete prop placement-box, runtime-label or held-state preflight getter is added by this migration. Administrative construction, stateful recovery and full workshop acceptance remain open.
