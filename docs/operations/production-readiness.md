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
- Integrated production BFT consensus, authenticated validator transitions and application-gated finality; the isolated Ed25519 certificate verifier is experimental and disconnected.
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



**20G1 — pinned Malachite EngineBuilder adapter (complete; PR #43 merged and post-merge main CI passed).** The isolated `experiments/malachite-spike` workspace now contains a generic, typed adapter that invokes the real v0.8.2 `EngineBuilder::build` API with upstream WAL, network, consensus, sync and request contexts. The adapter is compile-checked against the exact upstream commit and does not add Malachite to the APEX production workspace. It is not an engine runtime test: no engine is started, no socket or validator key is created, no APEX application event loop is attached, and no finality certificate is authenticated or committed. G2 remains BLOCKED. The next subphase must supply an APEX-specific application and signer/network boundary, then run actual multi-validator integration tests before any production adoption. See the Phase 20G1 record and ADR-0045.

**20G2A — canonical consensus signing preimage (complete; PR #44 merged and post-merge main CI passed).** Adds an experimental fixed-width preimage codec binding chain ID, protocol version, height, round, step, signer identity, validator-set hash and block hash. It does not implement signing or verification, bind signer identity to a public key, verify certificates, or ratify the profile. G2 remains BLOCKED.\n\nProduction adoption remains blocked until the complete suite, recovery/signing safety and independent review pass.

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


**20G2B — authenticated consensus signatures and commit certificates (implementation submitted for exact-head CI).** Adds pinned ed25519-dalek 3.0.0 strict Ed25519 verification, public-key-derived validator IDs, canonical authenticated validator-set hashing, exact-height authenticated validator-set schedules, and precommit certificate verification against the scheduled set. The verifier rejects malformed/weak keys, unknown/duplicate signers, mismatched chain/protocol/height/set/block context, invalid signatures and insufficient quorum. It does not authenticate validator-set governance updates, define a canonical certificate wire decoder, connect to Malachite or gate application commits. G2 remains BLOCKED pending runtime integration, signer-key custody/recovery, fault testing and independent review. See the Phase 20G2B record and ADR-0047.


**20G2B — authenticated consensus signatures and commit certificates (implementation submitted for exact-head CI).** Adds pinned ed25519-dalek 3.0.0 strict Ed25519 verification, public-key-derived validator IDs, canonical authenticated validator-set hashing, exact-height authenticated validator-set schedules, and precommit certificate verification against the scheduled set and caller-expected chain/protocol/height/round/block context. The verifier rejects malformed/weak keys, unknown/duplicate signers, mismatched context/set hashes, invalid signatures and insufficient quorum. It does not authenticate validator-set governance updates, define a canonical certificate wire decoder, connect to Malachite or gate application commits. G2 remains BLOCKED pending runtime integration, signer-key custody/recovery, fault testing and independent review. See the Phase 20G2B record and ADR-0047.


### Phase 20G2C1 — Malachite-native finality bridge contract (submitted)

The pinned v0.8.2 source audit establishes that Malachite emits its own `CommitCertificate<Ctx>` (height, round, value ID and native signatures), and exposes `VerifierExt::verify_commit_certificate` for native precommit signature and voting-power verification. The channel API separates early `Decided` from `Finalized`; the latter requires the application to respond with a next-height or restart decision.

This is not cryptographically interchangeable with the APEX Phase 20G2A/20G2B profile. APEX's verifier checks signatures over `APEX-CONSENSUS-MSG-V1`; Malachite uses the context's native signing scheme. The integration must verify the native certificate natively, then independently bind its value to canonical APEX block bytes, recomputed block identity, chain ID, protocol version, parent and APEX validator-set context. The application must not advance on `Decided`, and must acknowledge `Finalized` only after successful durable commit.

20G2C1 adds the reviewed contract and ADR only. No runtime bridge, canonical block codec, commit atomicity, signer rollback protection or BFT safety/liveness evidence is implemented. G2 remains BLOCKED. The next implementation subphase is 20G2C2, a compile-checked adapter against the exact pinned native APIs.


## Current verified Phase 20 status (supersedes earlier submitted/pending labels)

- **20G2B through 20G2C4 are merged.** Phase 20G2C4 merge commit: `7f2945c629f2eea41779f0f210038f7f8094a5c7`; post-merge main workflow #42421014413 passed all four jobs.
- **20G2C5 is submitted.** It classifies finality commit outcomes and prevents unsafe same-height restart after ambiguous or conflicting commits. Its contract and tests remain isolated research work.
- **G2 remains BLOCKED.** There is no concrete atomic application-state/finality journal, authenticated validator governance, signer anti-rollback, durable restart reconciliation, live multi-validator runtime or independent safety/liveness review.


### Phase 20G2C2 — isolated native certificate adapter (submitted)

Adds a generic helper in `experiments/malachite-spike` that checks native certificate height, round and value ID; requires an explicit native value-ID-to-canonical-APEX-identity binding; and calls the exact pinned Malachite `VerifierExt::verify_commit_certificate` with the native validator set and explicit thresholds. The direct signing dependency is isolated from the production Cargo workspace. The adapter produces an opaque verified token only after those checks succeed. Exact-head CI is the compile/test gate; no local build is claimed.

This is not a running `Channels` event loop, does not process `AppMsg::Finalized`, does not gate or persist an application commit, and does not prove the native/APEX validator address-key-power mapping or authenticated validator governance. Canonical value decoding, bounded value storage, parent linkage derivation, durable consensus/application/signer atomicity, restart recovery and signer anti-rollback remain blockers. **G2 remains BLOCKED; overall release remains NO-GO.** See the Phase 20G2C2 record and ADR-0049.


### Phase 20G2C3 — Malachite Channels finality event-loop gate (submitted)

The isolated spike now contains a gate over the real `Channels::consensus` receiver. It distinguishes `Decided` from `Finalized`: Decided is verified and staged before acknowledgement but cannot advance height; Finalized is reverified and can send `Next::Start` only after the application commit callback succeeds and the next-height increment is checked. Verification, binding, commit or overflow failure takes a restart/stall path. Non-finality messages are delegated to the application's normal handler. The direct engine dependency remains isolated in `experiments/malachite-spike`.

The gate is an ordering primitive, not proof of a correct application implementation. Canonical value decoding, bounded storage, authenticated validator-set provenance, durable cross-system atomicity, signer anti-rollback and restart recovery remain unimplemented. Only a height-overflow unit test is currently included; deterministic channel-level tests are the next subphase. G2 remains BLOCKED; release remains NO-GO. See Phase 20G2C3 and ADR-0050.


### Phase 20G2C4 — deterministic channel finality tests (submitted)

The isolated Malachite spike adds deterministic tests over the actual pinned `AppMsg` and `Next` types and the Tokio consensus receiver. Evidence covers Decided verification/staging acknowledgement order with no commit/advance, dropped acknowledgements on verification/staging errors, Finalized commit-before-start ordering, restart behavior on verification/identity-height/commit failures, reply drop when restart parameters are unavailable, checked-height overflow and channel closure after queued processing. Test fixtures do not provide production certificate verification or storage. No live engine or production validator is started. These tests validate gate control flow only; they do not prove cryptographic correctness, canonical APEX value binding, durable atomic commit, restart recovery or BFT safety/liveness. G2 remains BLOCKED; release remains NO-GO. See Phase 20G2C4 and ADR-0051.


### Phase 20G2C5 — idempotent finality commit contract (submitted)

The isolated Malachite gate now derives a stable commit key from the opaque verified APEX identity and distinguishes a proven non-commit from an ambiguous storage outcome or conflicting finalized identity. Only a proven non-commit may request same-height restart; ambiguous/conflicting outcomes drop the Finalized reply and stall pending durable reconciliation. The application contract requires atomic persistence of application state plus the key and idempotent same-key retries. Tests cover classification and reply behavior only. No concrete database transaction, durable recovery journal, power-loss test or live engine exists. G2 remains BLOCKED; release remains NO-GO. See Phase 20G2C5 and ADR-0052.


## Phase 20G2C6 — isolated durable finality journal prototype

The isolated Malachite spike now contains a redb 4.5.0 finality journal experiment. It atomically stores the APEX identity/payload row and chain-tip row in one database transaction, enforces idempotent exact retries, rejects conflicting same-height identities/payloads, checks contiguous height and parent linkage, and supports reopen/readback. This is not integrated with `FinalityApplication`, the consensus WAL or signer journal. No process-crash/power-loss, multi-process, backup rollback or cross-subsystem atomicity proof exists. G2 and G4 remain BLOCKED; release remains NO-GO.


### Phase 20G2C7 — finality journal process-termination recovery tests (merged; post-merge main CI passed)

The isolated redb finality journal now has child-process tests for abrupt exit at two application transaction boundaries. Exit after both rows are staged but before commit must reopen with no payload or chain tip and permit the same-height commit. Exit after commit returns but before acknowledgement must reopen with the complete payload and tip and accept an exact retry as `AlreadyCommitted`. The child uses `std::process::exit` to bypass Rust destructors; CI exercises a process termination/reopen sequence on its runner.

This does not simulate a power cut, kernel panic, torn write, disk-full/I/O failure, storage-controller cache loss or filesystem/device matrix. It does not establish multi-process writer safety, backup rollback protection, cross-subsystem atomicity or production recovery. The journal remains disconnected from the Malachite WAL and signer journal. **G2 and G4 remain BLOCKED; release remains NO-GO.** See the Phase 20G2C7 record and ADR-0054.

### Phase 20G2C8 — cross-subsystem recovery protocol design (proposed for review)

Defines a proposed recovery authority model across the Malachite consensus WAL, signer reservation journal, finality journal and materialized application state. It refines ADR-0052's atomic application-state/commit-key callback contract; that existing contract remains active until a reviewed integration decision supersedes it. The signer journal is monotonic and domain-aware; Malachite's native signing bytes must not be conflated with APEX's Phase 20G2A signing preimage. WAL records are replay evidence only; the finality journal is the ordered APEX commit ledger after native certificate verification; application state is a rebuildable projection. Startup must remain locked until all stores reconcile. The design specifies commit ordering, replay rules, a fail-closed decision matrix and validation gates. It does not establish that the pinned Malachite WAL exposes the required evidence, implement adapters, create cross-database atomicity or provide backup rollback protection. **G2 and G4 remain BLOCKED; release remains NO-GO.** See the Phase 20G2C8 record and ADR-0055.


### Phase 20G2C9 — pinned Malachite WAL recovery capability audit (proposed for review)

Audits the exact Malachite v0.8.2 commit. Default WAL open repairs/truncates structurally incomplete tails; engine fetch truncates at raw read errors; consensus startup resets WAL for restart/non-validator paths, and a requested-height/sequence mismatch resets the WAL. Persisted entries are a limited engine replay-input subset, not APEX finalized-block history. Append and sync are separate, and the signer is invoked before WAL append. Default WAL startup is therefore not approved as the first operation on APEX's sole recovery evidence. Immutable preflight, read-only scanning, guarded reset behavior and a signer-wrapper proof are required before integration. No runtime code changed. **G2/G4 remain BLOCKED; release remains NO-GO.** See Phase 20G2C9 and ADR-0056.
