# APEX Protocol Specification v0.2 — Completeness Baseline

**Status:** Draft for technical review. Not ratified. Not production-ready.  
**Supersedes:** v0.1 as the current working draft; historical v0.1 remains unchanged.  
**Scope:** Protocol domains, validity pipeline, state model, transaction/block contracts, deterministic execution requirements, consensus integration boundary, monetary invariants, genesis, upgrades and failure behavior.

This specification intentionally distinguishes normative architectural constraints from decisions that still require evidence. A value marked **TBD** is not an implicit default and MUST NOT be selected by an implementation silently. A protocol freeze requires the G0 evidence in `docs/operations/production-readiness.md`.

## 1. Protocol identity and design constraints

APEX is an independent proof-of-stake Layer-1. Its implementation language is Rust. Its architecture is hybrid: a deterministic application/state machine is separated from a BFT consensus engine, with explicit interfaces and independently testable subsystems. EVM-compatible execution is the target execution environment. Initial data availability is full on-chain transaction data. Optional Bitcoin anchoring is an external evidence publication mechanism only.

### 1.1 Normative constraints

- Every consensus-critical input MUST be authenticated, bounded, versioned and canonically decoded before it can affect state.
- Given identical parent state, protocol version, finalized block input and deterministic execution rules, every conforming node MUST derive identical receipts, validator changes, monetary accounting and application state root.
- Consensus MUST NOT depend on a node's local wall clock, locale, filesystem order, random source, RPC response or Bitcoin availability.
- The application MUST validate the complete proposed block before voting or accepting finality. A consensus-engine callback is not, by itself, application validity.
- State application MUST be atomic with respect to the finalized block identity and application state. A partially applied block MUST never be externally reported as committed.
- All parsing, queues, execution, proofs, RPC and peer requests MUST have explicit resource limits.
- Unknown critical versions, unsupported features, malformed encodings and ambiguous commit outcomes MUST fail closed.
- Historical prototype formats are not automatically promoted to production protocol formats.

### 1.2 Non-goals

APEX does not claim Bitcoin-equivalent security, permissionless validator decentralization before demonstrated, EVM equivalence before conformance testing, a bridge, a fixed TPS/finality guarantee, or production readiness from unit tests alone. The optional Bitcoin anchor does not validate APEX execution or replace APEX consensus.

## 2. Identity, domains and versioning

A production network MUST distinguish:
1. **Chain identity:** immutable identifier committed into genesis and all transaction/consensus signing domains.
2. **Network environment:** separate development, public-test and production genesis identities; environments MUST NOT share replay domains.
3. **Protocol version:** explicit version for transaction, block, state schema, consensus adapter and upgrade activation.
4. **Execution fork:** explicit EVM ruleset active at a deterministic height/epoch.
5. **Software version:** implementation release metadata; software version alone MUST NOT change consensus rules.

A production EVM chain ID MUST be an unsigned 256-bit integer, represented internally as 32-byte big-endian bytes. This aligns with the EVM CHAINID opcode's 256-bit stack value. For RLP transaction fields, it MUST be encoded as a canonical positive integer (minimal big-endian bytes; zero as the empty byte string), not as a fixed 32-byte RLP string. The production network MUST assign a unique, immutable, non-zero value in genesis; no APEX chain-ID number or registry allocation is approved yet. The existing ChainId(u64) remains a historical prototype type and MUST NOT be used in production EVM, genesis or transaction-authentication interfaces.

Every signature and hash MUST include a versioned, purpose-specific domain and the chain identity. Transaction, consensus vote, validator registration, governance, state commitment and checkpoint signatures MUST use distinct domains. A key MUST NOT be reused across roles by assumption.

## 3. Canonical data model

The Phase 14–16 account record is a research fixture only: `AccountId(u64)`, nonce `u64`, balance `u128`. It is not a production account model and its root MUST NOT be activated as a protocol state root.

The production logical state MUST contain the following independently versioned domains:

| Domain | Required logical contents | Determinism requirement |
|---|---|---|
| EVM accounts | 20-byte address, nonce, 256-bit balance, code hash/code reference | One canonical account per address |
| Contract storage | address + 256-bit slot key + 256-bit value | Zero-value and absent-slot semantics fixed |
| Contract code | content-addressed bytecode and code hash | Exact byte sequence; bounded code size |
| Validator registry | consensus identity, consensus key, status, stake, activation/exit heights, commission metadata if adopted | Deterministic ordering and transitions |
| Staking/bonding | delegations, unbonding entries, lock/withdrawal heights | Exact rounding and release rules |
| Monetary accounting | total issued, burned, outstanding supply and issuance counters | Supply conservation invariant |
| Consensus parameters | active validator set, voting power, timeouts and versioned parameters | Effective-height semantics |
| Governance | proposal, vote, timelock, execution and veto state | Deterministic tally and activation |
| Execution artifacts | transaction index, receipts, logs and cumulative gas | Canonical per-block ordering |
| Protocol metadata | chain ID, protocol version, fork schedule and upgrade state | No host-derived values |

