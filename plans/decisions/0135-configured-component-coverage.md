# 0135: State configured component coverage explicitly

Date: 2026-11-20. Status: source correction on the existing isolated feature branch; matched native acceptance remains open.

The entity_components implementation resolves names from components.txt and checks only those entries. Its former description, "Every reflected ECS component", overstated that coverage. known_types counts resolved configured entries, including any repeated entries; it does not establish a complete reflected type inventory.

Server0.49.1 preserves total, known_types and entity rows, and adds component_inventory_source="resolved_configured_names", complete_component_inventory_verified=false and component_absence_verified=false. A game-free projection owns these limits. The source description and exact contract index match. SDK, compiler and companion versions remain unchanged; main0.48.0 and staged native qualification stay separate.

An SDK-fake reproduction independently proves UsedItem is present while an intentionally restricted catalogue omits that name. Before the correction, it fails on the missing explicit inventory source. The corrected adapter and a separate policy case preserve observations while denying completeness and absence. Initial fixture errors are retained separately and are not production failures.

Read-only native inspection on the unchanged server observes nineteen nearby entities. Eight freshly selected session IDs all return factory not_observed; a second bounded query still observes the eight IDs with matching positions, templates and configured component lists. These are non-atomic observations. No positive FactoryStation, physical absence, complete component inventory, crafting eligibility, queue completeness or safe-removal result follows. The private receipts contain no reusable entity selectors for future tests. Further native factory acceptance needs an independently known station.

No installed file changes, gameplay commands, client input, lifecycle retry or provider adoption occur. The source fix requires matched native response acceptance before branch promotion.
