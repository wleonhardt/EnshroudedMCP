# Changelog

## 0.16.0

- Add nullable native health, stamina and mana to existing player rows. Preserve raw SDK integer values, observed zero and independent unavailable pools; UI unit conversion remains unverified.
- Add neutral original branding, setup, tool reference, architecture, research boundaries and contribution documentation.
- Pass 146 Rust tests, 30 Python tests, Clippy, formatting and both release builds in source and sanitized publication. Earlier isolated meter reads passed; the final write-gate correction below is source-tested and not live-installed.
- Fix `give_item` write-flag admission to use the common metadata validator: directory flags and future timestamps are refused. A regression first reproduced directory admission, then passed with the fix. This changes write admission only; no live give was performed for this fix.

## 0.15.0

- Add dry-by-default `player_reposition`, one bounded server administrative teleport request for a sole connected living actor. Bind preview to exact current server actor/session/generation/position and target; require fresh write gates again before dispatch. The 16 m gate limits the requested point, not the engine's best-spawn correction.
- Synchronize bounded durable identity/intent and journal before a retained once marker and one native request attempt. Canonical same-ID replay does not redispatch; altered requests and stale durable identities refuse. Submission stays `issued_unknown`, not an arrival receipt. Returning needs another separately gated request; no automatic retry, chain, native undo, walking, camera or menu executor.
- Pass 143 Rust tests, 30 Python tests, Clippy, formatting and release build. Correct an outdated tool-advertisement fixture; the read-only validator advertises the new write without ever invoking it. Independent scoped security review found no blocking issue, without asserting complete safety.
- Validate one isolated two-command fixture through six explicit phases, independent rendered/position observations, replay with unchanged journals, exact 4,096-cell construction preservation and cold-reload stale refusal after a verified 40-file archive. Observed positions differed from requested points. A zero-player old-preview/new-identity refusal establishes actor refusal only. Exact arrival, precise return, arrival causality and complete player-state recovery remain unverified.

## 0.14.0 and companion 0.3.0

- Pin official development revision `5f647f46d7f987e93d397e00596014b23f3c1112`, ABI 11.30, in a dedicated sibling checkout. Retire the private client-event ABI from the active build and retain only a three-file existing-field SDK metadata extension with no native ABI change.
- Update two registry resource lookups to the current typed API. Report compiled framework revision/ABI separately from live acceptance and add nullable native screen-host template observations. Fake-host tests cover absent, present and cleared templates without inventing a screen-change event.
- Pass 133 Rust tests, 30 Python tests, Clippy, formatting and both release builds. The SDK's prescribed nextest run passed 538 tests. Matched isolated installation followed a verified 38-file cold archive and rollback hashes for all eight replaced binaries.
- Pass eighteen server and nine companion read probes with broader acceptance incomplete. Preserve the exact 4,096-cell shell baseline and stale old operation. Fresh Menu/InGame observations passed; null held input, a non-null live host template and named event delivery remain unverified. The interaction fixture was stopped after an enemy attack; logout completed and writes stayed disabled.
- Keep input/control APIs observational. No vanilla movement, camera or menu actuator, current tab, passive recipe selection, safe prop lifecycle or functional workshop completion is claimed.

## 0.13.0

- Extend read-only `item_info` with distinct observed placed-template, known-no-template and unavailable states. Preserve existing metadata and return explicit null metadata when the runtime definition is unavailable. Template GUIDs use lowercase file-byte-order hexadecimal encoding.
- Expose an existing native ABI field through the pinned compatibility SDK; no ABI, loader or companion change. The combined 23-file patch reproduces against the exact pinned base while retaining the original 20 event-extension files unchanged.
- Pass 132 Rust tests, 28 Python tests, Clippy, formatting and release build. The isolated research-ID deployment passed all 18 settled read-only probes with broader acceptance incomplete. The immediate startup run passed 17 and failed one construction-readiness check; listener availability alone is not readiness.
- Keep functional prop identity, full placement bounds, state-aware recovery and workshop usability unverified. A nonzero template mapping does not authorize spawning.

## 0.12.0

- Extend `building_apply_plan` with up to eight absolute-position primitive parts. Bound the global admitted footprint to 16 m per axis and the candidate sum to 1024 before cell expansion. Union/deduplication assigns overlaps to the first lexical part ID and preserves stable canonical requests under part reordering. Keep the existing 256-cell, eight-brick and 64-cell-per-brick limits; pure `building_plan` is unchanged.
- Validate one six-cell compound union from eight raw part cells and two overlaps. Sixteen isolated proof checks covered independent whole-capture verification, rendered appearance, reordered replay without another transaction, renamed-part refusal and exact undo. Post-undo cold reload matched all 512 baseline cells; stale accept/undo, old-preview and same-identity replay were refused without changing ledger hashes.
- Pass 131 Rust tests, 27 Python tests, Clippy, formatting and release build. All 18 fixed read-only probes passed with broader acceptance still incomplete. Applied-compound persistence, larger modules and workshop functionality remain unverified.

