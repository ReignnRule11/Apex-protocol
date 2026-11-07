# APEX Protocol

[![Rust CI](https://github.com/ReignnRule11/Apex-protocol/actions/workflows/ci.yml/badge.svg)](https://github.com/ReignnRule11/Apex-protocol/actions/workflows/ci.yml)

**APEX** is a research-stage Layer-1 blockchain infrastructure project. This repository contains protocol drafts and a small Rust foundation; it is not a functioning blockchain, public testnet, or mainnet.

## Design direction

- Independent proof-of-stake BFT consensus, with explicit safety and liveness assumptions.
- Rust implementation and deterministic state-transition rules.
- EVM-compatible execution as a planned capability, subject to a pinned revision and conformance testing.
- Full transaction-data availability in the initial design.
- Optional Bitcoin checkpoint anchoring for external commitment evidence. Anchoring is not APEX consensus and does not transfer Bitcoin security to APEX.
- Hybrid monetary-policy direction: proposed hard supply ceiling, scheduled issuance, base-fee burn and proposer priority fees. Numerical parameters remain undecided.

## Current engineering status

**Phase 8 — versioned snapshots and atomic application boundary (prototype).** The `apex-consensus` crate plans ordered transactions against an in-memory overlay. The new `apex-state` crate binds those plans to a parent `StateVersion` and defines a compare-and-swap application contract. Its reference in-memory adapter validates the version, ordered unique account keys and all before-state values before any write, then applies all account updates and advances the version once. Stale/replayed plans and failed preconditions leave state unchanged. The adapter is process-local and non-durable.

These are research prototypes, not a functioning blockchain. There is no signature verification, canonical transaction identity, EVM execution, actual gas measurement, fee settlement, receipts, state roots, production database adapter, consensus engine, validator networking, commit-certificate verification, block production or consensus finality.

Malachite remains a preferred research candidate, not an integrated or production-approved dependency. Its upstream workspace declares Rust 1.88, while APEX declares Rust 1.85; the MSRV decision remains open. The `apex-codec` crate deliberately fails closed because canonical wire encoding has not been selected.

## Repository structure

```text
Apex-protocol/
├── .github/
│   └── workflows/ci.yml
├── crates/
│   ├── apex-codec/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── apex-consensus/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── apex-state/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                 # Versioned snapshots + in-memory reference adapter
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
│   │   └── ADR-0011-versioned-state-application.md
│   ├── protocol/
│   │   ├── bitcoin-anchoring.md
│   │   ├── consensus-adapter-contract.md
│   │   ├── consensus-engine-evaluation.md
│   │   ├── consensus-integration-spike.md
│   │   ├── consensus.md
│   │   ├── execution.md
│   │   ├── governance.md
│   │   ├── monetary-policy.md
│   │   ├── protocol-spec-v0.1.md
│   │   └── transaction-admission.md
│   └── threat-model/overview.md
├── genesis/devnet.json
├── .gitignore
├── Cargo.toml
├── CONTRIBUTING.md
├── README.md
├── SECURITY.md
└── rust-toolchain.toml
```

This tree describes committed paths and explicitly labels placeholders; it does not imply every subsystem is operational.

## Engineering principles

1. Safety before throughput claims.
2. Deterministic execution and canonical encoding.
3. Explicit trust assumptions and adversarial testing.
4. No custom consensus algorithm without formal review and compelling evidence.
5. No mainnet, decentralization or security claims without independent validators, audits and operational evidence.

## CI

GitHub Actions runs formatting, Clippy and workspace tests. A green CI run validates only the code included in that run; it is not a protocol security audit.

## Risk and token status

No token is being offered. No investment or return is promised. No production network is available. Do not use this software to custody funds or secure production assets.

## License

No license has been selected yet. All rights are reserved unless and until a license is added.
