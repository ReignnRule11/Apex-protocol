# Phase 19A — revm 43.0.3 compatibility spike

**Status:** Exact candidate dependency is isolated; Rust 1.90.0 incompatibility reproduced; toolchain raised to the declared minimum 1.91.0 and awaits CI validation.
**Protocol status:** No EVM fork or execution semantics are ratified.
**Release impact:** G1 remains BLOCKED.

## Objective

Prove whether the exact revm crate candidate can be resolved and compiled by the APEX workspace. This is a dependency and toolchain gate, not an EVM implementation.

## Candidate configuration

- Crate: `revm`
- Exact version: `=43.0.3`
- Default features: disabled
- Explicit feature: `std`
- Integration crate: `crates/apex-evm`
- Toolchain: workspace-pinned Rust 1.91.0
- CI checks: workspace format, Clippy with warnings denied, and full workspace tests.

## Reproduced compatibility finding

The first PR CI run, #42382412119, reached Clippy but failed before compilation because Cargo's MSRV check identified the candidate graph as requiring Rust 1.91.0. The error explicitly listed revm 43.0.3 and its 43.0.x components (including bytecode, context, database, handler, inspector, interpreter, precompile, primitives and state) as requiring rustc 1.91.0.

This was independently consistent with the published normalized manifest for `revm-context 43.0.3`, which declares `rust-version = "1.91.0"`. The prior APEX workspace/toolchain pin was 1.90.0. The failure is therefore a concrete MSRV mismatch, not a guessed incompatibility.

The workspace's `rust-version`, `rust-toolchain.toml` and CI toolchain action are now aligned at the exact 1.91.0 minimum declared by the candidate dependency graph. This is a deliberate, explicit toolchain change with reproducible CI enforcement; it is not a floating stable channel.

## Feature rationale

The exact version pin prevents silent minor/patch drift. Disabling default features avoids implicitly adopting the upstream default bundle (including optional precompile and tracing components) before APEX has reviewed its feature and cryptographic surface. The `std` feature is enabled explicitly for the current server-side node target. The final feature set must be justified by the required EVM fork and precompile suite.

## What this spike establishes

If the updated CI passes, it establishes only that Cargo resolved and compiled this selected dependency graph under the pinned Rust 1.91.0 toolchain and that the workspace's existing checks pass.

It does **not** establish:
- a verified revm execution API integration;
- support for a particular Osaka/Fusaka fork configuration;
- compatibility with official Ethereum execution fixtures;
- signature recovery, canonical transaction decoding or chain-ID enforcement;
- state database correctness or complete EVM state coverage;
- gas, receipts, EIP-1559 fee settlement or supply accounting;
- deterministic cross-client replay;
- acceptable performance, resource bounds or security.

## Required next evidence

1. Capture the exact updated CI run and compiler/toolchain versions.
2. Inspect the resolved transitive dependency graph and all licenses, build scripts, enabled features and cryptographic implementations.
3. Confirm the upstream release's supported API surface and dependency revisions.
4. Add a minimal execution harness using explicit block context and an in-memory test database only after the dependency graph is accepted.
5. Run the official execution-spec fixtures and compare results against an independent execution client.
6. Ratify a named fork and activation rules before exposing execution in a node.

## Stop conditions

If Rust 1.91.0 cannot compile the candidate, do not hide the failure with a floating toolchain, wildcard version, or unreviewed transitive override. Record the exact error and submit a separate compatibility decision with reproducible CI evidence.

If the candidate's feature graph requires unsupported or unreviewed cryptographic/precompile dependencies, keep the integration blocked until reviewed. Do not enable the upstream default feature set simply to make the build pass.

## References

- [revm 43.0.3 crate](https://docs.rs/crate/revm/43.0.3)
- [revm-context 43.0.3 manifest and declared MSRV](https://docs.rs/crate/revm-context/43.0.3/source/Cargo.toml)
- [revm architecture and API documentation](https://bluealloy.github.io/revm/)
- [Ethereum execution specifications](https://github.com/ethereum/execution-specs)
- [Ethereum execution test fixtures](https://github.com/ethereum/tests)

## Conclusion

The candidate is intentionally isolated behind `apex-evm`. The Rust 1.90.0 incompatibility has been identified and corrected at the declared minimum. A green compile spike is one prerequisite for an adapter implementation, not evidence that APEX can execute Ethereum transactions or safely launch a network.
