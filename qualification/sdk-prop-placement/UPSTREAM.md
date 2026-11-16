# Upstream submissions

- API request: [#1139 — Expose read-only prop placement metadata](https://github.com/enshwick/enshwick/issues/1139).
- Source-only PR, ready for review: [#1141 — feat(world): expose read-only prop placement metadata](https://github.com/enshwick/enshwick/pull/1141).
- Previous diagnostic draft, now ready for review: [#1111 — test(input): cover cached sampler diagnostic boundaries](https://github.com/enshwick/enshwick/pull/1111).

The user explicitly authorized these submissions and the ready transition. No PR was merged.

The placement draft is signed head cee4415f8408f6b9542779821934c217f1ea6b83 against exact main 73c18f31. It appends proposed ABI 12.6/slot 211, preserving official slots and the permissions API. It reuses the resolver and copies raw template/corners without props or transactions. Unknown bounds remain unknown. Full geometry, collision, permission, ownership, persistence and recovery remain separate concerns.

The scoped check passes 4,237 model tests and nine marked skips, lint, formatting, affected doctests, C/C++ layout/header, generated Luau declarations, rustdoc and fresh local-target links. Source and committed secret audits, exact 29-file allowlist, fresh apply/tree proof and both hooks pass. Broad trybuild is left to CI; 61 external book links were not checked. No native provider/value acceptance or runtime installation is claimed.

The diagnostic implementation was adopted upstream in #1118. The refreshed #1111 preserves that implementation and history, and adds only SDK/Luau age-boundary and unknown-flag regressions plus a change fragment. Signed head b96e968baeda865631711c83a32af9ade00a5236 is open, ready and mergeable. Full checks pass 4,256 tests, nine skips and both compile tests, including internal-access trybuild. Fresh local-target rustdoc/link checks also pass. There is no production or ABI change.

The old 70a468bd/ABI 12.2 patch remains a historical reference. Neither proposal is active in the MCP or installed framework. Product SDK 6a151a95 and the qualified runtime remain unchanged.

Ready checkpoint: the user requested review readiness on 2026-11-16. PR #1141 is now open, ready for review and mergeable at the same verified signed head. This does not claim native acceptance or authorize merging.
