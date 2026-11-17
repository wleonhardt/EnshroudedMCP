# Isolated native lifecycle-provider qualification

Status: bounded empty-server experiment passed on 2026-11-16. No product adoption or actual Join/Leave acceptance. See decision 0078.

## Exact inputs and provenance

- Official source: 6a5966ee6683fde480ac9dd3654f91e73da6e2a5, ABI 12.11.
- Official [development run 43958857151](https://github.com/enshwick/enshwick/actions/runs/43958857151), artifact 14274795299. The completed main-push run's head matches the exact revision.
- Outer artifact SHA-256: 5393020766443e02827b536ef4e586d022f09d44ca1cf4c7801169d0004c7642, matching GitHub's artifact digest; 24,526,885 bytes.
- Nested package SHA-256: 986073c66fe180826dc35b5b3438ff3a6115377ca17a723c2b0068a8fd02a07b. The nested package is covered by the verified outer archive; no separate SHA256SUMS file is supplied.
- The provider package contains the loader stub, provider DLL and license, with no detached release signatures. Preserve its [unofficial classifier](https://github.com/enshwick/enshwick/blob/6a5966ee6683fde480ac9dd3654f91e73da6e2a5/crates/runtime/loader/src/base/official.rs); never substitute signatures or disable the checker.
- Diagnostic probe: this exact source, locked Rust/Cargo 1.100.0 release build. Administration consumer: the separately qualified whole-product 0.46.1 migration from decision 0077, with the same SDK revision. Record both DLL hashes and source closures for each run.

## Preconditions and bounded procedure

1. Preserve working server/client hashes, durable namespace membership/bytes, original setting and absent write flags. Keep the client closed. Never overlay the signed working provider.
2. Create a fresh disposable server tree with sufficient headroom. Copy and independently hash only the twenty-three explicitly identified immutable dedicated-server runtime assets, excluding existing configuration, saves, caches and credentials. Never alter the Steam DRM stub or inspect game implementation.
3. Recheck Keen's [server configuration](https://enshrouded.zendesk.com/hc/en-us/articles/16055441447709-Dedicated-Server-Configuration) and [role configuration](https://enshrouded.zendesk.com/hc/en-us/articles/19191581489309-Server-Roles-Configuration). Generate new save/log paths, loopback binding, unused query/MCP ports, one slot and an opaque restricted access role. Write the new credential-bearing configuration once; never reopen, hash, archive or disclose it. Do not read process streams or game logs.
4. Stage the exact provider, probe and administration consumer in their own fixed mod namespaces. Record the complete file manifest, preserve disabled flags, and launch once while retaining exact PID/creation/executable identity.
5. Compare all four loaded DLL paths/hashes with the staged set. A Windows system library sharing the stub's basename is not the staged stub. Verify owned loopback bindings. Read only the probe's own bounded metadata report and SDK per-source classification log.
6. Require the reported provider header and compiled requirements to agree for this exact pair. Preserve SDK/source versus actual provider fields separately. Execute the candidate's fixed read/refusal suite and exact contract comparison. Compare one known item/GUID/template mapping with the signed provider; no physical geometry claim follows.
7. Check tiny top/X-Y/Z-Y images against independent complete native surveys, raw cells, coordinates, caption glyphs, reference pins and timestamps. A first unloaded-region error remains an error; it cannot become fabricated Air. Later chunk availability permits a new bounded observation, not a mutation retry.
8. With the experimental roster empty, preview and submit one deliberate native shutdown under its own fresh flag. Hold the exact-process observer before dispatch; remove the flag afterward. An unknown outcome never authorizes a repeated shutdown. Verify the journal prefix, four stop rows and one marker. Retain a successful held-handle exit receipt; a terminal PID may no longer be reopenable. Verify current executable absence without touching another process.
9. Recheck all copied assets, staged modules, unchanged header bytes, working installation/namespace hashes and original settings/flags. Keep the disposable tree quarantined. Process exit and fresh save output do not prove causal final-save completion or cold restore.

## Recorded bounded outcome

The provider reports ABI 12.11 and a 1,744-byte table, matching the compiled table size. All four loaded file paths/hashes and both loopback bindings agree. All forty fixed read probes pass while their broad gameplay verdict stays incomplete. Known item/GUID/template correspondence agrees with the signed provider. Three tiny images pass 2,205 independent interior pixel comparisons, visible-axis glyphs, raw-cell/pin/timestamp checks and equal bracketing surveys.

One native stop is accepted and independently exits 0, with exactly four rows and one identity. All twenty-three copied asset hashes, staged modules and header-report bytes remain exact. The primary twelve hashes, all primary durable namespaces, original pacification and absent flags remain unchanged; the client stays closed. Native SDK per-source logging preserves the unofficial classification. The SDK's empty startup crash-note file is not treated as a crash merely because it exists.

This is bounded server/header/read evidence only. It does not certify general provider fidelity, native client callbacks, real Join/Leave/cancellation, credential resolution, physical geometry, save causality, cold recovery or release provenance. The unsigned provider is not adopted; the working SDK remains ABI 12.1.
