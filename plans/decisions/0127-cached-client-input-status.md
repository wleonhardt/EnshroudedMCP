# 0127: Add a cached native client input-status read

Date: 2026-11-19. Status: source implementation and SDK-fake qualification pass; not installed or native-qualified.

## Supported API and scope

Decision0126 establishes that a responsive listener and immutable provider header cannot prove UI callback activity. Official alpha.17 SDK6a151a95 already publishes Input::local_status(). Its documentation explicitly permits any-thread reads of cached loader metadata without starting sampling. It returns independent request/layout/link/read/sample/fresh flags and nullable age. Fields are weakly consistent and provide no world/session/player/current-UI identity or input-execution proof.

Add client_input_status to source companion0.22.0, retaining server0.48.0, SDK/ABI12.1, Rust1.99.1, loopback endpoint and existing installed-version checkpoint. This extends the copied-data listener exception from decisions0002/0025 only to the documented cached diagnostic getter. It grants no listener World access, UI/archive execution, sampling request, command submission or OS-input path. Existing frame queues and all write gates remain unchanged.

## Implementation and contracts

Put the empty argument schema, bounded scalar projection and explicit proof limits in game-free policy. Unknown requested/dry/confirm arguments refuse before the adapter runs. The adapter uses only the already-open optional Input handle and published cached getter. Missing handle or typed SDK refusal returns an error instead of fabricated all-false data. Opening Input elsewhere already serves existing snapshot/command functionality; this read opens no additional handle or sampling interest.

Preserve all raw known flags and nullable age. The fresh flag is SDK sampler metadata, not current UI or execution acceptance. Do not recompute it from other flags or age: unlike the internally consistent copied-UI age/stale record, this API explicitly reports weakly consistent fields. Always report weakly_consistent true, sampling_requested_by_this_read false, current_ui_verified false, commands_submitted false and rendered_verified false. No held controls, entity IDs, session, address or credential is returned.

The source catalog becomes37 server plus14 companion tools. Add a capability handle-availability boolean without claiming the getter currently succeeds. Current fixed client probes become15 read/refusal checks; they preserve weak combinations and unknown age while refusing malformed fields or proof claims. Core command actions and their admission are unchanged.

## Verification and native limits

An integration test first fails because the tool is absent. Game-free tests reject every input argument and preserve independent flag combinations, unknown/zero/old/large ages and false proof limits. The published SDK fake supplies raw cached status with partial/weak flags, unknown-age sentinel and typed server refusal. Repeated reads preserve values and submit zero commands. Dispatcher tests verify discovery, missing-handle refusal, unnamed-argument refusal and an untouched frame queue.

A source-generation quoting error is corrected before the first complete build; retain its failed compile log separately from the intended missing-tool reproduction. Five added Rust methods yield414 total executions; two Python cases yield104 methods. Native getter fidelity, actual diagnostic values and callback-cause discrimination remain unverified because no new DLL is installed. The current0.21.1 client intentionally lacks this source0.22.0 tool; installed metadata drift is expected, not an adoption or failed-upgrade claim.

The separately modified client/provider remains unqualified. Frozen review0123 concerns the archived0.21.1 combinations and is not rewritten or expanded to this new0.22.0 source. Its pending approval cannot certify this tool. A future matched deployment requires a stable installation, empty-server/closed-client prerequisites, exact new binaries/rollback and bounded native read/metadata/refusal acceptance. No lifecycle retry, avatar movement, focus or gameplay action is authorized by this feature.
