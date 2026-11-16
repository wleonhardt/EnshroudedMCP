# Upstream submissions

- API request: [#1139 — Expose read-only prop placement metadata](https://github.com/enshwick/enshwick/issues/1139).
- Source-only PR, ready for review: [#1141 — feat(world): expose read-only prop placement metadata](https://github.com/enshwick/enshwick/pull/1141).
- Previous diagnostic draft, now ready for review: [#1111 — test(input): cover cached sampler diagnostic boundaries](https://github.com/enshwick/enshwick/pull/1111).

The user explicitly authorized these submissions and the ready transition. No PR was merged.

Placement PR [#1141](https://github.com/enshwick/enshwick/pull/1141) remains open and ready at signed 1b647ee7. A normal merge preserves its published history and official main b86d94d1, including statistics and transaction settlement. The proposal appends ABI 12.8/slot 212 after settlement slot 211; the host table is 1,728 bytes and the metadata record remains 52 bytes. Exact 29-file source/commit audits, fresh apply/tree proof and hooks pass. The upstream check passes 4,268 model tests with nine skips, lint, formatting, interfaces, affected doctests and fresh local-target documentation links. Broad trybuild and stable-surface checks remain with CI; 61 external book links and native provider/value behavior remain unverified. No provider was installed. Product SDK 6a151a95/ABI 12.1 and installed 0.42.0/0.20.1 remain unchanged.

The diagnostic implementation was adopted upstream in #1118. The refreshed #1111 preserves that implementation and history, and adds only SDK/Luau age-boundary and unknown-flag regressions plus a change fragment. Signed head b96e968baeda865631711c83a32af9ade00a5236 is open, ready and mergeable. Full checks pass 4,256 tests, nine skips and both compile tests, including internal-access trybuild. Fresh local-target rustdoc/link checks also pass. There is no production or ABI change.

The old 70a468bd/ABI 12.2 patch remains a historical reference. Neither proposal is active in the MCP or installed framework. Product SDK 6a151a95 and the qualified runtime remain unchanged.
