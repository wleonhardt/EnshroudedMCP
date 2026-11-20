# 0177: Share live terrain annotation capacity explicitly

Date: 2026-11-20. Status: reproduced and fixed in the standalone viewer; native/rendered-game acceptance remains separate.

The offline scene accepts up to 64 caller pins. Live capture previously appended available player positions and up to 32 prop origins before validating the combined list. A valid full caller budget therefore failed with "at most 64 pins" after all three reads. Caller labels such as "Player 1" or "Prop 1" also collided with generated observation labels and failed duplicate validation.

Preserve caller pins in order. Fill remaining capacity with available player positions, then prop origins in their existing read order. Generate a numeric label suffix only when needed to preserve uniqueness. Keep the 64-pin output bound, 32-prop annotation bound and three read-only calls unchanged. Do not silently remove caller pins or reinterpret omitted observations as absent entities.

Add player position availability/shown/truncation counts beside the existing missing-position count. Compute shown/truncated prop coverage from annotations actually retained, including shared-budget omissions. Display the additive player coverage in the existing provenance panel. Scene format1 and CLI arguments remain compatible; MCP schemas, native modules, installed runtime and frozen connection qualification artifacts do not change.

Three regression cases fail before the fix: a full 64-pin caller list, a partially occupied budget, and colliding automatic labels including an already used suffix. They then pass alongside the existing renderer and capture cases. The fixture uses typed synthetic read results, preserves caller coordinates and keeps identity fields out of output. This proves local capture/annotation behavior, not native live-player correspondence or engine rendering. No new client/server launch or gameplay action is required.

Full source checks pass 488 Rust and 128 Python tests, strict Clippy, formatting, release builds and contracts. BrowserOS renders a generated synthetic 64-pin page, exposes the omission counts in provenance and permits selection of the last caller pin. Visual inspection confirms the diagnostic canvas and pin controls render. The owned browser tab is closed after verification. These browser checks establish local viewer behavior, not game-rendered surfaces or native player/prop agreement.
