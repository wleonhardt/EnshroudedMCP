# 0092: Replay SDK 12.11 against the final-admission consumer

Date: 2026-11-17. Status: candidate and clean replay source-qualified; native execution and adoption remain open. No installed change or native candidate execution.

Preserve the historical migration artifacts from decisions 0077 and 0085. The new fourteen-file qualification/sdk-lifecycle/consumer-admission-candidate.patch targets exact public base 8a9de10f4971903e878c721224bdf58f878bbee2, server 0.47.0 / companion 0.21.1. It pins official SDK 6a5966ee6683fde480ac9dd3654f91e73da6e2a5, ABI 12.11 and Rust/Cargo 1.100.0. Product versions, tool schemas and inventory remain unchanged. The original signed-SDK baseline passes 399 Rust executions before migration.

The older patch needs three current-version/profile ports. Retain its four command snapshot warm-ups and one snapshot fixture warm-up. The newer custom copied-snapshot fixture also needs a truthful bootstrap response: this SDK reads the getter during initialization before its request script is armed. The first candidate run panics on an absent script and aborts across its foreign-call boundary. This is a synthetic test-process failure, not a native game crash or evidence of memory corruption. Preserve that failed log.

Return Raw::EMPTY only while the test script is absent. After initialization, explicitly assert Never, session zero and unknown age before arming the six request cases. Preserve valid dispatch, stale/session/HUD refusals, delayed getter expiry, revoked write permission and replay/reopen bytes. The entire production control prefix is identical to the base after newline normalization, including decision 0090's final permission and deadline checks. Do not extend waits, fabricate fresh context or weaken SDK delivery expectations.

The canonical patch has SHA-256 977cfd382b8d495545c2a377f8a111e9e63426265a860bf3fc61a8aec47be3b1. Independently apply it with whitespace errors refused in a clean checkout of the exact base. All 259 normalized tracked files agree and both staged indices reconstruct tree 5f304b27847ce718628cd22267aad00b40feecc5. Only ten blank unified-context prefix lines are normalized in the exported artifact; source whitespace checks remain enabled. Candidate and replay use separate targets, two build jobs and one Rust test thread, with no overlapping workspace builds.

Keep the signed 0.47.0/0.21.1 installation and SDK qualification checkout unchanged. Decision 0086's old runtime review is stale after the normal signed companion update. Neither source qualification nor a delayed approval for that obsolete closure authorizes an unsigned provider swap. A fresh coherent provider/consumer installation, rollback and concrete review must precede any native test. Menu reads cannot prove final-getter fault behavior, selected-character/peer identity, rendered effects or Join/Leave and credential acceptance.

Private source, logs, failed run and proof are under gamedata/sdk-current093; raw evidence and build outputs are excluded from publication. Next prepare the current candidate's matching native qualification closure, preserving decision 0059's signed-provider rule and explicit exception requirement.


Candidate and clean replay each pass 399 Rust executions, 96 Python tests, strict Clippy, formatting, both release builds and offline contracts with Rust/Cargo 1.100.0. Current signed twelve-file/durable baseline, exact server identity, empty roster, original setting, absent flags, closed client and original SDK checkout are preserved. Historical migration and timing patch hashes remain exact.
