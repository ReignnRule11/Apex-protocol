# ADR-0025: Isolated revm 43.0.3 compatibility spike

- **Status:** Proposed pending updated CI and dependency review
- **Date:** 2026-11-07
- **Related:** ADR-0024, Phase 19

## Context

APEX needs a deterministic EVM implementation but must not couple canonical protocol formats or state semantics to a third-party engine. ADR-0024 identified revm as the preferred candidate for an integration spike, while leaving its exact version, feature set and EVM fork unselected.

The first compile spike on Rust 1.90.0 failed at Cargo's MSRV check. The failure identified revm 43.0.3 and its 43.0.x components as requiring rustc 1.91.0. The published revm-context 43.0.3 manifest independently declares Rust 1.91.0.

## Decision

Keep the exact revm 43.0.3 dependency in an isolated `apex-evm` crate, with default features disabled and `std` explicitly enabled. Raise the workspace `rust-version`, `rust-toolchain.toml` and CI toolchain action from 1.90.0 to the exact 1.91.0 minimum declared by the selected candidate graph.

This is a compatibility decision for the compile spike only. It does not ratify revm as the production engine, an EVM fork, an API, a cryptographic precompile set, or any transaction semantics.

## Consequences

- The previous MSRV failure is recorded and reproducible in PR CI #42382412119.
- The workspace has an exact toolchain pin rather than a floating stable channel.
- Cargo/CI must now prove that the full workspace still builds and tests on 1.91.0.
- The upstream default feature bundle is not implicitly enabled.
- APEX's public API and future canonical encodings remain engine-independent.
- A green compile result must be followed by dependency/license/security review, explicit fork configuration, official fixtures, differential testing and deterministic replay.

## Rejected shortcuts

- Floating or wildcard revm versions.
- Hiding the MSRV failure with an unpinned toolchain.
- Treating a successful compile as proof of execution correctness.
- Exposing revm internal types as APEX protocol or storage formats.
- Enabling all default features without reviewing precompiles, cryptography and tracing.