The state-root specification MUST commit to every state domain that can influence future validity, execution, monetary supply, consensus membership or governance. Derived indexes MAY be omitted only if they are reconstructible deterministically and cannot affect validity. Operational metrics, local mempool contents and peer scores are not consensus state.

### 3.1 Account model constraints

The target execution model is EVM-compatible and therefore requires 20-byte account addresses and 256-bit EVM values. The current `AccountId(u64)` MUST remain isolated as a prototype type. Migration to the production model requires a new versioned codec and a new state commitment version; Phase 14 roots MUST NOT be reinterpreted or silently migrated into production roots.

Exact nonce width, balance representation, code/storage trie structure, empty-account semantics, precompile set and address derivation are decisions for the EVM/cryptography ADRs and MUST match the selected upstream execution engine.

## 4. Transaction contract

A production transaction is an authenticated, chain-bound, versioned envelope. It MUST carry enough information to validate sender authorization, replay protection, fee affordability and execution deterministically.

The logical envelope MUST define:
- envelope type and version;
- chain identity and execution-fork domain;
- sender authorization/signature data;
- sender nonce;
- destination (or contract-creation marker);
- value and bounded calldata;
- gas limit;
- fee cap and priority-fee cap, with explicit ordering constraints;
- access-list or equivalent execution hints only if adopted;
- canonical transaction identity derived from canonical signed bytes.

The initial transaction-envelope candidate is the EIP-2718 typed envelope with EIP-1559 transaction type 0x02 only. Its payload follows the EIP-1559 field order and RLP rules, including chain ID, nonce, priority-fee cap, fee cap, gas limit, destination, value, calldata, access list and signature fields. The signing digest is Keccak-256 over 0x02 concatenated with the RLP-encoded unsigned fields; the transaction ID is Keccak-256 over the complete canonical signed envelope. Transaction type 0x02 MUST be present in the signing preimage. Legacy transactions, type 0x01 access-list transactions, blob transactions, authorization-list transactions and unknown types are excluded from the initial candidate profile unless a later ADR explicitly adds them with fork activation and conformance vectors. The type-2 profile inherits the access-list field semantics defined by EIP-2930. The candidate requires secp256k1 ECDSA signature recovery with parity 0/1 and low-s validation according to the selected EVM fork. The exact execution fork, codec implementation and supported field bounds remain subject to pinning a specific EVM engine revision. APEX MUST NOT claim Ethereum transaction compatibility until official upstream vectors and differential tests pass.

### 4.1 Admission and execution order

For each transaction, block validation MUST:
1. Strictly decode within byte and field bounds; reject non-canonical encodings.
2. Verify chain identity, transaction type and active fork.
3. Verify sender authorization and signature domain.
4. Derive sender identity from authenticated signature data; never trust a caller-supplied sender field.
5. Verify canonical transaction hash and duplicate/replay rules.
6. Check nonce and fee-cap invariants against the evolving block state, not a stale pre-block snapshot.
7. Check maximum upfront affordability using overflow-safe arithmetic.
8. Execute under deterministic gas and resource rules.
9. Produce canonical receipt/logs and actual fee settlement.
10. Update state and cumulative block gas in transaction order.

A failed transaction MUST have precisely specified rollback semantics: transaction-local state changes revert, but protocol-defined nonce/fee effects may persist only if the selected EVM semantics require them. Invalid block-level structure rejects the whole block. These semantics must be pinned to the EVM fork and tested, not improvised.

### 4.2 Transaction identity and replay protection

Transaction identity MUST be a cryptographic hash of the canonical signed envelope with an explicit domain. Identical signed bytes MUST yield the same ID across implementations. Malleable signatures or alternate encodings MUST NOT create ambiguous identity. Replay protection MUST include chain identity and account nonce, and any additional replay domains introduced by transaction types.

## 5. Block and receipt contract

A block's logical header MUST commit to:
- chain identity and protocol version;
- height and parent block hash;
- consensus-defined timestamp;
- transaction-data commitment and transaction count;
- receipts commitment;
- complete post-state root;
- validator-set commitment/version effective for the block;
- execution fork/parameter version.

The finalized block identity MUST be unambiguous and derive from canonical header bytes. Consensus commit certificates are authenticated consensus artifacts bound to the exact chain, height, round/step, block ID and validator-set version. Their encoding and whether they are stored inline or as separately addressable evidence are implementation choices, but verification semantics are protocol-critical.

