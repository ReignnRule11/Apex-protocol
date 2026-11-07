# Phase 18 — Cryptographic Identity and Transaction Authentication

**Status:** Chain-ID type implemented as a non-wire primitive; transaction-envelope profile proposed for review. Cryptographic verification is not implemented.
**Release impact:** G0 and G1 remain BLOCKED.

## 1. Three non-interchangeable key roles

| Role | Purpose | Candidate / constraint | Must not be reused for |
|---|---|---|---|
| EVM transaction authorization | Authorize an EVM transaction and derive its 20-byte sender | secp256k1 ECDSA with Ethereum-compatible recovery, parity and low-s rules; candidate | Validator votes or libp2p peer identity |
| Consensus validator signing | Authenticate proposals, votes and commit evidence | Unselected until the BFT engine and exact revision are selected | Transaction authorization or peer identity |
| P2P peer identity | Authenticate a network peer and derive a stable peer ID | libp2p identity abstraction; Ed25519 support | EVM account identity or validator consensus identity |

Private keys, storage permissions, rotation procedures, backup policies and protocol domains remain separate. Key reuse across roles is prohibited by the current design.

## 2. Production chain identity

The EVM CHAINID opcode exposes a 256-bit value. A production chain identifier therefore cannot be represented by the existing prototype ChainId(u64).

This phase adds ChainId256([u8; 32]) to apex-types:
- in-memory representation is fixed-width 32-byte big-endian;
- conversion to/from bytes is lossless;
- from_u64 is a convenience constructor, not a restriction on the identifier space;
- zero detection is available, but genesis validation—not this primitive—must reject zero and duplicate network IDs;
- this type does not allocate an APEX network number and does not define RLP encoding.

For the proposed EIP-1559 envelope, chain ID is encoded as an RLP integer: minimal big-endian bytes with no leading zeroes; integer zero uses the empty byte string. It MUST NOT be encoded as a fixed 32-byte RLP string. Dev, staging, public-test and production genesis artifacts must each use distinct chain IDs and genesis hashes.

## 3. Initial transaction-envelope candidate

**Proposed initial profile:** EIP-2718 typed envelope, transaction type 0x02 (EIP-1559) only.

The signed payload field order is:
chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, destination, value, data, access_list, y_parity, r, s.

The signing digest is Keccak-256 over the type byte 0x02 concatenated with RLP of the unsigned fields.

The transaction ID is Keccak-256 over the complete canonical signed envelope, including type byte 0x02.

The type byte is part of the signing domain. Signature recovery uses secp256k1 and y-parity 0 or 1; low-s validation follows the selected EVM fork. The type-2 envelope includes an access list, even if empty.

Initial exclusions: unprotected legacy transactions, EIP-155 legacy transactions, type 0x01 envelopes, blob transactions, authorization-list transactions and unknown types. These are rejected unless a future ADR adds explicit support and fork activation. This deliberately narrow profile aligns with the proposed base-fee burn / priority-fee model while reducing initial transaction-type ambiguity.

The profile remains **proposed**, not ratified, until the EVM engine and fork are pinned and official transaction vectors pass differential tests.

## 4. Standards basis

- EIP-2718 defines typed envelopes and recommends including the type byte in signed data to prevent cross-type signature reuse.
- EIP-1559 defines type 0x02, its exact field order, RLP payload and Keccak-256 signing preimage.
- EIP-1344 defines CHAINID as a 256-bit value exposed to EVM code.
- EIP-2 requires low-s transaction signatures after Homestead.
- EIP-2930 defines the access-list structure used by type 0x02.

The exact EVM fork, supported precompiles, field bounds, transaction size limits, RLP decoder and cryptographic crate are still unselected. No crate dependency is added until the engine/toolchain compatibility review is complete.

## 5. Canonicality and replay requirements

The eventual decoder MUST reject non-canonical RLP integers (including leading zero bytes), malformed or trailing bytes, oversized fields, transaction types outside the active profile, chain IDs that differ from configured genesis identity, invalid signature scalars, zero scalars, high-s values where forbidden, parity outside 0/1, invalid signature recovery and nonce mismatch against the evolving block state.

The sender MUST be derived from the recovered public key, never trusted from an input field. The signing digest and transaction ID are distinct. Transaction identity is computed from canonical signed bytes. Chain binding and account nonce both participate in replay protection.

## 6. Consensus signing remains engine-dependent

No consensus signature algorithm is selected. The selected BFT engine determines message types, signing bytes, key formats, vote extensions, validator identity and compatibility constraints. Its exact revision, key formats, commit-certificate rules, durable anti-double-sign behavior, signer threat model, dependency/license/MSRV and audit evidence must be reviewed before implementation.

Consensus keys remain distinct from EVM transaction keys and peer identity keys.

## 7. P2P identity boundary

The network layer should consume the selected libp2p identity abstraction. The libp2p peer-ID specification requires Ed25519 support and defines deterministic public-key serialization and peer-ID derivation. Pin the actual libp2p release and define secure key storage, rotation, authenticated handshake, chain/version binding and resource controls in the network ADR.

A peer ID authenticates a network identity only; it does not prove validator membership or transaction ownership.

## 8. Implementation sequence

1. Ratify chain-ID representation and network allocation policy.
2. Pin EVM engine revision and execution fork.
3. Ratify type-2-only initial envelope profile, exact codec and field/resource bounds.
4. Add official EIP-1559 known-answer vectors and cross-implementation tests.
5. Implement bounded RLP decoding, signature validation, sender recovery and transaction IDs behind a narrow crate boundary.
6. Add malformed-input fuzzing, wrong-chain/type tests and signed-field mutation properties.
7. Connect authenticated transactions to state admission and deterministic execution.
8. Separately select the BFT engine/signature and libp2p release/key management.

## 9. Explicit non-deliverables

No wallet, private key, transaction signing, signature verification, transaction pool, consensus signer, validator key manager or P2P networking is implemented. No chain ID number is assigned. The candidate envelope is not ratified. No private keys or secrets belong in the repository or CI.

## 10. Exit criteria

Phase 18 remains open until chain identity allocation and the initial transaction profile are ratified; bounded canonical decoding, signature recovery, transaction IDs and replay protection are implemented against official vectors; and consensus/P2P key choices are resolved against their selected upstream engines. G0 and G1 remain BLOCKED.
