# 0029: Explicit recipe alternative selection

Date: 2026-11-15. Status: implemented; private/public source verified (305 Rust and 50 Python tests, Clippy, formatting and release builds pass); matched installed synthetic transport/read-only acceptance passes; connected production-chain acceptance remains unverified.

The production-planning goal includes alternatives. Extend recipe_plan with optional choices entries containing exactly item and recipe, rather than introducing automatic search or an objective function. Each item with multiple single-output producers requires one matching recipe ID. A single producer remains implicit, and existing calls need no choices field. Refuse duplicate choices, absent producers and IDs belonging to another item. Catalogue IDs remain globally unique.

Validate all supplied recipe fields and include all candidate outputs and ingredients in the existing 512-item bound before selecting. Only selected recipes enter dependency ordering, cycle detection, batch expansion and station/unlock requirements. An unselected cycle is irrelevant; malformed unselected data is still refused. The existing 128-recipe, 16-ingredient and arithmetic limits remain unchanged.

This is deterministic caller-directed selection, not alternative optimization, current game recipe validation, live stock eligibility or crafting execution. Categories, catalysts, coproducts and recycling remain unsupported. Verify selected batch costs and requirements, input-order invariance, selected versus unselected cycles, strict choices, unselected catalogue bounds and compatibility with single producers. Complete installed transport checks separately from source tests.

Matched 0.38.0 qualification passes after a complete cold archive and exact rollback copies. Live synthetic requests verify selection, rounding, shortages/requirements, input reordering and strict refusal, with all fixed read-only probes, complete captures, immutable ledgers, source closure and installed hashes preserved. A fresh native Menu observation after client relaunch does not establish joined gameplay or game recipe semantics.
