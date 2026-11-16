# Upstream submissions

Both authorized PRs merged at 2026-11-16T17:30:23Z:

- [#1141: read-only prop placement metadata](https://github.com/enshwick/enshwick/pull/1141), final head 3c956e5131e340c4c4d4a55743537c721fedf1fd, merge 3a3e52f71cdf9a02ffc855cf9fc921e8ccbd5b0c.
- [#1111: cached sampler diagnostic boundaries](https://github.com/enshwick/enshwick/pull/1111), final head 3c2f3b7ef7a462b26bd9ef5b1bc5845f9e795423, merge 9f6efc821524df426991fb584a761eaa0e5ea898.

The optional consumer workspace pins the latter merged revision at ABI 12.8. Its eight model contracts and the bounded isolated native server experiment are recorded separately. The unsigned provider is not adopted and metadata is not exposed through the active product MCP. See [native evidence](../../plans/decisions/0068-isolated-native-sdk128-metadata.md). The product retains SDK 6a151a95 / ABI 12.1. See [provider qualification](../../plans/decisions/0059-upstream-merges-and-provider-qualification.md).

## Historical proposal qualification

The following counts belong to earlier source heads, not to a fresh qualification of the final merged heads.

The earlier placement qualification used signed 1b647ee7 before the final review refresh. A normal merge preserves its published history and official main b86d94d1, including statistics and transaction settlement. The proposal appends ABI 12.8/slot 212 after settlement slot 211; the host table is 1,728 bytes and the metadata record remains 52 bytes. Exact 29-file source/commit audits, fresh apply/tree proof and hooks pass. The upstream check passes 4,268 model tests with nine skips, lint, formatting, interfaces, affected doctests and fresh local-target documentation links. Broad trybuild and stable-surface checks remain with CI; 61 external book links and native provider/value behavior remain unverified. No provider was installed. Product SDK 6a151a95/ABI 12.1 and installed 0.42.0/0.20.1 remain unchanged.

The diagnostic implementation was adopted upstream in #1118. The refreshed #1111 preserves that implementation and history, and adds only SDK/Luau age-boundary and unknown-flag regressions plus a change fragment. Signed b96e968baeda865631711c83a32af9ade00a5236 was an earlier review head. Full checks pass 4,256 tests, nine skips and both compile tests, including internal-access trybuild. Fresh local-target rustdoc/link checks also pass. There is no production or ABI change.

The old 70a468bd/ABI 12.2 patch remains a historical reference. Neither proposal is active in the MCP or installed framework. Product SDK 6a151a95 and the qualified runtime remain unchanged.
