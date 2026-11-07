# Consensus Adapter Contract — Phase 3

**Status:** design contract; not a complete application interface or engine integration.

## Purpose

Keep the selected BFT engine replaceable and prevent upstream engine types from leaking into APEX's protocol and execution crates. The APEX application remains responsible for deterministic protocol validity and state transitions. The engine remains responsible for its own consensus protocol, validator vote authentication and commit-certificate rules.

## Implemented preflight

The current `apex-consensus` crate defines:

- `ExpectedParent`: configured chain ID, protocol version, committed height and committed block hash.
- `ProposalHeader`: chain ID, protocol version, proposed height and parent hash.
- `validate_proposal_header`: checks chain identity, protocol version, exactly-next height and parent linkage, rejecting height overflow.

This is a structural preflight only. The in-memory structs are not a wire format and the function does not compute a block hash.

## Required engine adapter flow

1. Receive an engine proposal through a version-specific adapter.
2. Decode it using a future ratified canonical codec with strict size and canonicality checks.
3. Run header preflight against the locally committed parent.
4. Validate proposer eligibility and proposal-specific consensus rules using the pinned engine's documented API.
5. Validate the full application block deterministically: transaction signatures, nonces, balances, gas, execution, receipts, state root and supply invariants.
6. Return the engine-required application decision only after all required checks complete; reject or fail closed on unavailable state or malformed input.
7. On engine finalization, independently verify that the finalized decision corresponds to the expected chain, height, parent and proposal identity. Do not trust an unauthenticated callback or caller-provided boolean.
8. Apply and persist the finalized state transition with crash-safe commit semantics; make replay idempotent.
9. Derive validator-set updates deterministically from committed state and apply them at an explicitly specified height boundary.

## Signing and persistence requirements

Consensus signing must use a dedicated key domain, chain-bound context and durable anti-double-sign state. Signing state must survive crashes and restore procedures without allowing a validator to sign conflicting votes for the same consensus coordinates. Key rotation and recovery must be explicit. EVM transaction signing keys must not be reused by assumption.

Persist enough information to recover the engine and application consistently: engine state, signer state, application height/hash, validator-set version and commit metadata. Define ordering and atomicity across these records before enabling validator signing.

## Non-goals

This contract does not implement BFT, signatures, canonical encoding, transaction execution, storage, networking, validator economics or a finality proof. It does not authorize public validators or handling value.

## Required tests before engine integration

- Chain ID, protocol version, height, parent hash and overflow rejection.
- Canonical decode and malformed/oversized input tests once codec is selected.
- Deterministic proposal validation and state replay.
- Engine finality proof verification and invalid-certificate rejection.
- Crash injection around signing, engine persistence and application commit.
- No-double-sign across restart and backup/restore.
- Validator-set transition and equivocation evidence tests.
- Multi-process partition, delay, drop, reorder and Byzantine fault tests.
