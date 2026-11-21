# 0196: Refuse entity health reads across world lifecycle changes

Date: 2026-11-20. Status: source0.51.7 behavior qualified; native interruption remains unverified.

The dynamic observation audit left entity_health unproven. An isolated published SDK FakeHost fixture now resolves typed NetworkHealth against a deliberately synthetic padded reflected layout, with full-size buffers and published field names. It independently retains spatial candidates, health37/83, a present zero0/0 component, an absent component and a world key across MCP lifecycle events. No game memory, offsets, binary inspection or runtime capture is used.

Initial fixture attempts incorrectly used short component buffers to expect a decode error, then absence. Neither expectation was established; those attempts do not count as a product reproduction or evidence of malformed-component behavior. The corrected fixture uses only valid full-size buffers. Its unchanged-world dispatcher case passes before any production change, preserving present/zero/absent distinctions, spatial data, total/truncation, scope, world key and limit behavior.

Three corrected lifecycle cases fail before the fix: queued reads return retained SDK health after WorldLoaded changes the MCP epoch or ShutdownStarted makes it inactive, and admission after shutdown occupies a scheduler slot. The fixture independently confirms positive SDK values after each event. This demonstrates a synthetic MCP scope defect, not observed native interruption timing.

Source0.51.7 reuses existing active-session admission and game-free epoch comparison before spatial discovery, typed reads or world-key access. Reject inactive admission before scheduling, then require the same active epoch on the game thread. Keep existing argument checks and typed-availability error ordering, raw health values, absence/decode-error branches, output limits and schema unchanged. No health mutation, generalized dispatcher abstraction or SDK change is added.

All four dispatcher cases pass after correction. The absent branch is exercised independently of zero health; the existing decoder-error branch is retained by the unchanged body but this fixture does not qualify its error path. Original baseline503 Rust/133 Python and required checks pass. Corrected507 Rust/133 Python, strict Clippy, formatting, both release builds and contracts must pass before publication. No new native install or launch qualifies this source increment. Default SDK12.1, companion0.22.0 and all54 contracts stay separate from the older SDK12.16 migration and installed signed runtime.

Next qualify a fresh signed-provider source0.51.7 server, then independently known positive entity health versus its rendered meter and controlled native interruption. entities_near and entity_components remain separate unproven audit candidates. Consumed client191 and fresh unconfigured sender helpers198 are independent work; no further client launch is authorized by this change.
