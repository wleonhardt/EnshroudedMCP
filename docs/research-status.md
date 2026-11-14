# Evidence and remaining work

This is a research prototype with bounded administrative capabilities. Evidence is separated into source tests, fixed transport probes and controlled game fixtures. It does not establish general automation or a functional workshop.

## Reproducible source checks

The server 0.16.0 / companion 0.3.0 checkpoint passed 146 Rust tests, 30 Python helper tests, Clippy with warnings denied, formatting and release builds. The exact framework revision and SDK-only metadata patch passed 538 tests under the framework's prescribed nextest harness. These counts identify a development checkpoint, not an ongoing CI badge or a guarantee for another game build.

## Controlled observations

| Area | Evidence | Boundary |
| --- | --- | --- |
| Reads and transport | Eighteen fixed server probes and nine companion probes passed on the isolated installation. | Reports deliberately retain incomplete fixture/gameplay acceptance. |
| Inventory and health | Controlled stack changes, one durability comparison and one enemy health decrease were observed. Native player vitals distinguish unavailable data from zero. | Broad item semantics and UI-unit conversion are unverified. Positive health alone does not establish safety or quiescence. |
| Construction | One-cell, multi-cell, multi-brick and staged frame/filler trials verified exact images, deliberate replay and guarded reverse undo. | No general concurrent-edit safety, cost accounting or maximum-cap performance claim. |
| Composite shell | Six parts yielded 164 unique cells; all 3932 other cells in the 4096-cell region stayed unchanged. Rendered front/roof-edge/oblique views and disappearance after undo were observed. | Raised entrance, traversal, stairs, sloped roofs and functional fixtures were not accepted. |
| Cold recovery | Reverted geometry remained exact across controlled cold reloads; stale receipts/tokens were refused and ledgers remained unchanged. | A baseline reload does not prove an applied multi-part structure or player position persists. |
| Repositioning | Controlled once-command trials recorded one intent/outcome per identity, discarded-response reconciliation and unchanged replay journals. Subsequent server positions changed. | This is not direct live instrumentation of native invocation count, exact arrival/return, camera control or bounded engine correction. |
| Client companion | Fresh Menu/InGame observations agreed with visible screens. Named events have isolated historical evidence. | Current passive tab/recipe selection, migrated named-event delivery, a non-null host-template fixture and held-input semantics need further evidence. |

An isolated character died while idle during read-only observation. Normal client respawn and grave retrieval recovered the inventory; the final per-item totals matched the earlier snapshot. This is not evidence that a read caused death, nor evidence that a healthy-player check can prevent game hazards. No health restoration, inventory grant or write-gate bypass was used for recovery.

## Latest isolated checkpoint

The corrected server write gate was installed alone after a verified 40-file cold archive. Loader and companion hashes stayed unchanged. Directory and future-dated flag fixtures were refused using deliberately invalid player/item selectors; no inventory grant occurred. All 4096 preserved cells and complete pre-restart ledger bytes remained exact. An initial startup run passed 17 of 18 probes while construction was inactive; after observed readiness, all 18 passed, with broader acceptance still incomplete. These fixtures use the existing research installation; the neutral public mod namespace has not been independently live-installed.

The framework maintainer [reported a suspected held-input defect and planned diagnostic support](https://github.com/enshwick/enshwick/issues/1107#issuecomment-6883127694). Native look/movement/menu work is being investigated upstream. These are maintainer plans, not released capabilities. A local alternative diagnostic patch remains held rather than being installed or submitted as duplicate work.

## Next acceptance milestones

- Make a small shell accessible with independently reviewed support and entry geometry, then verify traversal visually and in the game.
- Establish supported prop identity, complete footprint and label-safe recovery before placing storage, doors or stations.
- Prove functional storage and one workbench recipe separately from geometric placement.
- Validate applied composed-structure persistence on a disposable fixture without overwriting unrelated state.
- Expand measured building modules and durable plan revisions before considering a general job executor.
- Obtain supported current-selection and semantic client-action APIs rather than treating input observations as actuators.

The long-term benchmark is a usable multi-storey workshop with stairs, roof, doors, storage and workstations. It remains open. Survival debit/refund, progression feasibility, arbitrary stateful rollback and unattended operation remain outside current acceptance.

Detailed sanitized checkpoints remain in [public evidence](../plans/research/live-validation.md). Private runtime identities, exact test locations, saves and screenshots are intentionally excluded.
