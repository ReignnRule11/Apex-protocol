# APEX Production Readiness and Release Gates

**Current release decision: NO-GO for production use, real funds, public validator operation, or mainnet.**

This document is the authoritative engineering release register. It separates implemented prototypes from protocol completeness and defines the evidence required to move from research to a public testnet and then a production network. A green CI run is necessary but never sufficient.

## 1. Current baseline

APEX is a research-stage Layer-1 infrastructure repository. The merged work currently provides:
- Rust workspace and basic CI (format, Clippy, workspace tests).
- Structural proposal-header checks, transaction admission and pure account transition planning.
- Versioned state interfaces, an in-memory adapter, and a redb persistence prototype with migration, recovery and failure-boundary tests.
- Canonical encoding for the current account record and an account-only SHA-256 Merkle root.
- Account membership and ordered-neighbor non-membership proof prototypes.

These are useful foundations, not a functioning blockchain. The following critical systems are not implemented or production-selected:
- A complete, versioned protocol specification and complete state model.
- Authenticated transaction envelopes, chain-bound signatures, canonical transaction IDs, replay protection and key/address derivation.
- Deterministic EVM execution, intrinsic/actual gas, receipts, fee settlement, contract storage/code and fork rules.
- Production BFT consensus integration, proposer/vote authentication, commit-certificate verification, validator-set transitions and finality.
- Authenticated P2P networking, peer discovery, rate limits, DoS controls and network protocol versioning.
- Atomic coordination of consensus state, signer anti-double-sign state, block metadata and application state.
- Complete state commitment, proofs for all committed state domains, and state sync.
- RPC/API, transaction pool, block production, genesis tooling and upgrade machinery.
- Validator key custody, monitoring, alerting, backup/restore, incident response and operational runbooks.
- Independent security audit, public adversarial testnet evidence and a legally distributable license.

Do not represent APEX as testnet-ready or mainnet-ready until the corresponding gates below have objective evidence.

## 2. Release gates

| Gate | Required evidence | Current status |
|---|---|---|
| G0 — Protocol freeze | Ratified protocol versioning, state model, signing domains, transaction/block formats, monetary parameters and upgrade rules | BLOCKED |
| G1 — Deterministic application | Authenticated transactions, canonical IDs, complete execution, gas/fees/receipts, deterministic replay and full state commitment | BLOCKED |
| G2 — Consensus correctness | Pinned engine, authenticated votes/certificates, validator transitions, anti-double-sign, restart recovery and formal safety/liveness assumptions | BLOCKED |
| G3 — Network correctness | Versioned P2P, peer authentication policy, bounded parsing, DoS controls, partitions/recovery and multi-node integration | BLOCKED |
| G4 — Durable operation | Atomic cross-subsystem commits, crash/power-loss testing, migrations, backup/restore, state sync and upgrade rollback | BLOCKED |
| G5 — Security assurance | Threat model closure, fuzzing, property tests, fault injection, dependency review, independent audits and remediation verification | BLOCKED |
| G6 — Public testnet | Reproducible node binaries, public genesis, independent validators, observability, incident drills, documented reset/upgrade policy and sustained soak evidence | NOT STARTED |
| G7 — Mainnet authorization | All previous gates closed, economic simulations, legal/license review, governance and emergency controls, independent sign-off and explicit go/no-go record | NOT STARTED |

No gate can be passed by documentation alone. Every implementation claim must link to code, tests, reproducible artifacts, testnet evidence or an independent report appropriate to that claim.

## 3. Ordered implementation program

### Phase 17 — Protocol completeness and specification freeze
- Define complete state domains: accounts, contract code/storage, validator set, consensus parameters, receipts, fee accounting and protocol metadata.
- Ratify transaction, block, receipt and state encoding; chain ID and signing domains; transaction identity; nonce/replay rules; gas schedule; fee accounting; fork/version activation; genesis; validator lifecycle; upgrade compatibility.
- Resolve all numerical monetary and validator parameters through explicit simulations and ADRs.
- Publish normative invariants and cross-language test vectors.
- Exit: reviewed protocol specification and no unresolved consensus-critical ambiguity.

### Phase 18 — Cryptographic identity and transaction authentication
- Select and pin reviewed signature/key algorithms through ADRs; define address derivation and domain separation.
- Implement strict canonical transaction envelopes, signature verification, transaction hashes, replay protection and bounded decoding.
- Add known-answer vectors, malformed-input tests, fuzz targets and key-rotation/recovery policy.
- Exit: independent implementations agree on all vectors and invalid transactions fail closed.

### Phase 19 — Deterministic execution and fee accounting
- Select and pin the EVM implementation and supported fork through ADRs.
- Implement account/code/storage state, transaction execution, intrinsic and actual gas, receipts, contract creation, precompiles and deterministic error semantics.
- Implement base-fee calculation, burn, proposer tips, issuance and supply conservation as deterministic state transitions.
- Differential-test against upstream EVM vectors; fuzz state transitions and replay blocks across architectures.
- Exit: deterministic replay and full state-root equality across independent nodes.

