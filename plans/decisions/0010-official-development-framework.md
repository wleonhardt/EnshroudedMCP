# 0010: Pin the exact official development framework pair

Date: 2026-11-14. Status: implemented, isolated installation and partial observation checks passed.

The active revision is `5f647f46d7f987e93d397e00596014b23f3c1112`, official ABI 11.30. Use its matching development artifact and rebuild server MCP 0.14.0 and companion 0.3.0 together. The alpha.15 package label is insufficient because its release tag uses ABI 11.28.

Preserve historical private compatibility source and rollback pairs. The old event patch is inactive. Use a separate sibling `enshwick-nightly` checkout with only the three-file SDK metadata patch exposing an existing native item field. No ABI slot, native layout, loader or game offset is changed by that SDK patch.

Keep loopback admission, typed policy gates, game-thread world scheduling, owning-callback UI snapshots, bounded named client-event copies, journaling and recovery requirements. Client IDs never become server selectors. All future semantic actions need an identified published execution route, explicit authority and their own validation.

Input masks and control bindings are observations, not vanilla menu or movement executors. Current tab/passive recipe remain unknown. Existing source and live evidence are scoped to the tested isolated fixtures; a native host-template or event route is not promoted merely because it compiled.

See `../../patches/README.md` for exact source and artifact reproduction. Shared deployments require separate authorization, matching builds, verified backups and coordinated disconnected players.


Current source amendment: official acd5550c607a38c983433f9807cc95564244fe0b retains ABI 11.30 and contains the local held-input row fix. Use the documented enshwick-native-control source checkout and matching artifact instructions in patches/README.md. Current source versions are server 0.20.0 and companion 0.5.1; isolated research runs server 0.19.0 and companion 0.5.1. The unchanged SDK passed 592 nextest tests. The new prop observation increment remains offline-only.
