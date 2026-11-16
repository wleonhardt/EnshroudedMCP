# 0051: Preserve upstream ABI additions when refreshing placement metadata

Date: 2026-11-16. Status: source qualified and submitted; native provider unqualified.

Official main b86d94d1c40dfba5661321276a29b989d80b8ba1 adds transaction settlement at ABI 12.7/slot 211 after statistics work. This conflicts with the published source-only placement proposal. Merge main normally, preserving published history, every official field and existing slot. Append placement metadata at ABI 12.8/slot 212, and regenerate the header and golden layout together. Preserve the 52-byte metadata contract and raw unknown bounds.

Placement PR [#1141](https://github.com/enshwick/enshwick/pull/1141) remains open and ready at signed 1b647ee7. A normal merge preserves its published history and official main b86d94d1, including statistics and transaction settlement. The proposal appends ABI 12.8/slot 212 after settlement slot 211; the host table is 1,728 bytes and the metadata record remains 52 bytes. Exact 29-file source/commit audits, fresh apply/tree proof and hooks pass. The upstream check passes 4,268 model tests with nine skips, lint, formatting, interfaces, affected doctests and fresh local-target documentation links. Broad trybuild and stable-surface checks remain with CI; 61 external book links and native provider/value behavior remain unverified. No provider was installed. Product SDK 6a151a95/ABI 12.1 and installed 0.42.0/0.20.1 remain unchanged.

Keep historical exact-base patches for reproduction. Future upstream append conflicts require the same preservation and fresh source qualification. A supported matching provider and native value acceptance remain separate gates before consumer adoption.
