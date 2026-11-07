# Phase 20C — Engine-Neutral APEX Application Boundary

**Status:** Implementation and exact-head CI validation in progress.
**Base main:** `7b52c164789905e16cd40b967e2fb4f905d3feb9`
**Release impact:** G1 and G2 remain BLOCKED. Production remains NO-GO.

## Objective

Define an APEX-owned boundary that does not expose Malachite types to protocol/application code and does not trust an engine's proposal or finality claims without explicit checks.

## Implementation

New module: `crates/apex-consensus/src/application_boundary.rs`.

- `ChainContext` binds a non-zero `ChainId256` and non-zero protocol version.
- `ParentBlock` carries the expected parent height and block hash.
- `BlockIdentity` binds chain context, height, parent hash and non-zero candidate block hash.
- `BoundaryLimits` bounds proposal payload and opaque finality-proof bytes; these are caller configuration, not protocol constants.
- `ProposalCandidate` and `FinalityClaim` keep their fields private and expose read-only accessors.
- `validate_proposal` checks chain/version, checked next-height arithmetic and parent linkage before invoking `ApplicationAdapter::validate_proposal`.
- `ValidatedProposal` can only be produced by that function and is consumed by finalization.
- `FinalityVerifier` is a separate engine-specific contract. The claim must match the exact validated block identity before the verifier is called.
- `VerifiedFinalizedBlock` has a private constructor and is created only after the verifier returns success.
- `finalize_proposal` invokes application commit only after that exact-identity finality check succeeds.

## Security and correctness boundaries

The module does not implement a real certificate verifier. The verifier adapter must authenticate the chain, protocol version, height, block hash, certificate domain, validator set, voting power and signature rules. A test verifier is used only to test orchestration and is not a production implementation.

The application adapter must perform canonical decoding, derive and check the block hash, validate every transaction, execute deterministically against the correct parent, and reject unsupported or malformed data. This module does not implement those operations; current execution work is incomplete.

The application commit contract requires atomicity, idempotency by block identity and stale-parent rejection, but the trait cannot enforce storage semantics. There is no durable replay journal, restart protocol, consensus-WAL/application-state atomic commit, or recovery procedure. A process crash after application commit but before consensus acknowledgement remains an unresolved cross-system ambiguity.

Proposal payload and finality proof bytes are opaque bounded byte vectors, not canonical APEX wire formats. No Malachite dependency is added to the production workspace and no engine actor, network, validator signer or finality process is started.

## Tests

The module includes tests for successful validation/verification/commit ordering, chain context mismatch, proposal context mismatch, wrong height, wrong parent hash, application validation rejection, invalid finality preventing commit, finality identity mismatch preventing verifier invocation, payload/proof bounds, zero context/limits, parent-height overflow and application commit failure.

These are boundary unit tests, not BFT safety/liveness tests or a production finality proof.

## Exit criteria

1. Exact-head Format, Clippy and workspace tests pass.
2. Negative tests demonstrate no application commit before successful finality verification.
3. Review confirms no engine-specific types leak into the APEX application contract.
4. Documentation states the missing production verifier, canonical codecs, full application validation, durable replay and atomic handoff.

**Decision:** Engine-neutral boundary prototype GO; consensus integration and production finality NO-GO.
