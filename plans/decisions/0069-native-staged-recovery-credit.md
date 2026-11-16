# 0069: Native accepted-stage recovery credit

Date: 2026-11-16. Status: bounded connected native acceptance passed; capacity faults and cold recovery remain open.

The unchanged SDK 6a151a95 / ABI 12.1 server 0.45.0 and companion 0.20.2 execute a tiny, quiescent, prop-free fixture with one full-health character. No movement is requested. A five-cell frame and one-cell centre fill share one aligned 512-cell brick. Independent surveys compare every cell against explicit expected images; native captures separately include props.

| Phase | Recovery owners | Retained recovery bytes |
|---|---:|---:|
| Before construction | 0 | 0 |
| Frame applied | 1 | 196608 |
| Frame accepted | 1 | 196608 |
| Fill applied | 2 | 393216 |
| Fill accepted | 2 | 393216 |
| Fill undone and baseline-verified status settled | 1 | 196608 |
| Frame undone and baseline-verified status settled | 0 | 0 |

Acceptance leaves recovery credit untouched. The later placement preserves the older operation's allowance. Reverse native undo and exact baseline verification release only the completed owner's remaining credit. Duplicate acceptance produces no extra journal row. This is positive session-local lifecycle evidence, not proof of near-capacity admission, storage capacity, append/sync faults, disconnect recovery or crash persistence.

Independent broader comparisons pass 19,312 cell observations across eleven overlapping protected regions at filled, reopened-frame and restored-baseline phases. These are observations, not 19,312 unique cells. Existing platform/headroom and its prop list remain unchanged. The source sample retains its material/fill through construction previews. All twelve installed hashes, character position and full health are preserved. Original pacification is explicitly restored; all four write flags are absent.

The original journal is an exact byte prefix. Eighteen new rows account for two pacification transitions and the two construction/apply/accept/reverse-undo sequences. Exactly two new construction identities exist; the other 87 original ledger files remain byte-identical. Native completion and baseline settlement are distinct from durable replay authority after a restart.

The operator independently sees the initial frame/open gap and the reopened gap after fill undo. The operator looked away during fill; rendered closure remains unverified. No equivalent placement is repeated merely to obtain that observation. No collision, traversal, terrain contact, functional workshop or kept-build persistence acceptance follows.

Private/public source checks retain 373 Rust executions, 80 Python tests, strict Clippy, formatting and release builds. Current native server/client metadata contracts match the index. No runtime implementation, installed package, SDK or public tool schema changes.
