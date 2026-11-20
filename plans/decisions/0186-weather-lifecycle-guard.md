# 0186: Refuse weather reads across world lifecycle changes

Date: 2026-11-20. Status: source0.51.5 behavior qualified; native interruption remains unverified.

The historical lifecycle audit identified weather_read as a dynamic observation without the active admission/execution epoch pattern. Reproduce the actual dispatcher behavior using the published SDK FakeHost with positive weather weights and a world key. Independently confirm those SDK values remain available after the MCP lifecycle event, so an empty observation or absent SDK world cannot explain a refusal.

Before the fix, three tests fail: a queued weather_read returns retained weights after WorldLoaded changes the MCP epoch, after ShutdownStarted makes it inactive, and when the request is first admitted after shutdown. An unchanged active-world request passes and preserves the same weights, dominance and world key. These failures establish an MCP scope defect in the synthetic retained-state scenario, not observed native reload timing.

Source server0.51.5 applies the existing construction active-session check at weather admission, then rechecks active state and compares the admitted epoch before either weather.at or world.key is read on the game thread. Reuse the existing game-free session policy and established dispatcher pattern; do not add a universal dispatcher gate, alter scheduling/wait behavior, normalize weights or reinterpret unavailable weather as clear. Diagnostic flags and JSON shape stay unchanged on successful reads.

All four dispatcher cases pass after correction. Existing weather tests separately retain unavailable samples, clear/rain/snow/blizzard dominance, ambiguous or non-finite weights, raw finite values and no weather forecast mutation. The new fixtures also independently retain positive SDK weather/world data and confirm no forecast writes. Default SDK12.1/Rust1.99.1, companion0.22.0 and all54 tool contracts remain unchanged. The exact older0.51.4 SDK12.16 patch and its checked native candidates retain their historical scope.

The unchanged baseline passes488 Rust/131 Python tests plus required checks. After the fix,492 Rust/131 Python tests, strict Clippy, formatting, both release builds and source contracts pass before publication. No game/client launch, installation, native interruption or gameplay mutation is part of this source increment. Existing signed runtime and durable preservation are rechecked separately; source success is not native acceptance.

Next qualify source0.51.5 on a fresh isolated signed-provider server and keep controlled native interruption and independently rendered weather/shelter/actor-safety correspondence separate. The remaining unproven dynamic audit candidates are entity_health, props_in, entities_near and entity_components; reproduce each against retained positive SDK state before changing it. The historical audit is preserved as a dated snapshot, not rewritten to claim it originally found proven bugs.

The first post-bump Python run retains failures from fixed probes still expecting0.51.4. Updating only their declared version, discovery label and exact metadata expectations restores all131 cases. Required492 Rust/131 Python, strict Clippy, formatting, releases and contracts pass; original runtime/durable preservation also passes. Native qualification remains outstanding.
