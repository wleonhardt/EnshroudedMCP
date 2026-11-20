# 0158: Roster and hosted-status lifecycle guards

Date: 2026-11-20. Status: source fix; new native acceptance pending.

Decision0157 identifies missing lifecycle guards without claiming a runtime failure. Four independent SDK-fake reproductions now fail on successful old observations: queued players and hosted status return retained positive roster/world data after reload, and each tool returns data after shutdown before admission. Direct SDK reads independently retain hosting=true, a nonempty world key and one known roster entry before and after the lifecycle event. Thus empty SDK data cannot conceal the missing boundary.

Source0.51.2 reuses existing game-free active lifecycle/session policy before queueing players or hosted status and again before execution, requiring an unchanged epoch. World data is not read when scope is inactive or changes while queued. Keep output projection, names, account strings, nullable native meters, save/clock fields and all non-atomic observation limits unchanged. No current-client mapping, connection lock or safety authority is added.

The hosting=false status path remains an immediate no-world diagnostic before the new guard. Its preservation case passes before and after the fix using the published client-side FakeHost hosting preset and synthetic World handles; this grants no actual client World access or client execution acceptance. The fake reader alone declares both sides. Real mod side/installation is unchanged.

Five cases cover both queued tools across unchanged/reload/shutdown, each pre-admission shutdown path, and inactive nonhosting status with an empty scheduler queue. Pinned scheduler Debug is only an enqueue barrier for the isolated queue. No game memory or unsupported native action is used. The prior audit remains a historical review of0.51.1; these two paths now have source-tested guards, leaving eight dynamic paths needing discriminating tests.

Prior0.51.1 native evidence cannot qualify the changed boundary. Native lifecycle interruption, new current-player roster/vitals correspondence and rendered UI remain open. No installed file, client process, actor position, configuration, credential, save or runtime log is changed/read. No Windows MCP or capture occurs.

Baseline458 Rust/117 Python passes before changes. Four separate reproduction tests fail on successful retained observations; the nonhosting diagnostic preservation case passes before implementation. All five cases pass after the guards. Final463 Rust/117 Python, strict Clippy, formatting, both release builds and offline contracts pass. Exact publication audits remain required.
