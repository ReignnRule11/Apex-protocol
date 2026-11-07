# APEX Protocol

[![Rust CI](https://github.com/ReignnRule11/Apex-protocol/actions/workflows/ci.yml/badge.svg)](https://github.com/ReignnRule11/Apex-protocol/actions/workflows/ci.yml)

**APEX** is a research-stage Layer-1 blockchain infrastructure project. This repository contains protocol drafts and Rust infrastructure prototypes; it is not a functioning blockchain, public testnet, or mainnet.

## Design direction

- Independent proof-of-stake BFT consensus, with explicit safety and liveness assumptions.
- Rust implementation and deterministic state-transition rules.
- EVM-compatible execution as a planned capability, subject to a pinned revision and conformance testing.
- Full transaction-data availability in the initial design.
- Optional Bitcoin checkpoint anchoring for external commitment evidence. Anchoring is not APEX consensus and does not transfer Bitcoin security to APEX.
- Hybrid monetary-policy direction: proposed hard supply ceiling, scheduled issuance, base-fee burn and proposer priority fees. Numerical parameters remain undecided.

## Current engineering status

**Phase 10 — durable state storage spike.** A separate `apex-state-redb` crate now implements a prototype persistent StateStore adapter using redb 4.5.0. Account updates and the next state version are committed in one immediate-durability database transaction. Snapshots are read consistently, stale plans are rejected, and existing initialized data is not overwritten by later bootstrap input. The adapter has close/reopen, atomicity and integrity-check tests.

The exact redb 4.5.0 release declares Rust 1.90, so the workspace MSRV and CI toolchain are explicitly raised from 1.85 to 1.90.0. redb is pinned exactly and isolated in its own adapter crate. This is a contained evaluation, not production approval; multi-process modes are not enabled.

The account record is a private 24-byte little-endian storage schema (u64 nonce + u128 balance). It is not protocol encoding or a consensus commitment. Canonical encoding remains undecided and the codec continues to fail closed. No state root or Merkle proof is implemented.

These remain research prototypes, not a functioning blockchain. There is no signature verification, canonical transaction identity, EVM execution, actual gas measurement, fee settlement, receipts, state roots, production consensus engine, validator networking, commit-certificate verification, block production or consensus finality.

Malachite remains a consensus research candidate, not an integrated or production-approved dependency. Consensus adoption requires separate dependency, fault-injection, adversarial testing and security review.

## Repository structure

```text
Apex-protocol/
├── .github/
│   └── workflows/
│       └── ci.yml
├── crates/
│   ├── apex-codec/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── apex-consensus/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── apex-state/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                 # Backend-neutral StateStore contract + memory adapter
│   ├── apex-state-redb/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                 # redb persistent adapter prototype
│   ├── apex-node/
│   │   ├── Cargo.toml
│   │   └── src/main.rs                # Placeholder CLI; not a running node
│   └── apex-types/
│       ├── Cargo.toml
│       └── src/lib.rs
├── docs/
│   ├── architecture/adr/
│   │   ├── ADR-0001-layered-architecture.md
│   │   ├── ADR-0002-rust.md
│   │   ├── ADR-0003-bitcoin-anchors.md
│   │   ├── ADR-0004-consensus-engine.md
│   │   ├── ADR-0005-canonical-encoding.md
│   │   ├── ADR-0006-cryptographic-domains.md
│   │   ├── ADR-0007-transaction-admission.md
│   │   ├── ADR-0008-state-aware-admission.md
│   │   ├── ADR-0009-transition-planning.md
│   │   ├── ADR-0010-block-transition-planning.md
│   │   ├── ADR-0011-versioned-state-application.md
│   │   ├── ADR-0012-storage-and-commitment-boundaries.md
│   │   └── ADR-0013-redb-state-adapter.md
│   ├── protocol/
│   │   ├── bitcoin-anchoring.md
│   │   ├── consensus-adapter-contract.md
│   │   ├── consensus-engine-evaluation.md
│   │   ├── consensus-integration-spike.md
│   │   ├── consensus.md
│   │   ├── execution.md
│   │   ├── governance.md
│   │   ├── monetary-policy.md
│   │   ├── phase-9-storage-and-commitments.md
│   │   ├── phase-10-redb-storage-spike.md
│   │   ├── protocol-spec-v0.1.md
│   │   ├── state-commitments.md
│   │   ├── storage-architecture.md
│   │   └── transaction-admission.md
│   └── threat-model/
│       └── overview.md
├── genesis/devnet.json
├── .gitignore
├── Cargo.toml
├── CONTRIBUTING.md
├── README.md
├── SECURITY.md
└── rust-toolchain.toml
```

The tree lists committed paths; comments identify placeholders and do not imply operational subsystems.

## Engineering principles

1. Safety before throughput claims.
2. Deterministic execution and canonical encoding.
3. Separate local storage integrity from protocol state commitments.
4. Explicit trust assumptions and adversarial testing.
5. No custom consensus algorithm without formal review and compelling evidence.
6. No mainnet, decentralization or security claims without independent validators, audits and operational evidence.

## CI

GitHub Actions runs formatting, Clippy and workspace tests on Rust 1.90.0. A green CI run validates only the code included in that run; it is not a protocol security audit.

## Risk and token status

No token is being offered. No investment or return is promised. No production network is available. Do not use this software to custody funds or secure production assets.

## License

No license has been selected yet. All rights are reserved unless and until a license is added.
