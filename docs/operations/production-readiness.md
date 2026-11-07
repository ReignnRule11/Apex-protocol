# APEX Production Readiness and Release Gates

**Current release decision: NO-GO for production use, real funds, public validator operation, or mainnet.**

This document is the authoritative engineering release register. A green CI run is necessary but never sufficient.

## 1. Current baseline

APEX remains a research-stage Layer-1. Phases 0–17 provide Rust infrastructure and bounded account/storage prototypes plus a protocol v0.2 completeness draft; they do not constitute a functioning blockchain. Phase 18 adds a 256-bit chain-ID value type and proposes a type-2 EIP-1559 transaction profile; no numeric chain ID, canonical transaction decoder or signature verification is implemented.

Phase 19A pins revm 43.0.3 and Rust 1.91.0. Phase 19B contains a minimal test-only execution harness. Phases 19C–19E add a full-width EVM state model, revm Database read adapter and atomic in-memory delta application. Phase 19F adds a separate full-width redb persistence prototype. Phases 19G–19H add selected reopen-corruption and commit-ambiguity tests, not real OS/device fault simulations. Phase 19I adds revm state-diff extraction; 19J gates extraction on successful execution outcomes; 19K adds the explicit APEX block-context model; 19L adds a bounded, explicit mapping into revm 43.0.3 configuration and block fields; 19M wires that mapped environment and the immutable APEX snapshot into the actual revm builder. The builder does not call Context::mainnet() or select SpecId::LATEST. The Phase 19L adapter rejects unsupported fork profiles and any 256-bit chain ID/base fee that would be truncated to revm's u64 fields. Its Ethereum mainnet gas table is provisional, not ratified APEX policy.

Critical systems not implemented or production-selected:
- Ratified full protocol specification and complete state model.
- Production chain identity/allocation and canonical transaction/block/receipt codecs.
- Authenticated transaction envelopes, chain-bound signatures, canonical transaction IDs and replay protection.
- Complete deterministic EVM transaction pipeline, gas/fee settlement, receipts and full state commitment.
- Authenticated block/genesis validation feeding the execution context and ratified APEX fork/gas/precompile semantics.
- Full transaction-to-persistent-state integration and atomic consensus/signer/application commit.
- Production BFT consensus, authenticated certificates, validator transitions and finality.
- Authenticated P2P networking, transaction pool, block production and sync.
- RPC, genesis tooling, upgrades, operator runbooks, observability and independent audits.

## 2. Release gates

| Gate | Required evidence | Current status |
|---|---|---|
| G0 — Protocol freeze | Ratified protocol versioning, full state model, signing domains, canonical transaction/block formats, numerical monetary/validator parameters, upgrade rules and cross-language vectors | BLOCKED — v0.2 draft; Phase 18 chain-ID/envelope and Phase 19 fork/gas mappings remain proposals |
| G1 — Deterministic application | Authenticated transactions, canonical IDs, complete execution, gas/fees/receipts, deterministic replay and full state commitment | BLOCKED — staged revm adapters exist; no authenticated full transaction pipeline, fee settlement, receipts or complete root |
| G2 — Consensus correctness | Pinned engine, authenticated votes/certificates, validator transitions, anti-double-sign, restart recovery and safety/liveness evidence | BLOCKED |
| G3 — Network correctness | Versioned P2P, bounded parsing, DoS controls, partitions/recovery and multi-node integration | BLOCKED |
| G4 — Durable operation | Atomic cross-subsystem commits, crash/power-loss testing, migrations, backup/restore, state sync and rollback | BLOCKED — storage adapters remain prototypes |
| G5 — Security assurance | Threat model closure, fuzzing, property tests, fault injection, dependency review, independent audits and remediation | BLOCKED |
| G6 — Public testnet | Reproducible binaries, public genesis, independent validators, observability, incident drills and sustained soak evidence | NOT STARTED |
| G7 — Mainnet authorization | All prior gates, economic/legal/governance review, independent sign-off and explicit go/no-go | NOT STARTED |

## 3. Ordered implementation program

### Phase 17 — Protocol completeness baseline
Merged draft specification v0.2, state-domain model and unresolved decision register. G0 remains blocked until ratification and evidence.

### Phase 18 — Cryptographic identity and transaction authentication
In progress. Separate EVM transaction, consensus signer and P2P peer identity roles. Chain ID and type-2 envelope proposals are not ratified; no production signature verification is implemented.

### Phase 19 — Deterministic execution and fee accounting
In progress. revm 43.0.3 is pinned under Rust 1.91.0. Phases 19A–19L progressively add the harness, full-width state model, Database/state-diff adapters, in-memory/persistent state prototypes, success gating, explicit block context and a bounded revm environment mapping. Phase 19M constructs the EVM engine; Phase 19N maps caller-supplied, already-authenticated type-2 fields into TxEnv but does not implement authentication or execute a transaction. Phase 19L is not a full execution pipeline: the chain-ID/base-fee u64 boundary, fork semantics, blob inputs, authenticated transactions, fee settlement, receipts and complete commitments remain unresolved. See Phase 19 plan, phase records 19K/19L/19M/19N and ADR-0035/0036/0037/0038.

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

**NO-GO.** Protocol v0.2 is a review draft and Phase 18/19 contain proposals and adapters, not a ratified production protocol. No numeric chain ID, transaction codec, BFT signature algorithm, APEX EVM fork/gas policy or production key lifecycle is ratified.
