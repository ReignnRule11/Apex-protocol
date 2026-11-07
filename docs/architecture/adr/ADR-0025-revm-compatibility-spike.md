# ADR-0025: Isolated revm 43.0.3 compatibility spike

- **Status:** Proposed pending CI and dependency review
- **Date:** 2026-11-07
- **Related:** ADR-0024, Phase 19

## Context

APEX needs a deterministic EVM implementation but must not couple canonical protocol formats or state semantics to a third-party engine. ADR-0024 identified revm as the preferred candidate for an integration spike, while leaving its exact version, feature set and EVM fork unselected.

The workspace is pinned to Rust 1.90.0. Upstream revm documents that it tracks current stable Rust and may change its MSRV. Compatibility must therefore be demonstrated, not assumed.

## Decision

Create an isolated `apex-evm` crate and add an exact dependency on revm 43.0.3 with default features disabled and `std` explicitly enabled. Use the existing Rust 1.90.0 CI toolchain as the compatibility test.

This is a candidate dependency pin for the spike only. It does not ratify revm as the production engine, an EVM fork, an API, a cryptographic precompile set, or any transaction semantics.

## Consequences

- Cargo/CI now has a concrete candidate graph to resolve and compile.
- The upstream default feature bundle is not implicitly enabled.
- APEX's public API and future canonical encodings remain engine-independent.
- CI failure blocks further integration until the compatibility issue is understood.
- A green compile result must be followed by dependency/license/security review, explicit fork configuration, official fixtures, differential testing and deterministic replay.

## Rejected shortcuts

- Floating or wildcard revm versions.
- Unreviewed toolchain upgrades to satisfy a dependency.
- Treating a successful compile as proof of execution correctness.
- Exposing revm internal types as APEX protocol or storage formats.
- Enabling all default features without reviewing precompiles, cryptography and tracing.
