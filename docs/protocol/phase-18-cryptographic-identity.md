# Phase 18 — Cryptographic Identity and Transaction Authentication

**Status:** Design baseline submitted for review; no cryptographic implementation or protocol algorithm is ratified by this phase.
**Release impact:** G0 and G1 remain BLOCKED.
**Scope:** Separate cryptographic roles, establish the EVM transaction-authentication candidate, and define the evidence required before keys, signatures or signed bytes become consensus-critical.

## 1. Why this phase is split into a design gate

The repository does not yet have a canonical production transaction envelope, an assigned chain ID, a selected BFT engine or a production P2P stack. Implementing signature verification before those boundaries are fixed would encode assumptions into consensus-critical code and create avoidable migration risk.

Phase 18 therefore starts with an explicit cryptographic role model and a reviewable algorithm proposal. It does not claim authenticated transaction processing is implemented.

## 2. Three non-interchangeable key roles

| Role | Purpose | Candidate / constraint | Must not be reused for |
|---|---|---|---|
| EVM transaction authorization | Authorize a state-changing EVM transaction and derive its 20-byte sender | secp256k1 ECDSA with Ethereum-compatible recovery semantics; proposal only | Validator votes or libp2p peer identity |
| Consensus validator signing | Authenticate proposals, votes and commit evidence | Unselected until the BFT engine and its exact revision are selected | Transaction authorization or peer identity |
| P2P peer identity | Authenticate a network peer and derive a stable peer ID | libp2p identity abstraction; Ed25519 support is mandatory in the libp2p peer-ID specification | EVM account identity or validator consensus identity |

A validator may operate all three roles on one machine, but the private keys, storage permissions, rotation procedures, backup policies and protocol domains remain separate. Key reuse is prohibited unless a future ADR proves a narrowly scoped compatibility requirement and security review approves it.

## 3. EVM transaction-signature proposal

**Recommendation for review:** use secp256k1 ECDSA transaction authorization with the Ethereum-compatible recoverable-signature model, subject to the exact EVM fork and transaction types selected in Phase 19.

The proposal is grounded in these compatibility constraints:

- EIP-155 defines chain-bound signing for legacy transactions and replay protection through a chain identifier.
- EIP-2718 defines typed transaction envelopes. Its security guidance strongly recommends including the transaction type in the signed data to prevent cross-type signature reuse.
- EIP-2 requires low-s transaction signatures after the Homestead fork to remove the simple ECDSA signature malleability described there.
- EIP-1344 exposes CHAINID as a 256-bit EVM value. Therefore the production chain-identity representation must not be prematurely constrained to the current prototype `ChainId(u64)`.

This is a compatibility proposal, not a decision to support every Ethereum transaction type or every historical fork. The exact accepted legacy/typed transaction set, EVM revision, signature recovery rules, precompile behavior and fork activation schedule must be pinned together.

## 4. Chain binding and replay protection

The production transaction signing domain MUST bind at least:
- the exact chain identity;
- transaction type and envelope version;
- the active execution/fork domain where required;
- every execution and fee field whose alteration must invalidate authorization.

The protocol MUST derive the sender from verified signature material. A caller-supplied sender field is never authoritative unless a future explicitly specified account-abstraction transaction type defines a different authentication model.

The protocol MUST reject:
- signatures that fail verification or recovery;
- invalid scalar ranges and zero values;
- high-s transaction signatures where the selected EVM rules require low-s;
- unsupported transaction types and non-canonical encodings;
- wrong-chain and wrong-fork signing domains;
- signature reuse across transaction types where the signing domain should differ;
- arithmetic overflow and out-of-bound inputs.

Nonce checking remains state-dependent and must be performed against the evolving block state. Chain binding does not replace nonce-based replay protection.

## 5. Canonical transaction identity

A transaction ID MUST be derived from the exact canonical signed transaction bytes using the hash function specified by the selected EVM transaction format. The signing digest and transaction ID are distinct values and MUST NOT be conflated.

Before implementation, the transaction codec decision must specify:
- legacy and typed envelope support;
- exact byte grammar and canonicality rules;
- maximum transaction bytes and field lengths;
- transaction-type allocation and fork activation;
- signing preimage for every supported type;
- transaction hash preimage;
- sender recovery and low-s validation;
- chain-ID width, encoding and allocation;
- error precedence for malformed versus invalid signatures.

No transaction hash or signing digest is currently standardized by APEX. Phase 14's SHA-256 account commitment hash is not an EVM transaction hash.

## 6. Consensus signing remains engine-dependent

No consensus signature algorithm is selected here. The selected BFT engine determines message types, signing bytes, key formats, vote extensions, validator identity and compatibility constraints. APEX must not wrap an arbitrary signature scheme around an engine and assume interoperability.

The engine evaluation must establish:
- exact engine and immutable revision;
- supported consensus key algorithms and signature encoding;
- canonical sign bytes, including chain/height/round/step and block identity;
- validator public-key registration and rotation;
- commit-certificate verification rules;
- durable anti-double-sign state and signer/application crash consistency;
- hardware signer or remote signer support and threat model;
- dependency, license, MSRV, maintenance and audit review.

A consensus key must be distinct from both the EVM account key and peer identity key.

## 7. P2P identity boundary

The network layer should consume the selected libp2p identity abstraction rather than directly depending on a concrete cryptographic crate's private-key representation. The libp2p peer-ID specification requires Ed25519 support and defines deterministic public-key serialization and peer-ID derivation.

The final network ADR must pin the libp2p release and specify:
- supported key types and peer-ID derivation;
- canonical key serialization and secure key-at-rest format;
- authenticated handshake binding chain ID, protocol version and capabilities;
- peer key rotation/replacement semantics;
- connection-level authorization and rate/resource limits.

A peer ID authenticates a network identity only; it does not prove validator membership or transaction ownership.

## 8. Required implementation sequence

1. Ratify chain-ID representation and allocation policy.
2. Select the EVM fork and supported transaction envelope set.
3. Ratify canonical transaction codec and exact signing/hash preimages.
4. Implement signature validation, sender derivation and transaction IDs behind a narrow crate boundary.
5. Add published known-answer vectors, negative vectors, differential tests against the pinned EVM client and fuzzing.
6. Only then connect authenticated transactions to state admission and execution.
7. Separately select the BFT engine and consensus signer after engine compatibility evaluation.
8. Separately pin libp2p and implement peer identity/handshake.

## 9. Required test evidence

- Ethereum official vectors for every supported transaction type.
- APEX-specific chain/fork/type-domain vectors.
- Cross-implementation sender recovery and transaction-ID agreement.
- Invalid scalar, malformed encoding, high-s, wrong-chain, wrong-type and replay cases.
- Fuzzing of transaction decoding and signature parsing.
- Property tests that mutation of any signed field invalidates authorization.
- Deterministic tests proving that no untrusted sender field can override recovered identity.
- Secret-handling tests and documented key-rotation/recovery procedures for each role.

## 10. Explicit non-deliverables

This phase does not implement production keys, a wallet, transaction signing, transaction signature verification, a transaction pool, consensus signatures, validator key management or libp2p networking. It does not select a chain ID, BFT engine, consensus signature algorithm, transaction codec or full cryptographic suite. No private keys or secrets belong in the repository or CI.

## 11. Exit criteria

Phase 18 is not complete until the chain identity and transaction format decisions are ratified, transaction authentication is implemented and tested against official vectors, and the consensus/P2P key choices are independently resolved against their selected upstream engines. G0 remains BLOCKED until all required protocol decisions and cross-language vectors are reviewed.
