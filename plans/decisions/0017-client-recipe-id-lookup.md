# 0017: Resolve client recipe observations by archive ID

Date: 2026-11-15. Status: source verified; not deployed.

The companion already observes published station recipe-selection events and returns raw archive recipe IDs. Its recipe reader only accepts an output item name, leaving an observed recipe ID without a direct metadata lookup. Extend `client_recipes` to accept exactly one output `item` or integer `recipe_id` in 0..4294967295, retaining the existing optional 1..8 result limit. Zero remains an observed ID, not missing.

Resolve through the same bounded vanilla registry parser on the native frame callback. Count all matching records and preserve collisions and truncation rather than selecting an arbitrary record. Archive identity does not prove a current UI selection, crafting applicability, effective mod definitions or eligibility. No executor, event injection, new SDK patch or world access is introduced. Existing output-name requests preserve their complete response shape. ID lookup must explicitly state its selector and ambiguity.

Verify argument admission, zero/max IDs, unmatched IDs, duplicate IDs, complete raw record preservation and existing output matching. Then perform a cold matched companion replacement and compare an existing live output query against direct lookup of its observed record ID, including unchanged inputs/outputs/requirements/category rows. Rendered selection applicability remains a separate unsupported or unverified dependency.
