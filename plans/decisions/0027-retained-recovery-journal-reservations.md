# 0027: Retained recovery journal reservations

Date: 2026-11-15. Status: runtime source integrated and private/public source verified; matched deployment and live staged recovery pending.

Accepted construction phases remain undoable in their live native session. Their remaining audit recovery budget must survive later construction and unrelated appends to the same journal. Preserve the 8 MiB cap, bounded per-record sizes, existing write/session gates and fail-stop native semantics. Do not replace admission with a diagnostic or rotate/delete audit history.

First add a game-free checked current + incoming + retained budget rule. A reproduction must show the old new-cycle-only check accepts where the aggregate check refuses. Boundary, overflow and zero-retained compatibility tests prove the arithmetic foundation, not live protection.

Runtime integration serializes budget admission, durable append and reservation accounting under one shared per-mod-directory lock. Use bounded process-local reservation owners tied to construction operation identities; this state is not persisted as native identity because native undo expires on world exit/reload. All three direct append paths (construction, reposition, pacification) and GameTools::journal must share the admission path. Preserve their existing record schemas, record limits, durability and outcome headroom contracts. Source callers, not tool arguments, choose any recovery owner.

Before a new construction native request, reserve its bounded future undo audit bytes while preserving every prior reservation and forward/acceptance headroom. Normal appends cannot spend reservations. Only that operation's recovery audit may consume its own remaining allowance, after a successful synced append. An uncertain append/reservation failure remains conservative and cannot enable retry or release another operation's protection. Retain unknown outcomes. Release unused reservation only after exact baseline-verified undo or a world lifecycle transition that invalidates native transactions. Acceptance alone never releases recovery budget.

Verify ownership isolation, later construction and unrelated writer refusal, exact capacity boundary, no state consumption after failed append, once-only recovery behavior, partial audit failure, world reset and existing zero-reservation compatibility. Complete the shared-writer review before claiming the original admission gap fixed. A tiny accepted overlapping-phase live fixture and cold recovery remain separate acceptance gates.
