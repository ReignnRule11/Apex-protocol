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

**Phase 19L — explicit revm environment mapping (in progress).** Requires an explicit matching APEX ForkId/revm SpecId binding; maps block context into revm 43.0.3 CfgEnv/BlockEnv; rejects chain-ID/base-fee values that would be truncated and unsupported fork profiles. The first adapter supports only post-Merge/pre-Cancun profiles and uses revm's mainnet gas table provisionally, not as ratified APEX gas policy.

**Phase 19 — remaining work.** Ratified fork/gas/precompile semantics, full-width chain ID/base-fee compatibility, blob context, authenticated type-2 transactions, fee settlement, receipts, complete state commitments, official execution fixtures and differential testing remain outstanding.

**Phase 18 — cryptographic identity and transaction authentication (in progress).** Adds ChainId256 as a 32-byte big-endian in-memory primitive and proposes EIP-2718 type 0x02 (EIP-1559) as the initial transaction profile. No numeric chain ID, canonical RLP decoder or signature verifier is implemented.

**Phase 17 — protocol completeness baseline (merged; draft).** Adds Protocol Specification v0.2, a complete state-domain model, transaction/block/consensus contracts, monetary invariants, genesis/upgrade requirements and an explicit unresolved decision register. G0 remains blocked; this is not a ratified production protocol.

**Phases 14–16 — account encoding and Merkle proofs (merged).** These are account-only research prototypes. The 38-byte account codec and SHA-256 Merkle root cover only AccountId(u64), nonce(u64) and balance(u128), not the complete EVM state model.

The workspace also contains transaction admission/planning and a separate redb persistence prototype for the historical narrow state model. Neither database adapter is a production selection. There is no production BFT engine, validator networking, commit-certificate verification, block production or finality.

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
    │   ├── apex-evm/
    │   │   ├── execution_context.rs
    │   │   ├── revm_environment.rs
    │   │   ├── state_diff.rs
    │   │   └── lib.rs
    │   ├── apex-evm-redb/
    │   ├── apex-state/
    │   ├── apex-state-redb/
    │   ├── apex-node/                   # Placeholder CLI; not a running node
    │   └── apex-types/
    ├── docs/
    │   ├── architecture/adr/            # ADR-0001 through ADR-0036
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
