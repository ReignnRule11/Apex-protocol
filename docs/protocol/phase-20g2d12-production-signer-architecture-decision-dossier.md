# Phase 20G2D12 — production signer architecture decision dossier

**Status:** Decision preparation; no custody or runtime option selected.  
**Base main:** `010e90a12f6763c2a382381951f7fa30554d5a9f`.  
**Related:** Phase 20G2D7–20G2D11; ADR-0067.  
**Release impact:** G2/G4 remain BLOCKED; production remains NO-GO.

## Objective

Convert the production-signer absence finding into a reviewable architecture decision package. Establish the options, security properties, evidence and acceptance criteria that must be resolved before APEX implements or enables a production signing path.

This phase does not choose a backend on behalf of the protocol owners. It does not create a signer, generate or handle keys, add cryptographic dependencies, alter consensus bytes, or enable validator startup.

## Decisions that must be made

### 1. Runtime ownership

The architecture owner must select and document exactly where signing requests originate and where authorization is enforced.

| Candidate boundary | Required evidence | Principal risks to resolve |
|---|---|---|
| In-process signer owned by validator runtime | Actual validator executable, signer construction site, complete call graph to all key operations | Process compromise exposes signer authority; hidden direct-key paths; lifecycle coupling |
| Local sidecar / host agent | Authenticated local IPC contract, process identity, peer authorization, restart and availability behavior | Socket/IPC spoofing, confused deputy, local privilege escalation, split-brain lifecycle |
| Remote signer service | Service identity, authenticated transport, request authorization, deployment boundary, quorum/availability model | Network partition, replay, request substitution, service compromise, latency and outage behavior |
| HSM/KMS-backed adapter | Specific supported device/service, API semantics, access policy, audit evidence, backup and recovery model | Provider-specific semantics, authorization drift, key availability, ambiguous operation outcomes |

These candidates are not mutually exclusive at every layer: a validator may call a local adapter backed by an HSM or remote service. The decision must identify the complete request path and the actual authority boundary, not just name the storage product.

### 2. Key custody and backend

The architecture decision must select one concrete custody model for the initial production deployment and state whether other models are explicitly out of scope.

At minimum, evaluate:
- private-key generation and enrollment;
- whether private key material can ever leave its protected boundary;
- signer authentication and per-validator authorization;
- access-policy administration and separation of duties;
- backup, restore, disaster recovery and loss-of-key behavior;
- rotation, revocation and validator-set transitions;
- auditability and tamper evidence;
- provider outage, throttling and ambiguous timeout behavior;
- compromise containment and emergency disablement;
- deployment, operating cost and operational ownership.

A backend's ability to sign bytes is not evidence that it meets APEX's consensus-safety or operational requirements. No software, HSM, KMS, remote, threshold or MPC option is preselected by this dossier.

### 3. Cryptographic and identity profile

The protocol owners must ratify, rather than infer:
- signature algorithm and exact suite/version;
- private/public key formats and canonical encodings;
- validator identity derivation and public-key validation;
- signature encoding and verification behavior;
- domain separation for each signing operation;
- compatibility and migration policy for existing or future validator keys.

The selected profile must match the consensus verifier and the pinned engine's native signing contract. A dependency already present in Cargo manifests is not a protocol decision.

### 4. Native signing request contract

For every operation, the implementation proposal must provide a source-level trace from typed request through canonical byte construction to the final backend invocation.

| Operation | Required contract evidence |
|---|---|
| Vote | Exact native bytes, chain/protocol identity, height, round, step, signer and domain |
| Proposal | Exact native bytes, chain/protocol identity, height, round, step, signer and domain |
| Vote extension | Exact native bytes plus scope/value binding; distinct domain and replay semantics |
| Validator proof | Exact proof bytes and peer/public-key binding; pre-engine lifecycle and authorization |

The APEX Phase 20G2A preimage is not a substitute for Malachite-native signing bytes. Any transformation or envelope must be specified and verified against the pinned engine version. The context used for reservation must be derived from trusted application state, not caller-controlled metadata.

### 5. Reservation and failure semantics

The selected design must prove that:
1. the exact bytes that will reach the cryptographic backend are determined;
2. the typed APEX coordinate and signer/domain context are validated;
3. the durable reservation is committed before the backend can be invoked;
4. an identical retry is handled idempotently;
5. different bytes at an occupied coordinate fail closed;
6. storage errors, uncertain commits and backend timeouts cannot silently authorize a second distinct key operation;
7. replayed historical signatures are distinguished from fresh key use;
8. signature hand-off occurs only after the backend result is validated.

The current reservation store is a prototype and is not yet connected to any production key path. Its existing contract must not be represented as an enforced control until the application call path proves it.

### 6. Recovery and rollback

A production design must identify the independently protected monotonic anchor, its administrative trust boundary, and the recovery procedure. It must cover journal restoration, anchor disagreement, stale backups, generation rollback, signer rotation, and operator authorization.

A valid checkpoint or restored journal alone does not prove completeness or freshness. Validator signing must remain disabled until cross-store reconciliation and anti-rollback checks succeed.

## Required decision record

Before implementation begins, the architecture owner must publish an accepted ADR or equivalent signed-off decision containing:

- production validator runtime and exact signer ownership boundary;
- selected custody/backend and rejected alternatives;
- threat model and trust boundaries;
- cryptographic/key/validator-identity profile;
- per-operation native byte traces;
- signer construction and complete key-use call graph;
- authorization and context-propagation contract;
- reservation-before-key-call proof and failure-state model;
- independent anti-rollback and recovery architecture;
- operational lifecycle, monitoring, incident response and ownership;
- test plan covering crash, concurrency, replay, timeout, rotation, rollback and recovery.

Each decision must cite implementation or provider evidence. “TBD”, a package dependency, a test signer, or a product name without an integration/security analysis does not satisfy the gate.

## Current decision state

| Decision | State |
|---|---|
| Validator runtime / signer owner | NOT SELECTED |
| Custody and backend | NOT SELECTED |
| Signature and key profile | NOT RATIFIED |
| Native byte and trusted context contract | NOT IMPLEMENTED / NOT PROVEN |
| Reservation-to-key enforcement | NOT IMPLEMENTED |
| Complete key-use coverage | NOT PROVEN |
| Independent anti-rollback anchor | NOT SELECTED |
| Recovery and operations ownership | NOT APPROVED |

No default is inferred. The absence of an explicit selection is a deliberate blocker, not permission to choose the easiest implementation.

## Phase exit criteria

- Architecture options and decision criteria are explicit.
- Runtime ownership and custody are treated as separate but linked decisions.
- Every native signing operation, including pre-engine validator proof, has a required evidence contract.
- Reservation, backend invocation, replay and recovery are distinct states.
- No backend, key, signature profile, or production signer is selected or implemented by assumption.
- No runtime, cryptographic, schema, WAL, custody or startup behavior changes.
- G2/G4 remain BLOCKED; production release remains NO-GO.

## Next permitted phase

After the protocol/security owners provide and review the concrete decisions above, create a separate implementation plan scoped to the selected backend and validator runtime. That plan must first close the exact-byte and complete-key-use call graph before any production key invocation is enabled.