A receipt MUST commit to transaction index/ID, execution status, gas used, effective fee, logs and any protocol-defined error/output data. Receipt ordering MUST equal transaction ordering. Cumulative gas MUST be monotonic and bounded by the active block gas limit. Reverted execution MUST have explicit log/state rollback semantics.

The exact field widths, timestamp unit/constraints, hash algorithms, canonical codec, maximum block bytes, maximum transactions, gas schedule and receipt encoding remain **TBD** and must be ratified together with test vectors.

## 6. Consensus contract

APEX uses a hybrid architecture: an independently maintained deterministic application state machine is driven by a separately versioned BFT engine. The project will integrate a mature BFT implementation rather than invent a new protocol.

The production consensus specification MUST define:
- validator eligibility, voting-power calculation and set update boundary;
- proposer selection and deterministic tie-breaking;
- proposal, prevote/precommit or equivalent protocol phases;
- rounds, timeouts, nil votes and timeout progression;
- commit certificate structure and verification;
- quorum threshold and the exact assumptions under which safety holds;
- evidence format, evidence age and deterministic penalties;
- validator activation, exit, unbonding and key rotation;
- state synchronization, restart and catch-up;
- behavior under partitions, long outages and validator-set changes.

The often-quoted less-than-one-third Byzantine voting-power condition MUST NOT be presented as an APEX guarantee until the selected engine's exact protocol, weighting, synchrony and validator assumptions are documented and tested. Malachite remains a research candidate; no engine is integrated or production-selected.

Consensus signer state MUST durably prevent double-signing across crash, restart, backup and restore. Consensus keys MUST be separate from EVM transaction keys and peer identity keys. Signing state and finalized application state require a crash-safe cross-subsystem commit/recovery protocol before public validator operation.

## 7. State transition and commitment

The state transition function is conceptually:

`(S[h], B[h+1], P[v]) -> (S[h+1], R[h+1], V[h+1], M[h+1])`

where S is complete protocol state, B is the proposed block, P is the active protocol/fork parameter set, R is ordered receipts, V is the next validator set and M is monetary accounting.

The function MUST be deterministic and side-effect-free until the validated transition is committed. All state domains influencing validity MUST be committed by the complete state root. Proofs MUST support inclusion and non-inclusion for each authenticated domain or use a documented authenticated map with equivalent semantics. Root authenticity and finality come from verified consensus, not from a proof alone.

The current account-only SHA-256 root and proof APIs are prototypes and MUST remain separately versioned. A production commitment design requires an explicit tree/data-structure ADR, domain-separated hash suite, canonical key ordering, proof vectors, cross-language verifier and migration/activation plan.

## 8. Monetary policy and invariants

The selected direction is hybrid:
- a hard maximum supply ceiling;
- predetermined issuance bounded by the ceiling;
- transaction fee market with base-fee burn and proposer priority fees;
- no discretionary administrator minting.

The following invariants are mandatory regardless of parameter values:
1. No state transition may cause outstanding supply to exceed the maximum supply.
2. Issuance is possible only through the published deterministic schedule and authorized genesis allocation.
3. Burn reduces outstanding supply exactly once.
4. Fee collection, burn and proposer allocation reconcile exactly to the charged fee.
5. Integer rounding is deterministic and any remainder destination is specified.
6. No governance action can retroactively alter finalized balances or silently expand the supply ceiling.
7. Validator rewards and slashing must conserve or explicitly account for supply.

Maximum supply, genesis allocation, issuance curve, validator reward share, base-fee adjustment constants, minimum/maximum fee, minimum stake, unbonding period, slashing rates and reward rounding remain **TBD**. They require economic simulations, adversarial analysis and governance review. No tokenomics value is approved by this draft.

## 9. Validator lifecycle

The protocol must specify a deterministic lifecycle:
1. registration and proof of control of consensus identity;
2. bond/delegation and eligibility checks;
3. activation at a defined future height/epoch;
4. active voting with bounded voting power and deterministic ordering;
5. evidence submission and adjudication for objectively provable offences;
6. exit request, unbonding delay and withdrawal eligibility;
7. key rotation with overlap/activation rules and no identity ambiguity.

The exact minimum stake, maximum voting-power cap, validator count, delegation, commission, unbonding and slashing parameters remain TBD. Slashing MUST be limited to objectively verifiable, versioned offences; no arbitrary administrator seizure. Evidence retention and appeal/appeal-like review semantics must be designed before economic activation.

## 10. Genesis and network identity

Genesis MUST be a canonical, reproducible artifact containing:
- immutable chain identity and protocol version;
- genesis timestamp/height semantics;
- initial account allocations;
- initial validator identities, keys and voting power;
- monetary parameters and initial supply accounting;
- initial consensus and execution parameters;
- fork activation schedule;
- governance/bootstrap authority policy;
- hashes of all referenced artifacts.

