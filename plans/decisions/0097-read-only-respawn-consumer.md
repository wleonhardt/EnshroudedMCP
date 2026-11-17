# 0097: Qualify a read-only respawn-target consumer separately

Date: 2026-11-17. Status: source qualification; no native acceptance or adoption.

Official releases still end at alpha.17 revision 6a151a95. Main revision daffae3937398f29939783adc5247d6e5bb75bbf is ABI 12.13 and requires Rust 1.100.0. Its new published respawn-point getter provides a useful future recovery observation. Newer main also includes breaking action-constructor and content-GUID admission changes; it is not a drop-in signed-provider upgrade. Preserve the installed SDK, product dependencies and decision 0093's exact SDK 12.11 review closure.

Add an independent rlib-only workspace under qualification/sdk-respawn. Use the published safe API and FakeHost only. One getter projects a future respawn target into an explicit read-only observation. Missing data is not a zero-origin position; unknown kinds are not fabricated raw codes. Preserve typed native errors and finite raw coordinates, reject malformed copied values, and make missing session identity explicit. No setter, new product tool, gameplay command or installed file changes.

Seven cases cover exact read-only observations, unresolved and departed players, kind labels and legitimate zero origin, wrong-thread/unavailable errors, reading without entity commands, and malformed copied observations. The first compilation exposes an unsupported serde_json array macro expression; replace that fixture with an explicit array and use the SDK's actual f32 quaternion type. Corrected debug tests and strict lint pass. Release and independent publication checks are recorded before publication.

The caller must still supply a current server Player and World on the game thread. Session/selector admission, provider fidelity, actual native target values, persistence and independent observation are unqualified. A future product route must use the existing Scheduler and policy conventions. This observation cannot prove arrival, contact, current health, exact return, revival or permission to move. No automatic movement follows from it.

Goals 2 and 5 gain a concrete next increment: qualify this read against a matching provider and independently observed current-player target before considering product integration. The older ABI 12.11 Menu review cannot satisfy that gate. Direct join remains an upstream capability request; maintainer discussion on #1144 describes planned favorite/address/password joining, not a delivered saved-profile or password resolver. No new maintainer contact occurs.

The active product baseline remains 401 Rust executions and 97 Python tests. Raw build receipts remain private. Publication contains only the independent source, lockfile and neutral evidence. Browser visual acceptance and the separate temporary signed-provider exception remain pending.

Private and independent public qualifier debug/release suites each pass all seven cases, strict lint and formatting. Private/public product checks each pass 401 Rust executions, 97 Python tests, strict lint/format, both release builds and the contract index. Live server discovery remains consistent with ABI 12.1/table 1680. The current source, candidate/rollback closure and signed runtime preflight remain exact; no game launch or installed change occurs.
