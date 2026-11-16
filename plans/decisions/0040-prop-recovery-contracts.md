# 0040: Qualify prop recovery contracts before exposing lifecycle controls

Date: 2026-11-16. Status: SDK-model probes pass; native acceptance remains open.

Keep prop placement, exact ownership and safe recovery as independent gates. The optional SDK qualification workspace is pinned to official revision 70a468bd4d0001f5e7237bf09ff3b3b0ffc04fc2; it is not the MCP runtime dependency.

The new integration fixture checks two orientations of a long synthetic placement box. Both anchors are allowed and both commits queue, but the rotated box reaches a denied zone and its execution is refused. An allowed anchor and an admitted transaction do not prove permission for the complete placement volume.

For each of the nine published HeldState kinds, the fixture queues a mixed placement/removal and then changes the removed prop's state before execution. The model refuses the entire write without placing or removing a prop. A separate completed mixed write is queued for undo; state added to its created prop before undo execution prevents both removing that prop and restoring the old prop. The receipt remains UndoRefused and no automatic retry follows. An empty-state control completes undo and restores the original item and position without relying on the old entity ID.

Synthetic metadata and held-state flags are not native component decoding or loader atomicity proof. The tests do not validate actual inventory contents, labels, factory state, geometry, persistence or rendering. CustomPlayerString is one of the held-state kinds: a future ownership label can itself obstruct recovery. No exact-owner exception is assumed or enabled.

The next API proposal should expose the existing loader item-placement metadata as a read-only value, preserving unknown or unavailable bounds separately from a known box. It must not substitute the loader's fallback permission box for complete physical geometry. Exact label text, owner verification and state-safe recovery remain separate requirements. No proposed ABI change, SDK patch, runtime migration, gameplay write or new executor is activated by this decision.

Primary source: [published prop tests](https://github.com/enshwick/enshwick/blob/70a468bd4d0001f5e7237bf09ff3b3b0ffc04fc2/crates/enshwick/tests/world_props.rs) and [published native placement resolution](https://github.com/enshwick/enshwick/blob/70a468bd4d0001f5e7237bf09ff3b3b0ffc04fc2/crates/runtime/loader/src/world/transactions/game/props.rs).