### Phase 20 — Consensus engine integration
- Complete a version/MSRV/license/dependency/security evaluation and select the exact BFT engine revision.
- Implement the APEX adapter, authenticated proposal/vote/certificate checks, validator-set changes, evidence handling, timeouts and finality.
- Persist signer state with durable anti-double-sign guarantees; test restart and backup/restore.
- Run multi-node Byzantine, partition, equivocation, delayed-message and crash-recovery tests.
- Exit: documented safety/liveness assumptions and independently reviewed consensus integration.

### Phase 21 — P2P networking and node services
- Implement versioned libp2p transport, peer identity policy, discovery, handshake, gossip/sync, peer scoring and resource limits.
- Add transaction pool, block proposal pipeline, block sync and bounded request/response protocols.
- Validate malformed traffic, Sybil/DoS scenarios, partitions and network upgrades.
- Exit: multi-node network operates under sustained adversarial and degraded-network tests.

### Phase 22 — Atomic persistence, state sync and upgrades
- Define atomic commit protocol spanning finalized block metadata, application state, consensus engine state and signer anti-double-sign records.
- Add crash/power-loss, disk-full, corruption, interrupted migration and multi-process tests on supported platforms/filesystems.
- Implement authenticated state sync, snapshot format, restore verification and upgrade/rollback procedures.
- Exit: recovery always converges to a valid finalized state and cannot cause double-signing or replay.

### Phase 23 — RPC, observability and operator experience
- Implement authenticated/admin RPC boundaries, public read APIs, rate limits and versioning.
- Add metrics, structured logs, traces, health/readiness endpoints, alerts and dashboards.
- Publish validator installation, key custody, backup, monitoring, incident response, upgrade and disaster-recovery runbooks.
- Exit: an independent operator can deploy, monitor, upgrade and recover a node using documented procedures.

### Phase 24 — Security hardening and independent review
- Complete threat-model review and dependency/license/SBOM review.
- Add fuzzing, property-based testing, deterministic cross-platform replay, fault injection and performance/resource-exhaustion testing.
- Commission independent protocol, consensus, cryptography, execution, networking and operational audits; publish scope and remediation status.
- Exit: all critical/high findings remediated and independently retested; residual risks explicitly accepted.

### Phase 25 — Public testnet and release candidate
- Publish reproducible signed binaries, source provenance, genesis parameters and a public incident channel.
- Operate with independent validators and no privileged single-operator dependency.
- Run sustained soak tests, validator churn, partition/recovery, upgrade drills, state sync and incident exercises.
- Exit: predefined reliability, decentralization, recovery and security metrics met over an announced observation window.

### Phase 26 — Mainnet go/no-go
- Confirm G0–G6 evidence, legal/license clearance, token/economic disclosures, governance, emergency procedures and independent sign-off.
- Freeze release artifacts and genesis; verify reproducible builds and checksums.
- Publish explicit launch authorization and rollback/incident plan.
- Exit: signed release decision by accountable maintainers and independent reviewers. Otherwise remain testnet or research-stage.

## 4. Non-negotiable engineering controls

1. **No fake completion:** no stubs, mock consensus, simulated signatures, placeholder execution, or fabricated network behavior presented as production functionality.
2. **Fail closed:** malformed, unauthenticated, stale, ambiguous or unavailable critical state must reject or halt safely.
3. **Determinism:** consensus-critical behavior must not depend on wall-clock time, randomness, host locale, unordered iteration or network side effects.
4. **Cryptographic discipline:** algorithms and encodings are versioned and pinned; changes require migration and compatibility analysis.
5. **Reproducibility:** pinned toolchains/dependencies, lockfile, SBOM, provenance, reproducible release artifacts and signed checksums.
6. **No unsafe launch:** no real funds, token sale, production custody or security claims before G7 authorization.
7. **No single-node decentralization claim:** public testnet evidence must demonstrate independent operators and documented validator concentration.
8. **No audit theater:** internal tests and automated scanners are not independent audits.

## 5. Repository and governance prerequisites

- Select an OSI-compatible project license and contribution/IP policy before inviting external contributions or distributing binaries as open source. The current repository is explicitly UNLICENSED.
- Define maintainers, security response ownership, disclosure process and release signing authority.
- Add branch protection and required CI checks; protect release tags and signing credentials.
- Establish a versioned changelog, deprecation policy, migration policy and release artifact retention.
- Maintain a risk register with owner, severity, mitigation, evidence and accepted residual risk.

## 6. Current decision

**NO-GO.** Phases 0–16 provide research specifications and bounded Rust prototypes. They do not implement a production blockchain. Continue with Phase 17 only after this readiness register is reviewed; preserve the existing hybrid architecture, Rust-first implementation and infrastructure priority, but do not freeze unspecified protocol parameters by assumption.
