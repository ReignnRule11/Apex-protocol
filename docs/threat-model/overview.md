# Threat Model Overview

## Assets
Consensus safety/liveness; balances and contract state; validator keys; transaction ordering and fee accounting; peer availability; release integrity; checkpoint evidence; governance integrity.

## Adversaries
Byzantine validators, colluding stake holders, eclipse/Sybil peers, spam senders, malicious contracts, compromised RPC operators, stolen keys, malicious dependencies, governance attackers and infrastructure outages.

## Principal controls
- Consensus: mature protocol, explicit assumptions, adversarial simulation and independent review. Malachite is an alpha research candidate, not an integrated or audited APEX component.
- Proposal preflight: compare chain ID, protocol version, sequential height and parent hash. Structural guard only, not cryptographic or consensus verification.
- Transaction admission: static envelope checks plus prototype account existence, nonce and affordability checks. Parent state must be stable and versioned. Sender identity is unauthenticated.
- Block planning: preserve transaction order, reject duplicate temporary IDs, enforce cumulative declared gas, merge writes through an overlay and fail the whole plan on any invalid transaction.
- State application: bind plans to a parent StateVersion, compare-and-swap the version, validate every before-state before writing, and commit all account changes plus version atomically.
- Persistent storage: `apex-state-redb` is a prototype adapter pinned to redb 4.5.0 and Rust 1.90.0. It uses immediate-durability transactions and one atomic account-plus-version commit. It is currently single-process/single-handle and has no process-kill/power-loss harness, disk-full injection, multi-process conformance, or production backup/restore workflow. Backend checksums are not state commitments.
- Corruption: malformed metadata, unsupported schema versions and malformed account records fail closed. Commit errors may be indeterminate; close/reopen and reconcile persisted version before retrying.
- State commitments: never trust database page checksums or physical storage roots as consensus commitments. Canonical bytes, deterministic ordering, domain-separated cryptographic hashes, and authenticated-tree semantics require a separate reviewed specification. No state root is implemented.
- Replay/signatures: chain- and version-bound signing domain, canonical encoding and test vectors.
- Execution: pinned EVM revision, deterministic host interface, differential tests and fuzzing.
- P2P: bounded decoding, quotas, peer scoring, connection limits and backpressure.
- Keys: separate key roles, secure storage guidance, rotation and recovery.
- Economics: supply invariants, staking/fee simulations and bounded governance.
- Anchors: confirmation tracking; anchoring never controls consensus liveness.
- Supply chain: locked dependencies, CI, provenance and review.

## Consensus-specific threats and required mitigations
- Engine/API drift: pin exact release and commit; review breaking changes and lockfile.
- Unsafe validator-set changes: test activation/exit boundaries, voting-power changes and validator identity binding.
- Equivocation and forged evidence: verify signatures and conflicting vote coordinates under a versioned domain; never slash on unverified or ambiguous evidence.
- Crash/restart and double-signing: durable signer state, monotonic signing safety, recovery tests and documented key backup/restore procedures.
- Network partitions and resource exhaustion: bounded queues, message sizes, timeouts, peer limits and fault injection.
- Application/consensus mismatch: validate deterministic state transitions before accepting finalized application state; halt safely on inconsistent or unavailable application state rather than inventing a result.
- Storage rollback/corruption: compare persisted head/version with committed block metadata; never accept a mixed or guessed state after recovery.
- Commitment divergence: cross-implementation test vectors and deterministic state-root calculation before consensus activation.

## Release gate
No public network handling value until consensus and execution are implemented, adversarial tests are reproducible, assumptions documented, independent security review complete, key operations rehearsed and incident procedures exist.
