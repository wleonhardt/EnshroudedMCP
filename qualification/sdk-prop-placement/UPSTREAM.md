Submission checkpoint, 2026-11-16: these are historical drafts against 70a468bd. Inspected main dc0cf4ce is now ABI 12.4 and occupies the proposed slot 207. Rebase, regenerate coordinated interfaces and requalify before submitting a current-main PR. Explicit posting authorization is still pending; no issue, PR or maintainer message was sent.

# Proposed upstream API request

Title: Expose read-only prop placement metadata

## Game system

Building and placed props, using the server's runtime item registry.

## What the mod needs to do

A planning mod needs the same local placement corners that the SDK's prop transaction resolver uses before it submits a placement. ItemInfo exposes a placed-template identity, but no public local-placement-box getter. A metadata read avoids guessing fields, constructing a zero-position placement or spawning a probe merely to discover the bounds.

The local proposal wraps the existing item resolver. It returns the item, copied template and optional declared local corners without normalizing them or substituting the fallback permission box. Unknown bounds are not an empty footprint. Permission, complete collision geometry, label ownership and safe recovery remain separate concerns.

## Language

Both Rust and Luau. The consuming planning mod is Rust.

## Example

```rust
fn inspect(world: enshwick::world::World, item: enshwick::ItemId) -> Result<(), enshwick::WorldError> {
    let info = world.prop_placement_info(item)?;
    if let Some(bounds) = info.local_box {
        enshwick::info!("Local corners: {:?} to {:?}", bounds.min, bounds.max);
    }
    Ok(())
}
```

## Would you send a pull request?

Yes. A source-only implementation against exact main revision 70a468bd is prepared. It appends ABI 12.2 data and a host entry, preserves existing slots, reuses the native resolver and coordinates Rust, C header and generated Luau definitions. The full upstream check passes with 4,136 affected tests and nine marked skips. Native provider/value qualification remains open, and nothing is installed into the game.

# Proposed draft PR

Title: feat(world): expose read-only prop placement metadata

Expose World::prop_placement_info so mods can inspect the existing resolver's copied template and optional local corners without creating a transaction or prop. Preserve raw corners and unknown bounds; introduce no new engine offsets or signatures. Append one headed ABI record and host slot in minor 12.2, and coordinate the safe SDK, loader, fake, C header, book and Luau declarations.

The new regression covers value preservation, unknown/refused/unavailable states and no prop/transaction edits. Generic headed guards include the new record. The upstream scoped check passes: 4,136 tests passed, nine marked skipped, plus C/C++ layout, lint, formatting, doctests and docs. Optional game-dependent inputs were absent, and 61 pre-existing external book links were not checked. Native provider/value compatibility and complete physical footprint, ownership and recovery guarantees are not established. No consumer migration or runtime installation occurred.
