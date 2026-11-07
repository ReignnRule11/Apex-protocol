# ADR-0022: Cryptographic Role Separation and Transaction Authentication Proposal

- **Status:** Proposed; pending protocol and security review
- **Date:** 2026-11-07
- **Decision scope:** Cryptographic identity boundaries and EVM transaction-authentication direction

## Context

APEX targets EVM-compatible execution while maintaining a separate BFT consensus engine and a libp2p-based network direction. The repository currently has no production transaction envelope, chain-ID allocation, BFT engine selection or P2P implementation. Its existing `ChainId(u64)` is a prototype and must not constrain the production EVM chain identity.

Using one keypair for transaction authorization, validator signing and peer identity would couple unrelated threat domains, complicate rotation and recovery, and risk cross-protocol signature reuse.

## Proposed decisions

1. **Separate key roles.** EVM transaction authorization, BFT consensus signing and P2P peer identity use separate key material, storage controls, lifecycle procedures and cryptographic domains.
2. **Transaction algorithm candidate.** For conventional EVM externally owned accounts, propose secp256k1 ECDSA with Ethereum-compatible public-key recovery and low-s transaction signature validation, subject to the pinned EVM fork.
3. **Typed signing domains.** For typed transactions, include the transaction type in the signed preimage as required by the selected EIP-compatible envelope. Chain identity and fork domain must be bound according to the selected transaction format.
4. **Chain identity remains open.** The production chain-ID type and allocation are not selected. The production representation must support the EVM's 256-bit CHAINID semantics; the current `ChainId(u64)` is not authoritative.
5. **Consensus algorithm remains open.** Select consensus key/signature algorithms only after selecting and reviewing the exact BFT engine revision.
6. **P2P abstraction.** Use libp2p's identity abstraction at the network boundary; its peer-ID specification requires Ed25519 support. Pin the actual libp2p version and key-storage format in the network ADR.
7. **No cross-role fallback.** A key or signature valid for one role must not be accepted as proof for another role.

## Rationale and standards evidence

- EIP-155 defines chain-bound signing for legacy Ethereum transactions.
- EIP-2718 defines typed transaction envelopes and strongly recommends including the transaction type in the signed data to prevent cross-type signature reuse.
- EIP-2 rejects high-s transaction signatures after the Homestead fork to address the documented ECDSA malleability.
- EIP-1344 specifies CHAINID as a 256-bit EVM value.
- The libp2p Peer IDs and Keys specification requires Ed25519 support and defines deterministic key serialization and peer-ID derivation.

References:
- https://eips.ethereum.org/EIPS/eip-155
- https://eips.ethereum.org/EIPS/eip-2718
- https://eips.ethereum.org/EIPS/eip-2
- https://eips.ethereum.org/EIPS/eip-1344
- https://github.com/libp2p/specs/blob/master/peer-ids/peer-ids.md
- https://libp2p.github.io/rust-libp2p/libp2p_identity/index.html

## Consequences

- No cryptographic crate is added until canonical message formats and upstream compatibility are pinned.
- No transaction signature verification is claimed to exist.
- The production chain identity requires a new type/codec decision rather than widening the prototype implicitly.
- Consensus signing and anti-double-sign remain blocked on engine selection and durable signer design.
- P2P identity remains blocked on libp2p version selection and network handshake specification.
- G0 and G1 remain BLOCKED.

## Rejected alternatives

- **One keypair for all roles:** rejected because it collapses distinct trust domains and complicates incident containment and key rotation.
- **Choose Ed25519 for all roles:** rejected because conventional Ethereum EOA transaction compatibility requires Ethereum-compatible secp256k1 recovery semantics; consensus engine compatibility is not yet known.
- **Choose consensus signatures before the BFT engine:** rejected because it risks incompatible signing bytes, key encodings and certificate verification.
- **Implement a custom transaction signature envelope now:** rejected because canonical transaction encoding, chain identity and fork support are not ratified.
