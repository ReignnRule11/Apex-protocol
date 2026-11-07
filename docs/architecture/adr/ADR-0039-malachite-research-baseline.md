# ADR-0039: Malachite v0.8.2 Research Baseline for Phase 20

**Status:** Accepted for isolated research evaluation only; production adoption NOT approved.
**Date:** 2026-11-07

## Context

APEX requires BFT proof-of-stake consensus and has previously preferred Malachite as a Rust-native Tendermint-style candidate. Earlier records described an MSRV incompatibility based on APEX Rust 1.85. The current workspace declares Rust 1.91.0.

## Evidence

- Upstream release: Malachite v0.8.2, published 2026-10-19.
- Exact upstream commit: a453b32155f4ba9f959724279b52f601bae92e09.
- Upstream workspace MSRV: Rust 1.88.
- APEX workspace MSRV: Rust 1.91.0; compatible.
- Upstream license: Apache-2.0.
- Upstream explicitly describes the project as alpha, under heavy development and not externally audited.
- Upstream v0.8.2 release notes include consensus/network behavior fixes. The upstream commit reports a five-validator rolling upgrade under load; this is not an APEX test or independent assurance.

## Decision

Pin Malachite v0.8.2 at the exact commit above as the Phase 20 research baseline. Keep CometBFT as the reference/fallback comparator.

Do not add Malachite to the production workspace in Phase 20A. Any subsequent dependency must be isolated in a dedicated adapter/spike crate, with exact crate and feature selection, lockfile pinning, full dependency/license/advisory review, and reproduced upstream tests.

## Safety boundary

The engine does not replace APEX application validity, canonical protocol codecs, state persistence, or signer safety. Before validator keys can be enabled, APEX must implement and test:
- chain-bound consensus signing domains and a dedicated key lifecycle;
- durable anti-double-sign coordinates and crash/restore protection;
- authenticated proposal/vote/certificate handling;
- deterministic application validation before proposal acceptance;
- validator-set transition semantics and equivocation evidence;
- replay-safe atomic coordination of engine, signer and application state;
- multi-process Byzantine, partition and recovery testing;
- independent security review.

No chain ID, signature suite, canonical consensus serialization, validator economics, block interval or finality target is ratified by this ADR.

## Consequences

- The old MSRV incompatibility is resolved; it is not a reason to block a contained spike.
- Alpha maturity, external audit absence, dependency graph and operational integration remain open risks.
- G2 remains BLOCKED. A green build or upstream test suite is not a production adoption decision.
- Reopen this ADR if the exact pinned candidate fails API stability, dependency, signer safety, recovery or adversarial test gates.

## References

- https://github.com/circlefin/malachite/releases/tag/v0.8.2
- https://github.com/circlefin/malachite/commit/a453b32155f4ba9f959724279b52f601bae92e09
- https://github.com/circlefin/malachite/blob/v0.8.2/code/Cargo.toml
- https://github.com/circlefin/malachite/blob/v0.8.2/LICENSE
- https://github.com/circlefin/malachite/blob/v0.8.2/RELEASE_NOTES.md
- docs/protocol/consensus-engine-evaluation.md
- docs/protocol/consensus-integration-spike.md
