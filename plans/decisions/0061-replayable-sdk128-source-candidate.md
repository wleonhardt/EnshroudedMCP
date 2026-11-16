# 0061: Preserve a replayable whole-product SDK source candidate

Date: 2026-11-16. Status: source qualified; native provider and adoption unverified.

The optional consumer model is insufficient to establish whole-product compatibility. Preserve an inactive migration patch against sanitized public commit c65a42643660260c4f166174f290a4beda5da00b. It pins official SDK 9f6efc821524df426991fb584a761eaa0e5ea898 / ABI 12.8 and Rust 1.100.0 without changing the working product dependencies or installed provider. The four product members and package versions remain unchanged.

The 11-file patch updates dependency/toolchain pins and compiled provenance, adjusts probe expectations, and adapts fake-frame tests to SDK snapshot warm-up. Five initial client tests failed because native sampling had not started. Fixtures now request a snapshot and draw a frame before testing fresh context. The negative fixture additionally proves a valid context first and asserts the intended refusal reason. No production admission, timeout, write, session or recovery gate is weakened.

Before migration, a fresh original-SDK copy with external build output exposed five health-fixture parent-directory failures. The independently published c65a426 fix creates and canonicalizes that test parent and bounds cleanup to its direct child. The corrected fresh original-SDK baseline passes all 370 Rust executions. That prerequisite fix is already in the patch base, not hidden inside the SDK migration.

The exact candidate and a second clean source tree reconstructed by applying the patch each pass 370 Rust executions, 71 Python helper tests, strict Clippy, formatting and both release builds. Checks use Rust/Cargo 1.100.0, two build jobs and one Rust test thread. The isolated patch SHA-256 is ba99ca173ff07a0211fc172170e106f60557d3f1525f084c512780cf576c2b93. The reconstructed Git tree is af4a33a131b95e89bf43c44ea8e57a51b71d8e36; normalized candidate source bytes match it.

No candidate DLL or native provider is installed, executed or live qualified. Source compatibility does not establish native ABI/value/thread fidelity, rendered behavior, persistence or rollback acceptance. The existing signed provider and release-provenance policy remain intact. Review a concrete bounded native-provider experiment separately; adopt only after matched deployment, read-only correspondence, existing regressions and rollback acceptance pass.
