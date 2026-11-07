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

**Phase 19A — revm compatibility spike (merged).** Exact revm 43.0.3 is pinned with default features disabled and `std` enabled explicitly. Rust 1.90.0 incompatibility was reproduced; workspace metadata, toolchain and CI were aligned to Rust 1.91.0. PR #21 merged after green PR CI. The post-merge run is tracked separately. See `docs/protocol/phase-19-revm-compatibility-spike.md` and ADR-0025.

**Phase 19B — minimal EVM execution harness (submitted).** A test-only fixture deploys a contract into revm's in-memory database, commits creation, calls it and verifies a storage value. It also checks repeatability across fresh contexts. It is not an APEX state adapter or production block executor. See `docs/protocol/phase-19b-evm-execution-harness.md` and ADR-0026.

**Phase 19 — deterministic EVM execution (in progress).** The next work is full-width EVM state, explicit fork/block context, authenticated transactions, fee settlement, receipts and complete state commitments. See `docs/protocol/phase-19-deterministic-execution.md` and ADR-0024.

**Phase 18 — cryptographic identity and transaction authentication (in progress).** Adds ChainId256 as a 32-byte big-endian in-memory primitive and proposes EIP-2718 type 0x02 (EIP-1559) as the initial transaction profile. No numeric chain ID, canonical RLP decoder or signature verifier is implemented. See `docs/protocol/phase-18-cryptographic-identity.md` and ADR-0022/0023.

**Phase 17 — protocol completeness baseline (merged; draft).** Adds Protocol Specification v0.2, a complete state-domain model, transaction/block/consensus contracts, monetary invariants, genesis/upgrade requirements and an explicit unresolved decision register. G0 remains blocked; this is not a ratified production protocol.

**Phases 14–16 — account encoding and Merkle proofs (merged).** These are account-only research prototypes. The 38-byte account codec and SHA-256 Merkle root cover only AccountId(u64), nonce(u64) and balance(u128), not the complete EVM state model.

The workspace also contains transaction admission/planning and a redb persistence prototype. These are not authenticated transaction processing, EVM execution, a production database selection or a functioning blockchain. There is no production BFT engine, validator networking, commit-certificate verification, block production or finality.

The workspace and CI are pinned to Rust 1.91.0, the minimum declared by the selected revm candidate graph. redb remains an isolated research adapter, not production approval.

## Production release status

**NO-GO — not production-ready.**

The production-readiness register defines G0–G7 and Phases 17–26: `docs/operations/production-readiness.md`. A green CI run validates only the code in that run; it is not protocol assurance or a security audit.

## Repository structure

```text
Apex-protocol/
├── .github/workflows/ci.yml
├── crates/
│   ├── apex-codec/
│   ├── apex-commitments/
│   ├── apex-consensus/
│   ├── apex-evm/                    # revm 43.0.3 candidate + test-only execution harness
│   ├── apex-state/
│   ├── apex-state-redb/
│   ├── apex-node/                   # Placeholder CLI; not a running node
│   └── apex-types/                  # ChainId256 primitive; legacy ChainId is prototype-only
├── docs/
│   ├── architecture/adr/            # ADR-0001 through ADR-0026
│   ├── protocol/                    # Specifications and phase records
│   ├── operations/                  # Production readiness and release gates
│   └── threat-model/
├── genesis/devnet.json
├── Cargo.toml
├── CONTRIBUTING.md
├── README.md
├── SECURITY.md
└── rust-toolchain.toml
```

## Engineering principles

1. Safety before throughput claims.
2. Deterministic execution and canonical encoding.
3. Separate local storage integrity from protocol state commitments.
4. Explicit trust assumptions and adversarial testing.
5. No custom consensus algorithm without compelling evidence and review.
6. No mainnet, decentralization or security claims without independent validators, audits and operational evidence.

## Token and license status

No token is being offered. No investment or return is promised. No production network is available. Do not use this software to custody funds or secure production assets.

No license has been selected. All rights are reserved unless and until a license is added.
