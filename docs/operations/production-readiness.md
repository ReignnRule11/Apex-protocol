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
- Production BFT consensus, authenticated certificates, authenticated validator transitions and finality.
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

**20A — candidate revalidation (complete; merged and post-merge CI verified).** Malachite v0.8.2 is pinned for research at upstream commit `a453b32155f4ba9f959724279b52f601bae92e09` (tag `v0.8.2`, Apache-2.0). Its Rust 1.88 MSRV is compatible with APEX Rust 1.91.0. Upstream labels it alpha and not externally audited. No dependency is added and no production engine is selected.

**20B — isolated dependency/API spike (complete; PR #38 merged; exact-head and post-merge main CI passed).** A separate Cargo workspace at `experiments/malachite-spike` pins `arc-malachitebft-app-channel` to Malachite v0.8.2's exact upstream commit. CI compiles and tests the exported builder API without adding the engine to APEX's production workspace. The job generates and uploads its lockfile, resolved dependency tree and package license metadata. The exact-head PR workflow and post-merge main workflow both passed the APEX workspace, isolated API smoke tests, and the pinned upstream unit suite. Dependency artifact: 415 package records, 397 registry, 17 pinned Malachite Git packages and one path package; all have license metadata, but legal compatibility and RustSec review remain outstanding. No signer, validator or live network is enabled.

**20C — APEX application/engine boundary (complete; PR #39 merged; post-merge main CI passed).** `apex-consensus::application_boundary` introduces engine-neutral proposal identity/linkage checks, bounded payload/proof envelopes, a deterministic application validation callback, an engine-specific finality-verifier contract and a commit handoff gated on successful verification of the exact validated identity. It does not implement a production certificate verifier, canonical block codec, full execution, durable replay protection or atomic consensus/application storage.

**20D — durable signer safety (complete; PR #40 merged; post-merge main CI passed).** Adds typed chain/protocol/height/round/step/signer coordinates, exact-message reservations and a redb journal that commits before returning. Conflicting bytes at one coordinate fail closed. No signature suite, key custody, signing implementation or backup rollback protection is implemented.

**20E — validator-set lifecycle primitives (merged; post-merge CI passed).** Adds deterministic sorted validator sets, nonzero identity/power checks, duplicate rejection, checked total power, strict quorum arithmetic over unique supplied signer identities, exact activation-height schedule selection and an unverified equivocation-claim envelope. It does not authenticate validator-set updates, verify signatures or commit certificates, establish finality, or implement staking/slashing. See the Phase 20E record and ADR-0043.

**20F — multi-process adversarial harness (merged; post-merge main CI passed).** Adds four independent OS worker processes, isolated append-only journals, deterministic fault scheduling for partition/drop/delay/reordering, conflicting-record rejection, restart recovery from each worker's journal, replacement-process replay, deterministic replay comparison and per-message payload bounds. The fixture exercises process and journal behavior only: it does not run BFT, authenticate votes, establish quorum certificates, simulate real network sockets, or prove consensus safety/liveness. CI runs the isolated harness's format, Clippy and process integration tests. G2 and G3 remain BLOCKED; the next required work is engine-backed consensus integration and authenticated network fault testing.



**20G1 — pinned Malachite EngineBuilder adapter (implementation submitted for exact-head CI).** The isolated `experiments/malachite-spike` workspace now contains a generic, typed adapter that invokes the real v0.8.2 `EngineBuilder::build` API with upstream WAL, network, consensus, sync and request contexts. The adapter is compile-checked against the exact upstream commit and does not add Malachite to the APEX production workspace. It is not an engine runtime test: no engine is started, no socket or validator key is created, no APEX application event loop is attached, and no finality certificate is authenticated or committed. G2 remains BLOCKED. The next subphase must supply an APEX-specific application and signer/network boundary, then run actual multi-validator integration tests before any production adoption. See the Phase 20G1 record and ADR-0045.

Production adoption remains blocked until the complete suite, recovery/signing safety and independent review pass.

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
