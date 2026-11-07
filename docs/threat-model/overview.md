# Threat Model Overview

## Assets
Consensus safety/liveness; balances and contract state; validator keys; transaction ordering and fee accounting; peer availability; release integrity; checkpoint evidence; governance integrity.

## Adversaries
Byzantine validators, colluding stake holders, eclipse/Sybil peers, spam senders, malicious contracts, compromised RPC operators, stolen keys, malicious dependencies, governance attackers and infrastructure outages.

## Principal controls
- Consensus: mature protocol, explicit assumptions, adversarial simulation and independent review. Malachite is currently an alpha research candidate, not an integrated or audited APEX component; its upstream MSRV (Rust 1.88) currently exceeds APEX's declared MSRV (Rust 1.85).
- Proposal preflight: compare chain ID, protocol version, sequential height and parent hash before deeper processing. This is a structural guard only, not cryptographic or consensus verification.
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

## Release gate
No public network handling value until consensus and execution are implemented, adversarial tests are reproducible, assumptions documented, independent security review complete, key operations rehearsed and incident procedures exist.
