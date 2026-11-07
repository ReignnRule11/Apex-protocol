# ADR-0023: EVM Chain-ID Width and Initial Transaction Envelope Profile

- **Status:** Proposed; chain-ID primitive implemented, transaction profile pending EVM pin and review
- **Date:** 2026-11-07

## Context

APEX targets EVM-compatible execution and a hybrid fee market with base-fee burn and proposer priority fees. The existing ChainId(u64) was introduced for early prototypes and is too narrow to model the EVM CHAINID value. The protocol has not yet pinned an EVM engine/fork or a production transaction codec.

## Decisions proposed

1. **Chain-ID width:** production chain identity is an unsigned 256-bit integer. Its in-memory representation is 32-byte big-endian. APEX adds ChainId256; the old ChainId(u64) is prototype-only.
2. **Wire encoding:** when carried by the proposed EIP-1559 envelope, chain ID is an RLP integer with minimal big-endian bytes and no leading zeroes; zero is the empty byte string. It is not a fixed 32-byte RLP byte string.
3. **Network allocation:** no numeric APEX chain ID is allocated here. Genesis must assign a unique, immutable, non-zero value; dev, staging, public test and production networks must differ. The registry/allocation process remains open.
4. **Initial transaction profile:** propose EIP-2718 type 0x02 (EIP-1559) as the only initially accepted transaction envelope.
5. **Excluded types:** legacy transactions, type 0x01, blob type 0x03, authorization-list type 0x04, and unknown types are rejected unless explicitly added by a later versioned ADR and fork activation.
6. **Signing and identity:** type byte 0x02 is included in the signing preimage. Signing digest is Keccak-256 over type byte plus RLP-encoded unsigned fields. Transaction ID is Keccak-256 over the complete signed envelope.
7. **Signature candidate:** secp256k1 ECDSA recovery, parity 0/1 and low-s validation according to the selected EVM fork.

## Rationale

EIP-1344 specifies CHAINID as a 256-bit value. EIP-155 provides chain-bound legacy signing, while EIP-2718 provides a typed envelope and recommends type inclusion in signed data. EIP-1559 defines type 0x02, its field ordering and its exact signing preimage. EIP-2930 defines the access-list semantics used by the type-2 envelope. EIP-2 defines the low-s transaction-signature restriction.

The type-2-only profile avoids accepting historical unprotected signatures and limits the first implementation surface. It also matches the current fee-market direction. This is not a claim that all Ethereum transaction types are supported.

## Implementation boundary

The Rust ChainId256 primitive is a fixed-width value type only. It does not implement RLP, assign network IDs, sign transactions, verify signatures or define a genesis schema. The transaction profile remains proposed until the EVM engine/fork and codec implementation are pinned and differential tests pass.

## Required evidence

- EIP-1559 official signed/unsigned transaction vectors.
- EIP-2718 typed-envelope boundary and unknown-type rejection tests.
- RLP canonical integer vectors, including zero and values with leading zeroes.
- 256-bit chain-ID round-trip and chain-mismatch tests.
- Cross-client sender recovery and transaction-hash agreement.
- Low-s, invalid scalar, parity and malformed RLP tests.
- EVM differential tests against the exact pinned engine/fork.

## References

- https://eips.ethereum.org/EIPS/eip-1344
- https://eips.ethereum.org/EIPS/eip-155
- https://eips.ethereum.org/EIPS/eip-2718
- https://eips.ethereum.org/EIPS/eip-1559
- https://eips.ethereum.org/EIPS/eip-2930
- https://eips.ethereum.org/EIPS/eip-2
- https://ethereum.org/developers/docs/data-structures-and-encoding/rlp/

## Consequences

- The existing u64 prototype identifier cannot be used as a production chain identity.
- The APEX chain-ID number remains unassigned.
- A canonical type-2 envelope implementation cannot begin until the EVM engine/fork and resource limits are pinned.
- Consensus signing and peer identity remain separate decisions.
- G0 and G1 remain BLOCKED; release remains NO-GO.