## 0.11.0

- Add gated, dry-by-default `building_accept`. Require a current Applied transaction, exact captures, the sole original actor, rights, persistence and a healthy journal. Sync the acceptance record before releasing placement exclusivity; retain the typed session-only undo receipt. Duplicate acceptance writes no additional record.
- Require strict expected postimages for accepted undo, including one-cell receipts, and block undo while another operation is unsettled. Queued undo revokes acceptance/settlement until exact baseline verification.
- Pass 125 Rust tests, 27 Python tests, Clippy, formatting and release build. The isolated research-ID installation passed all 18 settled read-only probes; one isolated acceptance/layering fixture and its post-undo cold reload passed.
- Validate a five-cell frame plus one-cell gap filler in six phases, with exact 512-cell capture comparisons throughout. Acceptance/replay guards, wrong-order undo refusal without losing the receipt, reverse-order undo and rendered gap closure/reopening/disappearance passed.
- Verify post-undo cold reload against all 512 baseline cells. Both native receipts stayed stale, old session replay was refused, no new reservation appeared, ledger hashes stayed unchanged and the session flag was absent. This is not persistence-while-applied or stateful recovery evidence.
- Record narrow rendered 0.10 observations without claiming incremental gap fill, collision/traversal safety, stateful recovery or workshop completion.

## 0.10.0

- Expand bounded administrative plans to 256 cells, eight aligned bricks and 64 cells per brick. Preserve exact per-brick captures, all-brick verification, guarded undo and existing session/replay/journal rules. One native receipt does not imply atomicity or resumable jobs.
- A separate 0.9.1 floor experiment verified persistence while applied: all 512 captured cells matched after cold reload. Preserve stale native IDs and the applied fixture.
- Fix parallel test temporary-directory collisions with a per-process nonce.
- Pass 122 Rust tests, 25 Python tests, Clippy, formatting and release build. The isolated installation passed 18 settled read-only probes; the 32-cell two-brick proof passed placement/read-back/replay/undo with exact captures; its final post-undo cold reload passed with all 1024 baseline cells equal.

## 0.9.1

- Add `building_apply_plan` for floor, wall and doorway geometry bounded to 64 requested cells in one aligned voxel brick. Preserve logical element keys, exact source material/fill, one native transaction and full-brick verification.
- Require an exact expected whole-voxel postimage before multi-cell undo. This mitigates a framework application-window foreign-edit gap without claiming concurrent safety, compare-and-swap or atomicity.
- Retain the single-cell recovery path, unchanged-props/persistence checks and native undo conflict guard. The 16-cell floor live proof and post-undo cold reload passed.

## 0.8.0

- Add gated `building_place_cell`, `building_operation` and `building_undo_cell` with session-bound dry previews, durable intent, replay tombstones, typed native receipts, exact capture verification and idempotent undo requests.
- Validate one isolated cell placement/read-back/undo and cold reload of all 512 baseline voxels. This proves the reverted fixture, not applied-structure persistence or stateful object recovery.

## 0.7.0

- Add read-only `building_survey`, bounded to 8 m per side and 4096 complete cells, with exact runtime material/fill, whole-box persistence and optional connected-player build rights. Gate arguments, allocation, full-cell bounds and selectors before scheduling onto the game thread. Unloaded and unavailable states remain explicit.
- Keep construction execution, material mapping, collision, survival accounting and recovery acceptance open. Surveying does not authorize writes.

## Research snapshot

- Server 0.6.0 provides fifteen tools, including bounded inspection, curated evidence, pure recipe/geometry previews and gated `give_item`.
- Optional companion 0.2.0 provides six native observation tools and bounded named client-event history through a separate local framework extension.
- Transport remains loopback-only. Write confirmation, fresh flag admission, game-thread scheduling and journaling remain required.
- This publication uses neutral `enshrouded.mcp` and `enshrouded.mcp-client` mod IDs and corresponding install folders. Earlier private installations must not be left active as duplicate mods.
- Administrative construction now has a narrow reversible cell proof and bounded plan implementation. Larger bounded plans, reusable modules, stateful recovery and the advanced workshop benchmark remain open.

Detailed private deployment history is deliberately excluded. Public semantic evidence and limitations are recorded in `plans/research/live-validation.md`.
