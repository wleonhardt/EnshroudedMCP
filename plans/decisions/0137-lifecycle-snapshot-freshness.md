# 0137: Preserve lifecycle snapshot freshness beside copied-target matching

Date: 2026-11-20. Status: inactive source consumer and exact SDK comparison; native lifecycle acceptance remains open.

The lifecycle prototype discarded snapshot state, age and session while retaining a target-text match. Preserve those SDK facts in a nullable snapshot_metadata object. Keep typed snapshot errors, command state/phase/age and matching/absent/different target observations separate. No match becomes current-peer authentication, character/roster acceptance or retry authority.

The new regression fails on absent metadata before correction. It exercises Fresh, Stale, Off and Never copies with matching text, changed/zero sessions and nullable age. The existing unavailable-getter case now checks null metadata. ABI12.11, exact SDK33cf4dc1/ABI12.16 and independent patch replay each pass ten debug/release cases plus strict lint/format under exact Rust/Cargo1.100.0. Normalized sources agree. A first verification with an unintended working directory/concurrency is retained separately and is not the controlled qualification result.

The four-file inactive optional-consumer patch applies to ff0759f625594c92fa9e53bed41bd1db224be7b0 and reconstructs tree 5f7844bab7502b258dceefa39c04681bad022e61; SHA-256 473c9721d401748fbbd39f4bc7d089c5b6e55e2427126046823e917dfdaa3b8a. See qualification/sdk-lifecycle/README.md for commands and limitations. Product versions, dependency pins, schemas and runtime code remain unchanged. The observer lives only under cfg(test); the compiled ABI12.11 provider probe is unchanged.

A real local build also exposes unignored nested qualification targets. A before/after git check-ignore reproduction verifies a scoped ignore rule while keeping source visible. No build output is staged or removed. No credential, installed file, game process, provider adoption or native command is involved. Provider fidelity and independent Join/Leave observations remain required before a product route.
