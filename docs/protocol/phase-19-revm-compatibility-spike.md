# Phase 19A — revm 43.0.3 compatibility spike

**Status:** Compile-only candidate integration submitted for CI validation.
**Protocol status:** No EVM fork or execution semantics are ratified.
**Release impact:** G1 remains BLOCKED.

## Objective

Prove whether the exact revm crate candidate can be resolved and compiled by the APEX workspace using the currently pinned Rust 1.90.0 toolchain. This is a dependency and toolchain gate, not an EVM implementation.

## Candidate configuration

- Crate: `revm`
- Exact version: `=43.0.3`
- Default features: disabled
- Explicit feature: `std`
- Integration crate: `crates/apex-evm`
- Toolchain: workspace-pinned Rust 1.90.0
- CI checks: workspace format, Clippy with warnings denied, and full workspace tests.

The exact version pin prevents silent minor/patch drift. Disabling default features avoids implicitly adopting the upstream default bundle (including optional precompile and tracing components) before APEX has reviewed its feature and cryptographic surface. The `std` feature is enabled explicitly for the current server-side node target. The final feature set must be justified by the required EVM fork and precompile suite.

## What this spike establishes

If CI passes, it establishes only that Cargo resolved and compiled this selected dependency graph under the current CI toolchain and that the workspace's existing checks still pass.

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

1. Capture the exact CI run and compiler/toolchain versions.
2. Inspect the resolved transitive dependency graph and all licenses, build scripts, enabled features and cryptographic implementations.
3. Confirm the upstream release's actual declared Rust requirements and supported API surface.
4. Add a minimal execution harness using explicit block context and an in-memory test database only after the dependency graph is accepted.
5. Run the official execution-spec fixtures and compare results against an independent execution client.
6. Ratify a named fork and activation rules before exposing execution in a node.

## Stop conditions

If Rust 1.90.0 cannot compile the candidate, do not hide the failure with a floating toolchain, wildcard version, or unreviewed transitive override. Record the exact error and submit a separate toolchain decision with reproducible CI evidence.

If the candidate's feature graph requires unsupported or unreviewed cryptographic/precompile dependencies, keep the integration blocked until reviewed. Do not enable the upstream default feature set simply to make the build pass.

## References

- [revm 43.0.3 crate](https://docs.rs/crate/revm/43.0.3)
- [revm architecture and API documentation](https://bluealloy.github.io/revm/)
- [Ethereum execution specifications](https://github.com/ethereum/execution-specs)
- [Ethereum execution test fixtures](https://github.com/ethereum/tests)

## Conclusion

The candidate is intentionally isolated behind `apex-evm`. A green compile spike is one prerequisite for an adapter implementation, not evidence that APEX can execute Ethereum transactions or safely launch a network.
