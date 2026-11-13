# Local enshwick compatibility patch

The project uses enshwick alpha.14 plus the local client-event bridge extension described in this document. Cargo's patch entry expects the writable checkout at ../enshwick; the dependency cache must remain unchanged.

Base commit: 8fa382949551f0256a6505ed161cb28e5a0ea8f5 (v1.0.0-alpha.14). The patch appends a client-event subscription entry and raises the mod/loader ABI to 11.20. SDK and loader source still use the alpha.14 package version; this local build is not an upstream release.

This private ABI lineage collides with later official 11.x appends. A numerically newer official loader is not compatible with a consumer built against this private 11.20 SDK: the same function-table offset has a different meaning. Keep this exact SDK and extended loader paired. Before moving to the current-main extension (ABI 11.28), rebuild every consumer against that SDK and install its matching loader; minor-version admission alone cannot detect this collision. Do not mix the two extension lineages.

To reproduce, clone the public enshwick repository beside EnshroudedMCP, check out the exact base commit on a local branch, and apply enshwick-client-events.patch with git apply. Build/test the framework and the MCP workspace before installing anything. The new companion requires the extended loader on both its isolated research server and client; earlier ABI mods remain subject to the loader's compatibility checks. Do not install this experiment on shared hosting.

Keep the original loader and companion DLLs plus a cold isolated-world save backup. Exit the client and coordinate an empty isolated server before replacing a loaded DLL. Rollback replaces both loader and companion with the backed-up pair while stopped. Never replace the Steam DRM stub; the existing enshwick bootstrap is unchanged.

The patch uses existing named event-stream routes only. It does not supply a verified selected-tab or passively highlighted recipe getter. Current MCP selection values stay unknown; client event history is evidence of observed events, not applied crafting or current UI selection.

Upstream enshwick is MIT licensed; its license is retained in enshwick-LICENSE. No game data, build tables, executables or generated game catalogues belong in this patch directory. Local build output and runtime evidence remain ignored.
