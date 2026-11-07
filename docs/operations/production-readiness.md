# APEX Production Readiness and Release Gates

**Current release decision: NO-GO for production use, real funds, public validator operation, or mainnet.**

This document is the authoritative engineering release register. A green CI run is necessary but never sufficient.

## 1. Current baseline

APEX remains a research-stage Layer-1. Phases 0–17 provide Rust infrastructure and bounded account/storage prototypes plus a protocol v0.2 completeness draft; they do not constitute a functioning blockchain. Phase 18 adds a 256-bit chain-ID value type and proposes a type-2 EIP-1559 transaction profile; no numeric chain ID, canonical transaction decoder or signature verification is implemented.

Critical systems not implemented or production-selected:
- Ratified full protocol specification and complete state model.
- Production chain identity/allocation and canonical transaction/block/receipt codecs.
- Authenticated transaction envelopes, chain-bound signatures, canonical transaction IDs and replay protection.
- Deterministic EVM execution, gas, receipts, fee settlement and full state commitment.
- Production BFT consensus, authenticated certificates, validator transitions and finality.
- Authenticated P2P networking, transaction pool, block production and sync.
- Atomic coordination of consensus, signer anti-double-sign state, block metadata and application state.
- RPC, genesis tooling, upgrades, observability, operator runbooks and independent audits.

## 2. Release gates

| Gate | Required evidence | Current status |
|---|---|---|
| G0 — Protocol freeze | Ratified protocol versioning, full state model, signing domains, canonical transaction/block formats, numerical monetary/validator parameters, upgrade rules and cross-language vectors | BLOCKED — v0.2 draft; Phase 18 chain-ID/envelope proposals pending ratification |
| G1 — Deterministic application | Authenticated transactions, canonical IDs, complete execution, gas/fees/receipts, deterministic replay and full state commitment | BLOCKED — revm 43.0.3 compile-only candidate; Rust 1.90.0 MSRV failure recorded; 1.91.0 validation pending |
| G2 — Consensus correctness | Pinned engine, authenticated votes/certificates, validator transitions, anti-double-sign, restart recovery and safety/liveness evidence | BLOCKED |
| G3 — Network correctness | Versioned P2P, bounded parsing, DoS controls, partitions/recovery and multi-node integration | BLOCKED |
| G4 — Durable operation | Atomic cross-subsystem commits, crash/power-loss testing, migrations, backup/restore, state sync and rollback | BLOCKED |
| G5 — Security assurance | Threat model closure, fuzzing, property tests, fault injection, dependency review, independent audits and remediation | BLOCKED |
| G6 — Public testnet | Reproducible binaries, public genesis, independent validators, observability, incident drills and sustained soak evidence | NOT STARTED |
| G7 — Mainnet authorization | All prior gates, economic/legal/governance review, independent sign-off and explicit go/no-go | NOT STARTED |

## 3. Ordered implementation program

### Phase 17 — Protocol completeness baseline
Merged draft specification v0.2, state-domain model and unresolved decision register. G0 remains blocked until ratification and evidence.

### Phase 18 — Cryptographic identity and transaction authentication
In progress. Separate EVM transaction, consensus signer and P2P peer identity roles. Review the secp256k1/EVM transaction-authentication proposal; settle chain-ID representation and canonical envelope before implementing verification. Consensus signature selection remains dependent on the selected BFT engine. No production signature code is claimed.

### Phase 19 — Deterministic execution and fee accounting
In progress. revm 43.0.3 is isolated in `crates/apex-evm` with default features disabled and `std` enabled explicitly. The first CI run proved Rust 1.90.0 is below the dependency graph's declared 1.91.0 MSRV; the workspace/toolchain/CI pins are now raised to 1.91.0 and require fresh CI validation. This is not an execution adapter or engine approval. Dependency graph review, fork ratification, state adapter, official fixtures, differential tests, fee settlement and complete-state commitment remain open. See Phase 19A and ADR-0025.

### Phase 20 — Consensus engine integration
Select pinned BFT revision; implement authenticated proposals/votes/certificates, validator changes, evidence, durable anti-double-sign and multi-node Byzantine/recovery tests.

### Phase 21 — P2P networking and node services
Implement versioned libp2p, peer identity, discovery, gossip/sync, quotas, DoS controls, mempool and block pipeline.

### Phase 22 — Atomic persistence, state sync and upgrades
Coordinate finalized block, application state, consensus state and signer records atomically; test power-loss, corruption, multi-process races, snapshots and rollback.

### Phase 23 — RPC, observability and operator experience
Implement safe RPC, metrics, logs, traces, alerts, dashboards and validator/operator runbooks.

### Phase 24 — Security hardening and independent review
Complete threat/dependency reviews, fuzzing, property testing, fault injection, independent audits and verified remediation.

### Phase 25 — Public testnet and release candidate
Publish reproducible artifacts and genesis; onboard independent validators; sustain adversarial soak, upgrade and recovery drills.

### Phase 26 — Mainnet go/no-go
Require all previous gates, legal/license clearance, economic disclosures, governance/emergency controls, independent sign-off and a signed launch decision.

## 4. Non-negotiable controls

- No mocks or stubs presented as protocol functionality.
- Fail closed on malformed, unauthenticated, stale, unsupported or ambiguous state.
- Deterministic consensus-critical behavior; no host-time, random or unordered-iteration dependencies.
- Pinned toolchain/dependencies, lockfile, SBOM, reproducible artifacts and signed release provenance.
- No real funds, token sale, production custody or mainnet claims before G7.
- Automated CI is not an audit. Internal tests are not independent review.
- Resolve the repository license and contribution/IP policy before external distribution.

## 5. Current decision

**NO-GO.** Protocol v0.2 is a review draft and Phase 18 contains proposals. G0 remains BLOCKED. No numeric chain ID, transaction codec, BFT signature algorithm or production key lifecycle is ratified by this work.
