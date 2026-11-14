# 0011: Bound native administrative reposition requests

Date: 2026-11-14. Status: implemented; one limited isolated two-command fixture passed with explicit arrival and recovery limits.

Add one dry-by-default server tool through the published `Effects::teleport_player` route. Require a sole connected matching actor with observed positive health, finite project bounds, and a requested target within 16 m Euclidean distance. Bind a real request to an exact current actor/session/generation/position/target preview. Joined, left and death events invalidate generation. Client UI IDs never select server actors.

Authority is the trusted loopback operator plus the existing fresh write flag and exact confirmation. The target's kick/ban administrator bit is not a substitute for that authority. No role or security setting changes.

Run only through the server scheduler. Synchronize an exclusive bounded durable identity/intent and journal before retaining a once marker and one dispatch attempt. Canonical same-ID replay returns the stored outcome without redispatch; altered requests and stale durable identities refuse. Storage/native failures retain identity and uncertainty. Do not invent an arrival job, acknowledgement endpoint, automatic retry or generic command chain.

The native engine searches for a best spawn point and supplies no causal arrival receipt or published correction radius. The 16 m gate bounds the requested point, not adjusted displacement. Preserve the native orientation default without adding arbitrary angles. Submission remains issued_unknown, with no collision, hazard, exact-arrival or complete-state guarantee.

A return requires a new preview, identity and gated request. It is not native undo and cannot restore complete player state. Later position/rendered observations stay separate from command acceptance. A zero-player refusal cannot establish preview-session-specific rejection.

The companion, official revision and ABI 11.30 stay unchanged. Keep construction and reposition operation records distinct and backed up together with journals/saves. Raw runtime context, previews and screenshots remain private.
