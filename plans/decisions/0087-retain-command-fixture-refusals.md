# 0087: Preserve synthetic command refusals before dispatch assertions

Date: 2026-11-17. Status: source qualified; intermittent refusal causes remain unresolved.

The current SDK candidate's first variant run exposes two existing command-fixture failures. One sees refused_or_unknown instead of queued. The terminal-reason loop discards the submission result and then unwraps an empty dispatch list. Neither original failure retains enough detail to establish its cause. Do not attribute it to storage, scheduling, metadata or SDK behavior without discriminating evidence.

Port only the two diagnostic assertions into the active SDK 12.1 source. The first includes the complete retained synthetic submission in the queued-state assertion. The second binds the submission result and checks queued with its retained result before accessing the dispatch list. Synthetic fixtures use no real credentials or server targets. No production code, tool contract, version or deadline changes. The normalized production prefix before the test module retains SHA-256 e84aef5f3425f7ec154bc006530a7c1c1d320bc888b042d6c5b6b10cfe67d91d.

Before editing, the active workspace passes 398 Rust executions. After editing it again passes 398 Rust executions, strict Clippy, formatting, both release builds and 96 Python tests. Serialized tests and two build jobs retain the established verification configuration. A green subsequent run does not explain the earlier refusals. Preserve future failure results and reproduce their exact error before changing admission or production timing.

No installed changes or game execution are part of this increment. Native interruption, storage failure and frame scheduling acceptance remain separate. Current-consumer native Menu qualification retains its independently staged artifacts and applicable approval gate. This diagnostic improvement supports quality goal 4 without declaring that fault-safety acceptance complete.
