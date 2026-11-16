# 0043: Recover caller command identities through a read-only inventory

Date: 2026-11-16. Status: source verified; native Menu inventory acceptance passes; connected faults remain open.

After a lost reply or companion reload, recovery should not require browsing the filesystem or guessing a replacement operation ID. Add client_commands with no arguments. Merge retained current-boot outcomes with caller operation identities from the existing bounded reservation namespace, sort and deduplicate, and return the same outcomes as client_command_status. There can be at most 64 current outcomes and 64 durable names, including failed reservations retained only in memory, so the union is bounded at 128 records.

Read only validated command-<operation>.json filenames and regular-file metadata. Never decode their contents, recover native IDs, dispatch, reserve or journal. More than 64 entries, invalid names and non-regular entries refuse the inventory. Prior-boot state remains unavailable with automatic_retry and native_identity_reconstructed false. Missing reservation directories produce an empty durable set; other filesystem errors remain explicit. Any subsequent action retains existing previews, once-only identities and write admission.

The new regression first fails on the missing inventory method. Coverage checks sorted unknown-content reservations, current/durable deduplication, unchanged journal and SDK dispatch count, reload uncertainty, exact bounds and malformed entries. Additional adapter coverage drives applied into each published cancellation reason, an unknown reason and unavailable, preserving terminality and replay without additional input or terminal audit records. Synthetic SDK status does not prove real disconnect timing or rendering. Fixed client probes summarize only counts/states and omit caller identities, payloads and errors.

The official product SDK remains 6a151a95 / ABI 12.1. The inactive placement proposal is unrelated. Native installation, joined fault/disconnect acceptance and actual rendered outcomes remain separate gates.

Private and independent sanitized-public checks pass 358 Rust executions (including 19 repeated shared-module checks), 59 Python tests, strict Clippy, formatting and both release builds. Seven unusable copied contexts refuse all six command families before reservations, journal entries or SDK dispatch. Native deployment remains unverified.

## Native acceptance

Signed sanitized source 6389439030ad5375778797141f81a9cee276cc16 passes its required checks and exact 164-file publication audits. Matched 0.41.0/0.19.0 installation verifies 18 prior-boot identities against individual status reads, repeat equality, five unknown-selector refusals and six Menu dry-action refusals. The read creates no reservation or journal row and submits no client command. All 13 fixed client and 26 server probes, 1,296 synthetic planner cases, branching/overlap balances and full world baselines pass.

Four exact rollback DLLs, all 12 installed hashes, eight official framework/signature files and the new runtime source closure are verified. The initial cold installation archive has 133 members. A launch mistakenly inherited the default loopback port; this was corrected with explicit process environment settings and a second native shutdown, preserving a 135-member correction archive. Both exact-process observers report exit code 0. This is not causal final-save proof. The two stops add exactly eight unique journal rows and two durable markers under four archive aliases; all old journal prefixes and other immutable ledger bytes are unchanged. There are no gameplay/client-command writes, active flags or players. Future launches must specify the dedicated and client-host/companion ports before process creation.

Fresh native Menu is separate from a joined command context. Current-boot live inventory outcomes, real disconnect timing, rendering and crafting remain unverified. Windows MCP/capture remain off. The SDK placement proposal stays inactive and upstream submission authorization remains pending.
