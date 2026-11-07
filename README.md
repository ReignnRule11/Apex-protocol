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

**Phase 19 — Deterministic EVM execution (engine evaluation).** Revm is the preferred candidate for a compile/conformance spike; no engine dependency, release or fork is pinned. Osaka/Fusaka-era behavior is a candidate baseline, while Glamsterdam remains excluded pending finalization and ratification. See docs/protocol/phase-19-deterministic-execution.md and ADR-0024.

**Phase 18 — Cryptographic identity and transaction-authentication (in progress).** Adds ChainId256 as a 32-byte big-endian in-memory primitive, keeps transaction/consensus/P2P key roles separate, and proposes EIP-2718 type 0x02 (EIP-1559) as the initial transaction profile. No numeric chain ID, RLP decoder or signature verifier is implemented. See docs/protocol/phase-18-cryptographic-identity.md and ADR-0022/0023.

**Phase 17 — Protocol completeness baseline (merged; draft).** Adds Protocol Specification v0.2, a complete state-domain model, transaction/block/consensus contracts, monetary invariants, genesis/upgrade requirements and an explicit unresolved decision register. G0 remains blocked; this is not a ratified production protocol. See `docs/protocol/protocol-spec-v0.2.md` and `docs/protocol/phase-17-protocol-completeness.md`.

**Phase 16 — account non-membership proofs (merged; PR CI and post-merge CI green).** Ordered-neighbor absence proofs for empty, before-first, between-neighbor and after-last cases. Account-only research prototype. See `docs/protocol/phase-16-account-non-membership-proofs.md`.

**Phase 15 — account membership proofs (merged; CI green).** Inclusion proof generation and verification, preserving the Phase 14 root. Account-only research prototype.

**Phase 14 — canonical account encoding and state commitment prototype.** The 38-byte account codec and SHA-256 Merkle root cover only AccountId(u64), nonce(u64) and balance(u128). This is not the production EVM state model.

The workspace also contains transaction admission/planning and a redb persistence prototype. These are not authenticated transaction processing, EVM execution, a production database selection or a functioning blockchain. There is no production BFT engine, validator networking, commit-certificate verification, block production or finality.

The workspace and CI are pinned to Rust 1.90.0 due to the exact redb 4.5.0 dependency. redb remains an isolated research adapter, not production approval.

## Production release status

<badge color="danger">NO-GO — not production-ready</badge>

The production-readiness register defines G0–G7 and Phases 17–26: `docs/operations/production-readiness.md`. A green CI run validates only the code in that run; it is not protocol assurance or a security audit.

## Repository structure

```text
Apex-protocol/
├── .github/workflows/ci.yml
├── crates/
│   ├── apex-codec/
│   ├── apex-commitments/
│   ├── apex-consensus/
│   ├── apex-state/
│   ├── apex-state-redb/
│   ├── apex-node/                  # Placeholder CLI; not a running node
│   └── apex-types/                  # ChainId256 primitive; legacy ChainId is prototype-only
├── docs/
│   ├── architecture/adr/           # ADR-0001 through ADR-0023
│   ├── protocol/                   # Specifications and phase records
│   ├── operations/                 # Production readiness and release gates
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
