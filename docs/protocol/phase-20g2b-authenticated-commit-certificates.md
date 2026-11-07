# Phase 20G2B — Authenticated Consensus Signatures and Commit Certificates

**Status:** Merged; exact-head and post-merge main CI passed. Experimental and not ratified.
**Base main:** 75d2bafb80b9dcb12211712e97399bc7a0131a38
**Scope:** Ed25519 validator identity binding, canonical authenticated validator-set hashing, and strict precommit certificate verification.

## Objective

Move beyond unsigned consensus preimages and caller-supplied signer IDs. A certificate is accepted only when every included signature verifies against a scheduled validator public key and the authenticated signers exceed two-thirds of scheduled voting power.

## Implementation

- Pin ed25519-dalek 3.0.0 exactly, with default features disabled and only fast and zeroize enabled.
- Pin sha2 0.11.0 under a separate workspace dependency alias; preserve the existing SHA-256 0.10.9 dependency used by other crates.
- Derive validator IDs from a domain-separated SHA-256 digest of an algorithm identifier and canonical 32-byte Ed25519 public key.
- Reject invalid/weak Ed25519 public keys and require an exact one-to-one mapping between validator-set identities and public keys.
- Construct a canonical authenticated validator-set hash over a domain/version, bounded u32 member count, sorted validator ID, voting power and public key.
- Provide an authenticated validator-set schedule with exact activation-height semantics.
- Verify only precommit certificates; bind chain ID, protocol version, height, round, signer identity, validator-set hash and block hash in the existing canonical signing preimage.
- Strictly verify each Ed25519 signature, reject unknown/duplicate signers, reject excess signature cardinality, and require strictly greater than two-thirds of scheduled voting power.
- Keep the implementation in the consensus crate; do not add Malachite to the production workspace or enable keys/validators.

## Security properties and boundaries

The verifier authenticates signatures against the key registry supplied in its authenticated validator set, and computes quorum from that set's voting powers. It does not authenticate how the initial set or later schedule updates were authorized. The schedule is caller-supplied local configuration, not governance proof.

The certificate is an in-memory structure, not a canonical network decoder or wire format. Callers must bound and authenticate bytes before constructing it. This phase does not connect verification to Malachite, the application boundary, the durable signer journal or atomic finalized-state commit. It does not prove BFT safety/liveness, implement anti-double-sign key custody, handle key rotation, or provide a production protocol ratification.

No production validator key, real funds, public network or mainnet may use this experimental profile.

## Tests

The module includes deterministic test-key fixtures and exercises real Ed25519 signing/verification, strict quorum boundary, wrong chain/height/block/set hash, tampered signatures, duplicate signers, unknown signers, validator-set hash binding and exact schedule activation. CI must additionally pass root workspace format, Clippy/tests and all existing isolated Malachite/harness jobs.

## Exit criteria

1. Exact pinned cryptographic APIs compile under Rust 1.91.0.
2. All signatures are strictly verified against registered keys.
3. Validator identity is derived from its public key, not trusted from certificate input.
4. Quorum uses only authenticated, unique signers and scheduled voting power.
5. Negative tests fail closed and the existing CI matrix remains green.
6. Documentation and repository tree distinguish certificate verification from integrated BFT finality.

## Next subphase

20G2C should connect this verifier and durable signer reservations to an actual isolated Malachite runtime, then test multi-validator process behavior, restart recovery and certificate-gated application commit. No production promotion before independent protocol, dependency and security review.
