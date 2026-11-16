# 0034: Cross-check recipe workshop reference field contracts

Date: 2026-11-16. Status: implemented; private/public source verified (338 Rust executions, including 18 shared module checks, 54 Python tests, Clippy, formatting and release builds); matched native acceptance pending.

Observed workshop reference GUIDs are zero while their IDs resolve to nonzero definition GUIDs. Before attributing that to cooking, an unused field or a reader bug, compare the published SDK RecipeLayout with canonical reflection of the recipe registry. The SDK fact catalogue explicitly names RecipeInfo.workshopGuid and workshopId separately, and its retarget helper writes both. That source does not explain the observed zero references.

Resolve the canonical recipe array and record, GUID field and transparent u32 ID field (direct scalar or aliased value wrapper). The game-free ReferenceLayout policy requires equal array/record/field descriptions, correct GUID/u32 widths/kinds and admitted record spans. Refuse disagreement. This adds no guessed offsets, resource dump, World access or SDK patch.

Successful public recipe observations add workshop_reference_layout_checked true. Do not expose offsets or rewrite any raw value. Agreement between two published interface descriptions is not an independent measurement of game semantics, reference value correctness or effective definitions. Zero references remain zero and their cause remains unverified.

Test exact agreement, scalar/alias forms, offset/size/kind mismatches and false/missing probe markers. Matched native qualification must run both recipe selection routes, require the marker, compare all pre-upgrade observations after removing only that marker, and preserve complete runtime baselines/ledgers/binaries/gates. Even successful qualification cannot turn the zero-reference cause or crafting applicability into a verified claim.
