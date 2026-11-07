# ADR-0024: EVM Execution Engine Candidate and Fork-Compatibility Gates

- **Status:** Proposed; revm selected as preferred evaluation candidate only
- **Date:** 2026-11-07

## Context

APEX is a Rust-first hybrid Layer-1 targeting EVM-compatible execution. It needs a deterministic executor, canonical transaction handling, complete state access, receipts, gas accounting and fee settlement. The repository has no execution engine dependency or production EVM state adapter. The workspace is pinned to Rust 1.90.0.

## Decision

1. Evaluate revm as the preferred Rust EVM engine candidate.
2. Do not add or pin revm as a workspace dependency until its exact release, MSRV, feature graph, license, security posture and compatibility with the APEX toolchain have been tested.
3. Keep the engine behind an APEX-owned adapter. Revm/Alloy types must not become APEX canonical wire formats by accident.
4. Use a named frozen EVM fork. Osaka/Fusaka-era behavior is the initial candidate baseline; Glamsterdam and later changes are excluded until their specifications and activation are finalized and explicitly ratified.
5. Require official Ethereum execution fixtures and cross-client differential tests; upstream unit tests alone do not prove APEX conformance.

## Evidence

The revm documentation describes an execution API and framework API, integration with Reth and Foundry, Alloy primitive re-exports and an MIT license. Docs.rs currently lists revm 43.0.3, released 2026-09-24. The upstream MSRV policy explicitly allows the minimum Rust version to change with latest stable Rust; therefore APEX Rust 1.90 compatibility cannot be assumed.

Ethereum's roadmap lists Fusaka as deployed and Glamsterdam as in development. The Ethereum Foundation's 2026-10-22 notice postponed Glamsterdam activation on Hoodi with no replacement date at that time. This is a reason to avoid enabling development-fork behavior by default.

Ethereum's execution-specs project provides an executable reference implementation and fixtures; ethereum/tests provides common client test suites for EVM and transaction behavior.

## Consequences

- No revm dependency or EVM implementation is added in this ADR.
- No exact revm release or EVM fork is ratified.
- The APEX execution adapter, state model, fee settlement and conformance harness remain to be implemented.
- Phase 19 begins with a compatibility spike and explicit conformance gates.
- G1 remains BLOCKED; release remains NO-GO.

## References

- https://docs.rs/crate/revm/latest
- https://bluealloy.github.io/revm/
- https://ethereum.org/roadmap/
- https://blog.ethereum.org/en/2026/10/22/glamsterdam-hoodi-delay
- https://github.com/ethereum/execution-specs
- https://github.com/ethereum/tests
- https://steel.ethereum.foundation/docs/execution-specs/writing_tests/types_of_tests/
