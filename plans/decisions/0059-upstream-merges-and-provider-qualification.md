# 0059: Keep upstream merges separate from trusted provider qualification

Date: 2026-11-16. Status: both upstream PRs merged; matching development provider unqualified.

GitHub confirms both PRs merged at 2026-11-16T17:30:23Z:

| PR | Final source head | Merge commit |
|---|---|---|
| [#1141: read-only prop placement metadata](https://github.com/enshwick/enshwick/pull/1141) | 3c956e5131e340c4c4d4a55743537c721fedf1fd | 3a3e52f71cdf9a02ffc855cf9fc921e8ccbd5b0c |
| [#1111: cached sampler diagnostic boundaries](https://github.com/enshwick/enshwick/pull/1111) | 3c2f3b7ef7a462b26bd9ef5b1bc5845f9e795423 | 9f6efc821524df426991fb584a761eaa0e5ea898 |

Published source at 9f6efc82 declares ABI 12.8. Its safe World::prop_placement_info returns the queried item, template GUID and an optional declared local box. None means absent, invalid or unavailable, never an empty footprint. This is metadata, not complete physical geometry, collision, permission, persistence, ownership or safe-recovery proof. The reviewed CommandAction enum still has only screen/tab/look/move/recipe-highlight families; it adds no join or logout executor.

The successful [official dev-build run 43919080948](https://github.com/enshwick/enshwick/actions/runs/43919080948) provides artifact 14252493516 for 9f6efc82. The downloaded outer ZIP SHA-256 is cacf7345d1ebcd29679182708727e6c0a940c1cc446d68ce7e2ead52817d7fe6, matching GitHub metadata. Its nested package SHA-256 is 1c5315d3f0fe0067445663910e3acc3f0827f764743c1ae927f4bf1ff7bcc299. The package contains the loader and framework DLLs but no detached signatures. No DLL was installed, executed or queried for native ABI. No signature bypass is introduced. Source ABI declarations do not prove native provider compatibility.

Latest inspected main 4ce03662 declares ABI 12.9 and adds unstable container/rule/transfer/pickup APIs. It is a separate moving source checkpoint. The latest published release observed through the official release list remains [alpha.17](https://github.com/enshwick/enshwick/releases/tag/v1.0.0-alpha.17), exact 6a151a95. Product SDK, ABI 12.1 and Rust pin remain unchanged. Matched product runtime 0.45.0/0.20.2 is installed separately; neither product version implies a framework upgrade.

Next goals:

1. Qualify a matching provider through the established distribution/signature checks before native adoption. Keep the current signed provider until that gate passes. Do not chase moving main or infer trust from a successful CI build alone.
2. Prepare a source-only consumer qualification at an exact merged revision using the existing isolated qualification workflow. Verify required toolchain and source contracts before changing product pins. Preserve four root workspace members.
3. Add a bounded game-thread item/template/nullable-declared-box observer only once its supported dependency is qualified. Preserve raw identity namespaces and unknown bounds. Do not convert declared boxes into terrain contact, complete collision envelopes or build authorization.
4. Qualify native item/template correspondence and unavailable/refused cases before using metadata in construction planning. Keep exact read-back, journaled once-only writes and recovery gates for any later placement fixture.

Historical source-check counts for earlier proposal heads do not certify these final merged heads. The merge and successful artifact build are authoritative upstream state; downstream compatibility, native values and visible geometry still require their own evidence.
