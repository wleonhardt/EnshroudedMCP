# 0110: Open terrain sections through the selected point

Date: 2026-11-17. Status: source and archived-native-data replay verified; browser visual acceptance pending.

The technical terrain viewer previously required manually choosing a view and slice to inspect a selected pin, cell or proposed target. Add X/Y through selection and Z/Y through selection buttons using the existing renderer and slice controls. The selected point must lie inside all three half-open capture bounds. Select its exact containing half-metre cell; never clamp an outside point into observed space. Disable both controls before selection and for an outside selection. A defensive click check also refuses disabled/outside actions, including a zero-index slice correctly.

Reuse the shared selected-column path for pins, clicked cells and proposed targets. No interpolation, mesh, collision, clearance or contact claim is added. Proposed targets remain geometry-only and outside observations remain unknown. There is no network dependency, game input, new MCP tool, SDK change, version bump or installation.

The feature test fails before implementation because sectionIndex does not exist. Pure geometry checks cover both axes, inclusive minima, exclusive maxima on every axis, below-bound points, nonfinite coordinates, invalid views, zero slice and immutable inputs. Existing DOM event tests now cover pin selection, actual canvas cell picking, proposed target selection, exact section captions and disabled-click refusal. All 16 terrain tests pass; the complete private cohort passes 409 Rust executions and 98 Python tests, strict lint/format, both release builds and live server contracts.

An offline replay uses previously captured native survey data, preserving its source hashes. All 4,096 cells and 16 pins are retained. Thirteen pins are inside and three outside; 6,688 independent index/cell comparisons pass. This is archived-data replay, not a fresh native capture or browser visual test. A standalone private HTML artifact is produced for later visual acceptance. BrowserOS is unavailable, so layout, actual browser input and rendered appearance remain unverified. No alternate browser, Windows MCP or capture is used.

The signed runtime guard and staged Menu-review preflight pass. Pending review 0107 remains byte-identical; its archived source counts and authorization scope are unchanged. Game client stays closed. No installed file, credential, live write flag, world geometry or durable runtime namespace changes. Public verification and publication results are recorded separately.
