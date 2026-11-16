# 0039: Qualify the newer settlement API before changing runtime pins

Date: 2026-11-16. Status: isolated source compatibility checks pass; working source and runtime pins are unchanged. Settlement-helper behavior and native loader compatibility remain unverified.

Official SDK main revision 70a468bd4d0001f5e7237bf09ff3b3b0ffc04fc2 adds World::when_settled and starts snapshot sampling when Input opens. The command enum is unchanged from the qualified pin; it supplies no new join, credential, logout, category-change or crafting action. The checked release list still starts with alpha.17. New source availability does not justify changing a qualified runtime automatically.

An isolated candidate using Rust 1.100, updated exact provenance and candidate versions passes 343 Rust executions (including 19 repeated shared-module checks), 58 Python tests, Clippy, formatting and both release builds. A separate source signature probe compiles the new callback API and confirms numeric ABI 12.1. Equal ABI numbers and source compatibility are not native provider or binary-compatibility proof. Initial failures in two client fixtures were corrected by driving the published fake draw after snapshot requests; runtime freshness/session guards were preserved. The qualified working pin remains 6a151a9550ed7a6ade99c23fa817a91e25b71bc9 with Rust 1.99.1 and installed server 0.40.0 / companion 0.18.0.

Keep the new helper outside the product until its own pending/applied/expired/failure/bound/world-exit behavior and matching loader provenance are qualified. It can return standing props after undo data expires, and its post-write scheduler pass can occur beyond the requested bound. Neither standing nor a passed bound grants undo authority, proves a postimage, cancels work or permits retry. Preserve durable once-only identity, session ownership and independently checked recovery conditions.

The helper does not supply an exact placement footprint, exact label ownership, atomic held-state recovery or native client crafting. Those remain separate requirements. Preserve cold rollback, complete read-back, immutable ledgers, write gates and separate native/rendered outcomes for any later matched migration. No runtime upgrade, loader replacement, game write or SDK patch occurred during this source qualification.

Primary source: [official SDK change](https://github.com/enshwick/enshwick/pull/1130) and [immutable transaction API](https://github.com/enshwick/enshwick/blob/70a468bd4d0001f5e7237bf09ff3b3b0ffc04fc2/crates/internal/world/src/world/transactions.rs).