Every node MUST derive the same genesis block ID, state root and validator-set hash from the same genesis bytes. Genesis tooling MUST reject duplicate addresses/keys, invalid supply totals, invalid validator weights, unknown parameter versions and non-canonical ordering. Dev, staging, public test and production networks MUST use distinct chain identities and genesis hashes.

No production genesis values are approved in this draft.

## 11. Governance and upgrades

Consensus-critical changes MUST be versioned, reviewable and activated at deterministic heights/epochs after a public timelock. An upgrade proposal MUST include specification diff, code diff, security analysis, economic impact, compatibility/migration plan, activation condition and rollback/recovery procedure.

Governance MUST NOT permit arbitrary state edits, retroactive transaction validity changes, discretionary minting or unilateral seizure. Emergency powers, if any, must be narrowly scoped, time-limited, auditable and independently controlled; exact authority and thresholds remain TBD. A node MUST reject unsupported consensus-critical protocol versions rather than guessing.

## 12. Data availability, networking and Bitcoin anchoring

Initial data availability is full on-chain transaction data: a block is not valid/finalizable unless the full bounded block data is available to the validating node. A commitment without retrievable data is insufficient for initial protocol validity.

Networking MUST use a versioned handshake binding chain ID, protocol version, node identity and supported capabilities. Messages require maximum lengths, bounded queues, request deadlines, quotas, peer-level resource accounting and deterministic validation independent of peer ordering. Peer reputation is local operational state and cannot affect consensus validity.

Bitcoin anchoring is optional and asynchronous. An anchor may commit to APEX chain ID, height, block ID, complete state root, validator-set hash and previous anchor. Inclusion and confirmation status must be tracked separately and reorgs handled. APEX consensus MUST continue safely when Bitcoin is unavailable; anchoring does not create APEX finality or prove correct execution.

## 13. Resource and failure semantics

Every consensus-critical parser and operation MUST define maximum input size, collection count, nesting depth, execution gas, memory/resource budget and timeout behavior. Resource exhaustion MUST cause deterministic rejection or safe local backpressure, never divergent partial state.

A finalized block's application commit MUST be idempotently recoverable. Ambiguous storage acknowledgements require reconciliation against durable block identity, height and state version before retry. Nodes MUST NOT sign a new conflicting consensus message while signer state or finalized application state is ambiguous.

Disk corruption, unsupported schema, inconsistent validator state, invalid commit certificate or unknown fork MUST fail closed and trigger a diagnosable halt/recovery procedure rather than silently resetting state.

## 14. Required conformance artifacts

Before G0 can pass, the repository MUST contain:
- canonical encoding vectors for genesis, transaction, block, receipt, validator and consensus evidence;
- state transition vectors covering success, revert, fees, issuance, burn, staking and validator changes;
- supply conservation vectors and economic simulation reports;
- cross-language decode/hash/signature verification vectors;
- fork activation and upgrade compatibility vectors;
- adversarial malformed-input corpus and fuzz harnesses;
- deterministic replay fixtures with expected receipts, validator set and complete state root;
- an explicit table of all consensus-critical constants and activation heights;
- independent review sign-off for protocol-critical decisions.

## 15. Decision register

| Decision | Current disposition | Evidence required to close |
|---|---|---|
| Rust-first hybrid architecture | Selected | Existing ADRs; revalidate at system integration |
| EVM-compatible execution | Target selected, implementation open | Pinned revm/fork ADR and differential conformance |
| Production address/account model | 20-byte EVM address target | EVM integration and cross-language vectors |
| Chain ID representation/allocation | TBD | EIP-155 compatibility, collision and domain review |
| General transaction/block/receipt codec | TBD | Canonical codec comparison, fuzzing and vectors |
| Transaction signature algorithm | TBD | EVM compatibility, security review and vectors |
| Consensus key/signature algorithm | TBD | Selected engine compatibility and audit evidence |
| BFT engine and exact revision | TBD | Dependency, MSRV, license, fault and security review |
| Complete state commitment | TBD | Full state model, tree ADR and proof vectors |
| Storage engine and atomic cross-subsystem protocol | Prototype only | Crash/power-loss and signer/application recovery evidence |
| Monetary numerical parameters | TBD | Economic simulations and security analysis |
| Validator set and staking parameters | TBD | Attack-cost/concentration simulations |
| Governance thresholds and emergency powers | TBD | Capture analysis and constitutional review |
| Bitcoin anchor policy | Optional; parameters TBD | Operational/reorg/fee analysis |

## 16. Ratification and status

This document is a completeness baseline, not a frozen consensus specification. The G0 gate remains BLOCKED until the TBD decisions are closed, normative test vectors exist, and independent technical review approves the result. Implementation MUST NOT silently turn TBDs into protocol constants.

**Release decision remains NO-GO.** This specification does not make the repository a functioning node, testnet or production protocol.
