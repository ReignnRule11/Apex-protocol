# APEX Protocol

[![Rust CI](https://github.com/ReignnRule11/Apex-protocol/actions/workflows/ci.yml/badge.svg)](https://github.com/ReignnRule11/Apex-protocol/actions/workflows/ci.yml)

**APEX** is a research-stage Layer-1 blockchain infrastructure project. This repository contains protocol drafts and Rust infrastructure prototypes; it is not a functioning blockchain, public testnet, or mainnet.

## Design direction

- Independent proof-of-stake BFT consensus using a mature engine, subject to selection and integration gates.
- Rust implementation with a hybrid consensus/application architecture.
- EVM-compatible execution target, subject to a pinned revision and differential conformance testing.
- Full transaction-data availability in the initial design.
- Optional Bitcoin checkpoint anchoring for external commitment evidence only; it does not transfer Bitcoin security to APEX.
- Hybrid monetary-policy direction: hard supply ceiling, scheduled issuance, base-fee burn and proposer priority fees. Numerical parameters remain unapproved.

## Current engineering status

**Phase 19A–19B — revm compatibility and harness (merged).** revm 43.0.3 is pinned with default features disabled and std enabled explicitly under Rust 1.91.0. The test-only harness deploys and calls a contract in an in-memory database and verifies a storage value.

**Phase 19C–19E — full-width EVM state adapters (merged).** Adds 20-byte addresses, 256-bit balance/storage words, immutable bounded snapshots, a read-only revm Database adapter, deterministic state-diff extraction and atomic in-memory delta application.

**Phase 19F–19H — persistent EVM state prototype and selected recovery tests (merged).** Adds a separate `apex-evm-redb` crate with versioned account/code/storage/block-hash records and atomic current-version publication. Controlled tests cover selected reopen corruption and commit-ambiguity cases. This remains a single-process/single-writer prototype, not production durability.

**Phase 19I–19J — state-diff and outcome gating (merged).** revm state changes are translated to deterministic parent-version-bound deltas; revert and halt outcomes are rejected by the success-gated extraction API. This is not a complete authenticated transaction pipeline.

**Phase 19K — explicit APEX block context (merged).** Adds immutable block inputs and a validated fork activation schedule. It does not authenticate source blocks or ratify a fork.

**Phase 19L — explicit revm environment mapping (merged).** Requires an explicit matching APEX ForkId/revm SpecId binding; maps block context into revm 43.0.3 CfgEnv/BlockEnv; rejects chain-ID/base-fee values that would be truncated and unsupported fork profiles. The first adapter supports only post-Merge/pre-Cancun profiles and uses revm's mainnet gas table provisionally, not as ratified APEX gas policy.

**Phase 19M — explicit revm builder integration (merged).** Adds an APEX-owned builder that binds the immutable EVM snapshot and Phase 19L-mapped CfgEnv/BlockEnv into revm 43.0.3 without Context::mainnet() or SpecId::LATEST. This constructs an EVM instance; it does not yet authenticate or execute APEX transactions.

**Phase 19N — validated type-2 transaction environment (merged).** Adds a bounded adapter from caller-authenticated type-2 fields into revm TxEnv, with exact chain-ID binding and caller-supplied input/access-list bounds. It does not decode RLP, verify signatures, recover senders, perform state admission, execute transactions or settle fees.

**Phase 20A — consensus-engine candidate revalidation (merged and independently verified).** Pins Malachite v0.8.2 at exact upstream commit `a453b32155f4ba9f959724279b52f601bae92e09` for research only. Its Rust 1.88 MSRV is compatible with APEX Rust 1.91.0, but upstream labels it alpha and not externally audited. No engine dependency or production adoption is approved. See `docs/protocol/phase-20a-consensus-engine-revalidation.md` and ADR-0039.

**Phase 20B — isolated Malachite dependency/API spike (merged; exact-head and post-merge CI passed).** Adds a separate Cargo workspace under `experiments/malachite-spike`, pinned to the exact v0.8.2 upstream commit. It compiles only the public channel-builder API surface and runs a small API smoke test. The dedicated CI job generates a lockfile, checks/tests the spike, and uploads the resolved dependency tree and license metadata for review. No Malachite dependency is added to the production workspace. See `docs/protocol/phase-20b-malachite-isolated-api-spike.md` and ADR-0040.

**Phase 20C — engine-neutral application boundary (merged; exact-head and post-merge CI passed).** Adds bounded proposal and proof envelopes, chain/version/height/parent linkage, an application validation callback, a separate engine-specific finality-verification interface, and a commit handoff that is reachable only after exact-identity verification. No production verifier, canonical codec, durable replay protection or cross-system atomic commit is implemented. See `docs/protocol/phase-20c-application-engine-boundary.md` and ADR-0041.

**Phase 20D — durable consensus signer safety (merged; post-merge CI passed).** Adds typed signing coordinates, exact-message reservations and a redb journal that commits before returning a reservation. It does not select a signature suite, hold keys, sign messages, prevent backup rollback or authorize validator operation. See `docs/protocol/phase-20d-durable-signer-safety.md` and ADR-0042.

**Phase 20E — validator-set lifecycle primitives (merged; post-merge CI passed).** Adds deterministic validator-set ordering, identity/power validation, checked aggregate power, strict greater-than-two-thirds quorum accounting over unique supplied signer identities, exact-height schedule selection, and a typed equivocation-claim envelope. It does not authenticate validator updates, verify signatures/certificates, slash validators, or establish finality. See `docs/protocol/phase-20e-validator-lifecycle.md` and ADR-0043.

**Phase 20F — multi-process adversarial harness (merged; post-merge main CI passed).** Adds an isolated, std-only process harness with four independent OS worker processes and append-only per-process journals. Deterministic tests inject partition, dropped/delayed/reordered delivery, conflicting writes, process restart, replacement-node replay, deterministic replay and payload bounds. This is a replicated-log fault-injection fixture, not a BFT engine, authenticated network, validator implementation or finality test. See `docs/protocol/phase-20f-multiprocess-adversarial-harness.md` and ADR-0044.

**Phase 20G1 — pinned Malachite EngineBuilder adapter (merged; post-merge CI passed).** Extends the isolated Malachite v0.8.2 research workspace with a typed adapter that calls the real upstream `EngineBuilder` and requires the upstream WAL, network, consensus, sync and request actors/codecs. It is compiled against the exact pinned commit and remains outside the production Cargo workspace. This phase does not instantiate an engine, start validators, open sockets, create signing keys, run the APEX application loop, validate APEX certificates, or commit finalized application state. Those are subsequent integration gates. See `docs/protocol/phase-20g1-malachite-engine-builder-adapter.md` and ADR-0045.

**Phase 20G2A — canonical consensus signing preimage (merged; post-merge CI passed).** Adds a fixed-width, big-endian, domain-separated signing preimage binding chain ID, protocol version, height, round, step, signer identity, validator-set hash and block hash. Nil votes are permitted for prevote/precommit; nil proposals are rejected. This constructs bytes only: no signature algorithm, public-key binding, signature verification, certificate validation or ratified wire profile is provided. See docs/protocol/phase-20g2a-canonical-consensus-message.md and ADR-0046.\n\n**Phase 19 — remaining work.** Ratified fork/gas/precompile semantics, full-width chain ID/base-fee compatibility, blob context, authenticated type-2 transactions, fee settlement, receipts, complete state commitments, official execution fixtures and differential testing remain outstanding.

**Phase 18 — cryptographic identity and transaction authentication (in progress).** Adds ChainId256 as a 32-byte big-endian in-memory primitive and proposes EIP-2718 type 0x02 (EIP-1559) as the initial transaction profile. No numeric chain ID, canonical RLP decoder or signature verifier is implemented.

**Phase 17 — protocol completeness baseline (merged; draft).** Adds Protocol Specification v0.2, a complete state-domain model, transaction/block/consensus contracts, monetary invariants, genesis/upgrade requirements and an explicit unresolved decision register. G0 remains blocked; this is not a ratified production protocol.

**Phases 14–16 — account encoding and Merkle proofs (merged).** These are account-only research prototypes. The 38-byte account codec and SHA-256 Merkle root cover only AccountId(u64), nonce(u64) and balance(u128), not the complete EVM state model.

The workspace also contains transaction admission/planning and a separate redb persistence prototype for the historical narrow state model. Neither database adapter is a production selection. There is no integrated BFT engine, validator networking, application-gated finality, block production or production finality. Certificate verification exists only as an isolated experimental primitive.

## Production release status

**NO-GO — not production-ready.**

The production-readiness register defines G0–G7 and Phases 17–26: `docs/operations/production-readiness.md`. A green CI run validates only the code in that run; it is not protocol assurance or a security audit.

## Repository structure

    Apex-protocol/
    ├── .github/workflows/ci.yml
    ├── crates/
    │   ├── apex-codec/
    │   ├── apex-commitments/
    │   ├── apex-consensus/
    │   │   ├── src/consensus_message.rs     # Experimental domain-separated signing preimage\n    │   │   ├── src/application_boundary.rs  # Engine-neutral proposal/finality handoff
    │   │   └── src/validator_set.rs        # Deterministic validator-set/lifecycle primitives
    │   ├── apex-evm/
    │   │   ├── execution_context.rs
    │   │   ├── revm_environment.rs
    │   │   ├── revm_execution.rs
    │   │   ├── revm_transaction.rs
    │   │   ├── state_diff.rs
    │   │   └── lib.rs
    │   ├── apex-evm-redb/
    │   ├── apex-state/
    │   ├── apex-state-redb/
    │   ├── apex-signer-safety/          # Engine-neutral anti-double-sign policy
    │   ├── apex-signer-redb/             # Durable signing-intent journal prototype
    │   ├── apex-node/                   # Placeholder CLI; not a running node
    │   └── apex-types/
    ├── experiments/
    │   ├── malachite-spike/          # Pinned Malachite API + typed EngineBuilder adapter
    │   └── consensus-harness/       # Separate OS-process fault-injection fixture
    ├── docs/
    │   ├── architecture/adr/            # ADR-0001 through ADR-0047
    │   ├── protocol/                    # Specifications and phase records
    │   ├── operations/                  # Production readiness and release gates
    │   └── threat-model/
    ├── genesis/devnet.json
    ├── Cargo.toml
    ├── CONTRIBUTING.md
    ├── README.md
    ├── SECURITY.md
    └── rust-toolchain.toml

## Engineering principles

1. Safety before throughput claims.
2. Deterministic execution and canonical encoding.
3. Separate local storage integrity from protocol state commitments.
4. Explicit trust assumptions and adversarial testing.
5. No custom consensus algorithm without compelling evidence or review.
6. No mainnet, decentralization or security claims without independent validators, audits and operational evidence.

## Token and license status

No token is being offered. No investment or return is promised. No production network is available. Do not use this software to custody funds or secure production assets.

No license has been selected. All rights are reserved unless and until a license is added.
