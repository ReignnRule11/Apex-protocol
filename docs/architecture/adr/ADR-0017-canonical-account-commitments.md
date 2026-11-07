# ADR-0017: Canonical Account Encoding and State Commitment Prototype

- **Status:** Accepted for research prototype; production protocol selection withheld
- **Date:** 2026-11-07

## Context

The existing state model contains only AccountId(u64), nonce(u64) and balance(u128). It has no EVM storage, code, system-account, validator-set or receipt state. A complete Layer-1 state root cannot be claimed from this model.

## Decisions

1. Encode the current account record as APEX magic, u16 schema version, u64 AccountId, u64 nonce and u128 balance, all integers big-endian, exactly 38 bytes.
2. Strictly reject wrong length, magic and unsupported version.
3. Use SHA-256 from the RustCrypto sha2 crate, pinned to 0.10.9, for this prototype. SHA-256 is specified in NIST FIPS 180-4.
4. Build an ordered binary Merkle tree using the RFC 6962 recursive split shape. APEX-specific distinct domains are used for account leaves, internal nodes, empty tree and top-level root.
5. Sort leaves by numeric AccountId. The root commits to schema version, account count and tree root; it is content-addressed and does not include StateVersion or block height.
6. Keep the implementation in a separate apex-commitments crate.

## Consequences

Identical logical account maps produce identical roots regardless of insertion order, storage backend or StateVersion. The prototype is not a complete state commitment because the state model is incomplete. Inclusion/non-inclusion proofs, incremental updates, state sync, EVM storage and protocol activation rules are not implemented.

## References

- NIST FIPS 180-4: https://csrc.nist.gov/pubs/fips/180-4/upd1/final
- RFC 6962, Section 2.1: https://www.rfc-editor.org/rfc/rfc6962.html
- sha2 0.10.9: https://docs.rs/sha2/0.10.9/sha2/
