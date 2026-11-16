# 0063: Require the intended client context refusal

Date: 2026-11-16. Status: source test defect reproduced and corrected; runtime gates unchanged.

While reviewing pending-cancellation coverage, inspect existing fixtures before adding another overlapping test. The invalid-context regression used a real request with the deliberately invalid preview old-context and asserted only that some error occurred. That allowed the preview gate to hide loss of the initial context-admission gate.

Reproduce against an isolated product source copy of sanitized public commit 6a8f18ac513f98e93f9d3f52361f7322ce516f5b. Remove both gate::applicable calls only in that disposable copy. The original unusable_native_contexts_refuse_before_reservation_or_dispatch test still passes. No production context guard is removed or changed in the working or published source. This is a false-positive test defect, not evidence that the shipped runtime lacks its context checks.

Tighten the production test fixture. Start copied-snapshot sampling and draw a fake frame, inject a valid fresh context, assert freshness, and prove a dry command succeeds before invalid cases. For seven unusable snapshots across all six command families, require the precise initial context-admission error instead of any error. Continue checking no native commands, reservation directory, journal or retained operation. The intentionally invalid preview remains useful because the assertion now distinguishes its refusal from the context gate's refusal.

The corrected test passes against unchanged production guards. Copying only the corrected test module into the isolated mutation makes it fail: actual error is client preview changed; expected error is client command requires a fresh nonzero session and readable HUD screen. This proves the improved test detects the missing initial context refusal. It does not independently mutation-qualify the later post-reservation context recheck.

Private/public complete product checks pass 372 Rust executions, 71 Python helper tests, strict Clippy, formatting and both release builds. Test count is unchanged because an existing test is repaired. Test-only setup and assertions change; package versions, SDK pins, admission code, deadlines, installed files and game state remain unchanged. No native action is required to qualify this source-test correction. Actual pending cancellation, callback stalls, disconnect, post-reservation context races and append/sync faults retain separate open acceptance.
