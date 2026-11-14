# Changelog

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
- Administrative construction now has a narrow reversible cell proof and bounded plan implementation. Multi-cell live acceptance, reusable modules, applied persistence and the advanced workshop benchmark remain open.

Detailed private deployment history is deliberately excluded. Public semantic evidence and limitations are recorded in `plans/research/live-validation.md`.
