# Independent shutdown-exit evidence

`process_exit.py` is a local Windows observation tool, separate from the MCP actuator. It never starts, stops, kills, restarts or controls a target. It holds one process handle with only limited query and synchronization rights, verifies PID, executable path and raw creation FILETIME, then waits once for at most 30,000 API milliseconds. Access or identity failures remain refused. Windows sleep can extend elapsed wall time.

The APIs and requested access rights are documented by Microsoft: [process rights](https://learn.microsoft.com/en-us/windows/win32/procthread/process-security-and-access-rights), [creation times](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes), [image path](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew), [process wait](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject) and [exit status](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getexitcodeprocess). No process-memory read, write or termination access is requested.

## Interface

The supervising fixture calls `capture_identity(server_pid)` for its explicitly selected process. Keep that identity private and independently choose the expected server executable. `Identity(pid, creation_time, executable)` supplies the expected fingerprint; `observe(identity, timeout_ms, ready=callback)` opens and checks a pinned handle before invoking readiness. The creation value is raw Windows FILETIME, not Unix time or .NET ticks. Capturing a fingerprint is not independent proof that the PID belongs to the intended server; the expected executable must come from the reviewed fixture.

The CLI accepts the same fingerprint. These PowerShell variables must already contain the fixture's reviewed values:

```powershell
python scripts/process_exit.py --pid $serverProcessId --expected-creation $serverCreationFiletime --expected-executable $serverExecutable --timeout-ms 30000
```

After successful identity checks the first JSON line is `state: "observing"`. Only then may a separate, already authorized native shutdown request proceed. The terminal line is `exited` with an exit code, `running` with a null exit code after timeout, or `refused` with a bounded reason. Reports omit raw PID, path and creation time. CLI exit 0 means observation completed, not that the target exited successfully: inspect `successful_exit` and `exit_code`. CLI exit 2 with a `running` JSON result means timeout; exit 1 with `refused` means observation refusal. Argument-parser errors can also exit 2, with diagnostics and no observation JSON; never infer a running target from an exit status alone.

A signaled process handle establishes exit independently of the numeric exit code, including a real exit code of 259. Successful exit, native shutdown acceptance, ordered SDK shutdown/save events, save integrity and post-restart recovery remain separate evidence. `final_save_verified` and `causal_completion_verified` are always false. Never retry the MCP shutdown because this observer timed out or refused; reconcile the original identity and journal first. A fresh observer cannot recover an exit code from a process it never opened while alive.

## Live fixture prerequisites

Before live use, verify the matched deployment, cold archive, disabled unrelated writes, coordinated access and empty SDK character roster. Capture and bind the exact isolated server process; wait for observer readiness; issue exactly one gated native request; retain its uncertain receipt and bounded event journal. Observe exit, remove the flag and restart separately. Compare complete captures, operation identities, admitted journal/event deltas and installed hashes. A changed save timestamp alone is not final-save proof.

Offline tests use disposable Python children only: zero/nonzero/259 exits, timeout with a still-running child, wrong creation/path and invalid arguments. They release children through ordinary stdin/EOF, without killing anything. These offline tests remain distinct from the bounded isolated game fixture below. Connected/loading-peer races and live fault cases remain unverified.

## Bounded native server lifecycle acceptance

The isolated alpha.17 / server 0.34.0 fixture accepts one deliberate native server_save with no connected player characters and the client closed. Missing write flags refuse both save and shutdown before dispatch. Same-session save replay returns the retained receipt without changing the journal. Independent save metadata changes, but the receipt correctly retains save_completed_verified and causal_completion_verified false; timestamps and an uncorrelated SDK event do not establish which request completed a save.

Two separately deliberate shutdown requests run in different server sessions. Each records native acceptance, shutdown_started and then world_saved_after_shutdown. The first external observer reaches its bounded timeout before the process exits; later process absence is observed, but that observer supplies no exit code. No request is repeated in that session. After a verified 91-member cold archive and restart/read-back, prior save/shutdown identities report durable_outcome_unknown and real reuse refuses without journal changes.

The second fixture attaches the independent observer's limited-query/synchronize process handle before native dispatch in the same supervising process. It verifies PID, creation identity and reviewed executable, then observes a signaled process handle with exit code 0. This avoids consuming the observation window during tool handoff. A fresh-session request is not a retry of the old process. The observer still reports final_save_verified and causal_completion_verified false.

A verified 93-member cold archive follows that exit and includes the save identity, both shutdown identities, prior operation ledgers and all ten client command identities. A separate final restart verifies every previous complete voxel capture and soil column, the explicitly restored research setting, exact archived ledger membership/hashes, all 12 installed binary/signature hashes, zero players and absent write flags. Native receipt identities are never reconstructed after reload. The isolated server is running for the next fixture; the client remains closed.

This is bounded preservation and recovery evidence for an unchanged known world, not proof that an arbitrary concurrent edit was captured by a particular save request. Connected/loading-peer races, held-save refusal, unsupported native builds, live audit/storage faults, shutdown reply loss and public-namespace installation remain separate acceptance gaps. No process kill, forced save, automated restart after unknown state or shared-server deployment occurs.
