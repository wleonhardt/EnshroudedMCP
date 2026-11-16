# Isolated native placement-metadata qualification

Status: staged source/artifact procedure, 2026-11-16. No native execution or provider adoption is claimed. Product installation and SDK pin remain unchanged.

## Exact inputs

- Provider source: official 9f6efc821524df426991fb584a761eaa0e5ea898, ABI 12.8. Official development run 43919080948, artifact 14252493516.
- Outer artifact SHA-256: cacf7345d1ebcd29679182708727e6c0a940c1cc446d68ce7e2ead52817d7fe6. Nested package SHA-256: 1c5315d3f0fe0067445663910e3acc3f0827f764743c1ae927f4bf1ff7bcc299.
- Probe: this crate's locked Rust 1.100.0 release build, eight debug/release source contracts. Record source closure and DLL hash for the exact run.
- Optional administration consumer: inactive whole-product candidate built from public c65a426 plus consumer-candidate.patch, SHA-256 ba99ca173ff07a0211fc172170e106f60557d3f1525f084c512780cf576c2b93. This is the separately qualified 370-execution historical candidate, not a claim that it contains later product fixtures.

The provider package lacks release signatures. Upstream's [official-build classifier](https://github.com/enshwick/enshwick/blob/9f6efc821524df426991fb584a761eaa0e5ea898/crates/runtime/loader/src/base/official.rs) labels unsigned builds unofficial. Preserve that classification and the project's release-provenance/adoption gate. Never disable its checker or substitute signatures from another build. A successful research experiment does not authorize replacing the signed working provider.

## Isolation preconditions

1. Use a new disposable server tree. Preserve the existing signed server/client installation, save paths, journals and client state. Capture exact installed and immutable-ledger membership/hashes before starting. Do not overlay working DLLs.
2. Copy only explicitly identified immutable dedicated-server runtime assets; exclude existing configuration, saves, account/cache data and credentials. Record copied asset hashes. Never touch the Steam DRM stub or disassemble game files.
3. Verify the current published dedicated-server configuration schema before generating a fresh configuration. Use loopback binding, unused game/query/MCP ports, new save/log/mod paths and an empty character roster. Do not read or copy a credential-bearing configuration. Configuration and launch are not yet verified by this procedure.
4. Stage the exact provider, probe and optional administration consumer with a manifest. Keep write flags absent. No client join, avatar movement, geometry, grant, clock or other gameplay fixture is needed.
5. Capture an independent template mapping for the known selector through the existing signed provider's read-only item_info API and record relevant build/asset identity. This is a mapping reference, not physical geometry.

## Bounded experiment

Start only the new server, retaining exact process identity. Confirm expected development-provider classification and the candidate's compiled SDK/ABI profile. The contract checker can compare server discovery against an explicitly supplied experimental profile; do not rewrite the active product matrix to accept a mismatch. Compiled metadata is not provider fidelity proof.

Wait for one complete exclusive placement-observation.json. Preserve partial/error output; do not delete it to trigger an automatic retry. Validate exactly two results, copied item/template identities, nullable raw corners and the invalid-selector error. Compare template mapping with the independent reference and optional item-info result. Missing independent reads stay unknown. Correlation does not prove footprint, collision, permissions, labels, persistence or recovery.

Use the candidate's own read-only probes for baseline transport/API regression where applicable, retaining their incomplete gameplay verdicts. The game client and primary server remain outside this experiment.

## Shutdown and rollback

If the optional candidate administration endpoint is healthy, preview one deliberate server_shutdown request with an empty SDK character roster. Create a fresh write flag only in the experimental administration mod folder, submit once with the exact session and confirmation, and remove the flag afterward. Preserve its intent/outcome ledger and never retry an uncertain stop. Independently observe the pinned process exit; process exit is not causal final-save proof.

If native shutdown is unavailable, record the lifecycle capability gap. Any necessary OS cleanup must target only the newly created, identity-verified experimental process; it is not native MCP execution. Do not stop another process or restore files into the working installation. Keep the disposable tree quarantined for inspection rather than deleting uncertain evidence.

Verify that working installation hashes, immutable ledger membership/bytes, original settings and absent flags remain unchanged. A cold backup/restore or rollback claim requires its own exact evidence. All native outcomes, failure/recovery observations and limitations belong in the private report; publish only sanitized conclusions. Product adoption remains a later, separate decision.
