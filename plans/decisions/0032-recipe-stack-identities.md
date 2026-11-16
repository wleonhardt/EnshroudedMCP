# 0032: Preserve raw recipe stack and workshop identities

Date: 2026-11-16. Status: implemented; private/public source verified (331 Rust executions, including 18 shared module checks, 52 Python tests, Clippy, formatting and release builds); matched native correlation pending.

A native Menu recipe lookup resolves actual recipe records and typed ingredient IDs, but raw output stacks omit their item IDs. Recipe-ID lookup therefore cannot correlate output identities through the existing bounded item_lookup API without guessing or deriving an ID. The published RecipeLayout already supplies stack_item and workshop offsets, and existing game-free admission checks these fields against each complete record.

Read the raw u32 item field for every input/output stack and report item_id beside item_guid/count. Report workshop_id beside workshop_guid from its existing admitted field. Preserve zero and full u32 values exactly; never compute an ID from a GUID/name. Keep every output position, recipe collision, result truncation and existing workload limit. Raw input IDs remain alongside the richer typed input observations; a category-only zero item branch is not a free ingredient.

Only stack item IDs belong to item lookup. Workshop definition IDs and required prop type IDs are separate namespaces, not server entity or inventory selectors. No new executor, effective-definition assertion or category/catalyst/eligibility interpretation is introduced. The companion continues to read on its published native frame callback without World access.

Verify failing missing-field reproductions, synthetic unrelated GUIDs and zero/full-u32 identities, strict client probe validation, existing array/layout/truncation cases and both recipe selection routes. After matched deployment, compare output-name and recipe-ID observations, correlate nonzero IDs and GUIDs independently through the server registry, preserve category uncertainty and check complete runtime preservation. Connected UI applicability and eligible stock remain separate milestones.
