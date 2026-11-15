# 0018: Deliberate native dedicated-server shutdown

Date: 2026-11-15. Status: implemented and source verified; live acceptance pending; installed runtime unchanged.

Published World::stop requests the dedicated server's normal shutdown after the current step, final save and process exit; acceptance is immediate and does not prove any of those later outcomes. It refuses clients/hosted games and unsupported builds. Add server_shutdown through Scheduler::next_tick, never a process-kill wrapper. No automatic restart, save hold, configuration editor or countdown is introduced.

Require an operation identity (1..64 lowercase ASCII letters, digits, underscores or hyphens). Dry preview is the default and returns the active lifecycle session and observed empty SDK player-character roster. Real requests require that session, the existing fresh flag and exact confirmation. The SDK roster omits loading clients and is not an admission lock; coordinate access separately. Recheck session, roster, deadline and write gate after durable intent work. Cancel queued work on pre-start timeout.

Reserve bounded exclusive identities under server-shutdown/ before dispatch, sync intent and shared journal, and retain an uncertain marker before native execution. Same-identity replay never dispatches again. Prior-session/durable identities remain blocked; dry requests for retained identities report status. Permit at most one native attempt per active lifecycle session in the running controller, including uncertain native errors. Do not automatically remove identities or retry after timeout, lost reply, refusal or journal failure.

Native acceptance does not establish process exit, successful final save or rollback. Live acceptance needs a verified cold archive, empty controlled fixture, exact process identity, observed process exit, separate restart and complete native read-back/ledger/binary preservation. The final save is SDK behavior to verify separately; a changed timestamp alone is not causal proof. Keep this dedicated-host operation separate from the missing client join/logout/process executors.
