# Phase 20E — Validator-Set Lifecycle Primitives

**Status:** Implementation submitted for exact-head CI validation.
**Base main:** `8a672599ea2a96abe2404b58afc997c2a6e42544`
**Release impact:** G2 remains BLOCKED. Production remains NO-GO.

## Objective

Establish deterministic, fail-closed validator-set data structures and lifecycle semantics before connecting an engine, validator keys or finality claims.

## Implementation

- `apex-consensus::validator_set` introduces fixed-width opaque `ValidatorId` and `Validator` voting-power entries.
- `ValidatorSet::new` rejects empty sets, all-zero identities, zero voting power, duplicate identities and checked aggregate `u64` overflow.
- Accepted entries are sorted by validator identity, independent of caller input order.
- Quorum accounting rejects unknown or duplicate signer identities and requires strictly more than two-thirds of total voting power. Arithmetic uses `u128` intermediates.
- `ValidatorSetSchedule` accepts complete replacement sets at strictly increasing, nonzero activation heights. A replacement is selected at its exact declared height.
- `EquivocationClaim` carries one validator, height, round, step and two distinct caller-supplied message digests. It is an unverified evidence envelope only.

## Semantics and deliberate boundaries

The activation schedule is deterministic local data. It does not authorize changes, authenticate a governance proposal, define validator registration/staking/exit, or prescribe an engine-specific update delay. A future protocol decision must ratify update authority, activation timing, validator identity/key binding, power limits, and set commitment encoding before network use.

Quorum arithmetic over supplied identities is not certificate verification. No public-key parsing, signature verification, canonical consensus codec, signing-domain binding, commit-certificate verification, finality proof or Malachite integration is implemented. The equivocation envelope does not establish that either digest came from a valid signature or that the signer violated a ratified consensus rule.

No validator reward, bond, unbonding, slashing, governance, key rotation, HSM/KMS, anti-rollback backup anchor or validator networking is included.

## Tests

- Input-order-independent canonical sorting.
- Empty set, zero identity, zero power and duplicate identity rejection.
- Aggregate voting-power overflow rejection.
- Strict greater-than-two-thirds boundary.
- Duplicate and unknown signer rejection.
- Exact activation-height behavior immediately before, at and after activation.
- Zero and non-increasing activation-height rejection.
- Equivocation envelope rejects identical message digests and invalid identity.

## Exit criteria

1. Exact-head Format, Clippy and workspace tests pass.
2. Tests establish deterministic ordering, checked arithmetic and exact activation boundaries.
3. Docs distinguish quorum arithmetic/evidence envelopes from authenticated finality.
4. No engine, signer or validator key is enabled by this phase.

**Decision:** Validator-set data-model prototype GO; authenticated validator lifecycle and production finality NO-GO.
