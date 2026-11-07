# Threat Model Overview

## Assets
Consensus safety/liveness; balances and contract state; validator keys; transaction ordering and fee accounting; peer availability; release integrity; checkpoint evidence; governance integrity.

## Adversaries
Byzantine validators, colluding stake holders, eclipse/Sybil peers, spam senders, malicious contracts, compromised RPC operators, stolen keys, malicious dependencies, governance attackers and infrastructure outages.

## Principal controls
- Consensus: mature protocol, explicit assumptions, adversarial simulation and independent review.
- Replay/signatures: chain- and version-bound signing domain, canonical encoding and test vectors.
- Execution: pinned EVM revision, deterministic host interface, differential tests and fuzzing.
- P2P: bounded decoding, quotas, peer scoring, connection limits and backpressure.
- Keys: separate key roles, secure storage guidance, rotation and recovery.
- Economics: supply invariants, staking/fee simulations and bounded governance.
- Anchors: confirmation tracking; anchoring never controls consensus liveness.
- Supply chain: locked dependencies, CI, provenance and review.

## Release gate
No public network handling value until consensus and execution are implemented, adversarial tests are reproducible, assumptions documented, independent security review complete, key operations rehearsed and incident procedures exist.
