# 0136: Reconstruct current feature source against SDK 12.16

Date: 2026-11-20. Status: inactive source qualification; native provider and lifecycle acceptance remain unverified.

Qualify exact feature base 27ffaa1481e7e1fe0e5c55aa8a38703f49f52f70 against official SDK 33cf4dc149f88d1abe3d3c11a12053d0ab94967f/ABI12.16 and Rust 1.100.0. Preserve main and feature dependencies, compiler, installed files and pending native baseline. The reproducible patch and commands live in qualification/sdk-lifecycle/README.md.

The migration changes sixteen paths, limited to exact pins/provenance, source profiles/probes and SDK-fake initialization prerequisites. SDK 12.16 Input construction starts snapshot and held sampling. Explicit initialization handling preserves an empty observation and the requested diagnostic flag; the existing final admission fixture still checks exactly two request reads and durable-intent ordering. No runtime gate or deadline changes. Initial assertion, non-unwinding callback and cached-flag failures remain separate evidence, not native crash diagnosis.

Candidate and independent replay each pass 424 Rust/107 Python, strict Clippy, format, release builds and source contracts. All 325 tracked files match byte for byte; both indices reconstruct Git tree 049da547eb234955b055fb7e401e7817ce2e48fe. The patch is SHA-256 5f2f8d1bc156c014fee4fac96099abce8c0eae08803141906619c8f3b83b6669. No new product tool, lifecycle route, profile/credential reader, provider adoption, installation or game action occurs. Native fidelity and independent Join/Leave acceptance remain separate prerequisites.
