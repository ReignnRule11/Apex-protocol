# Phase 20A — Consensus-Engine Candidate Revalidation

**Status:** Complete; merged and independently verified. No consensus engine dependency or runtime integration.
**Base main:** 4390fe2b40a5cdf07dba89fd5af3f07dbcaae67d
**Release impact:** G2 remains BLOCKED. Production remains NO-GO.

## Objective

Revalidate the previously preferred Rust-native BFT candidate against the current APEX toolchain and record a reproducible upstream research baseline before introducing consensus-engine dependencies.

## Evidence and decision

- Candidate: Malachite, https://github.com/circlefin/malachite
- Release: v0.8.2, published 2026-10-19.
- Exact upstream commit: a453b32155f4ba9f959724279b52f601bae92e09.
- Upstream workspace version: 0.8.2.
- Upstream declared MSRV: Rust 1.88.
- APEX declared MSRV: Rust 1.91.0; compatible.
- Upstream license: Apache-2.0.
- Upstream maturity statement: alpha, under heavy development, not externally audited.
- Upstream reports a five-validator rolling upgrade from 0.8.1 to 0.8.2 under load. This is upstream evidence, not an APEX result or independent assurance.

The earlier Rust-version mismatch is no longer a blocker. The alpha maturity and audit gap remain material. The v0.8.2 release notes include fixes in proposal round validation, commit-certificate sync round handling and validator-proof session behavior. These reinforce the need to pin and test the exact revision.

**Decision:** Pin v0.8.2 at the exact commit above as a research baseline only. Do not add Malachite to the production workspace, enable validator signing, or claim APEX finality. CometBFT remains the fallback/reference comparator.

## Scope delivered

- Corrected the prior stale MSRV statement: APEX is Rust 1.91.0, not 1.85/1.90.
- Recorded exact upstream release, source commit, license and maturity statement.
- Defined Phase 20B–20F as gated work rather than one large engine integration.
- Updated README, production readiness, consensus evaluation, integration spike and consensus overview.
- Added ADR-0039 and this phase record.

## Explicit non-deliverables

No Malachite dependency, engine binary, network service, validator key, consensus signature algorithm, canonical vote/proposal codec, commit certificate verifier, validator economics, anti-double-sign store, or finality implementation is added here.

## Next phases

- 20B — isolated dependency/API spike: select exact crates/features; resolve and inspect the full dependency graph and license/advisory metadata; compile a separate adapter crate; reproduce upstream tests.
- 20C — APEX application boundary: deterministic proposal validation, engine callbacks, finalized decision verification and replay-safe commit handoff.
- 20D — signer safety: ratified consensus key/signature domain and durable anti-double-sign across crash, restore and rotation.
- 20E — validator/finality semantics: validator updates, activation boundaries, certificate verification and equivocation evidence.
- 20F — multi-process adversarial harness: at least four independent validator processes, network faults, Byzantine behavior, restarts, churn, replay and bounded-resource tests.

## Exit criteria

Phase 20A is complete: this evidence and decision are merged, and post-merge main CI passed on `104cec631eca96ef4e95adcb355963cd8944a93b`. It does not close G2 or authorize production adoption.

**Decision: RESEARCH SPIKE GO; PRODUCTION ADOPTION NO-GO.**
