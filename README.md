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

**Phase 5 — state-aware transaction-admission prototype.** The Rust `apex-consensus` crate contains tested proposal-header preflight, static admission and a minimal read-only state-aware admission helper. The latter checks account existence, exact nonce equality and balance sufficiency for value plus the maximum-fee upper bound using checked arithmetic. It relies on the caller to supply a stable view of the exact candidate parent state; snapshot/version binding is not enforced. Sender identity remains unauthenticated. No signature verification, canonical address/encoding, intrinsic gas, state mutation, fee settlement or EVM execution is implemented.

Phase 2's consensus integration documentation and compatibility updates are committed. Malachite remains the preferred research candidate, but it is not integrated or approved as a production dependency. Its upstream workspace declares Rust 1.88, while APEX declares Rust 1.85; the MSRV decision remains open. See the consensus evaluation, ADR-0004 and integration-spike plan.

The `apex-consensus` crate's voting-power helper, proposal-header preflight and static/state-aware admission checks are primitives only, not a BFT engine or full transaction validator. The `apex-codec` crate deliberately fails closed because a canonical wire encoding has not been selected. No validator networking, consensus state machine, commit-certificate verification, block production or transaction execution is implemented.

## Repository structure

```text
Apex-protocol/
├── .github/
│   └── workflows/
│       └── ci.yml                         # Format, Clippy and workspace tests
├── crates/
│   ├── apex-codec/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                     # Fail-closed codec boundary
│   ├── apex-consensus/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                     # Quorum + proposal/transaction admission
│   ├── apex-node/
│   │   ├── Cargo.toml
│   │   └── src/main.rs                    # Placeholder CLI; not a running node
│   └── apex-types/
│       ├── Cargo.toml
│       └── src/lib.rs                     # Shared protocol primitives
├── docs/
│   ├── architecture/
│   │   └── adr/
│   │       ├── ADR-0001-layered-architecture.md
│   │       ├── ADR-0002-rust.md
│   │       ├── ADR-0003-bitcoin-anchors.md
│   │       ├── ADR-0004-consensus-engine.md
│   │       ├── ADR-0005-canonical-encoding.md
│   │       ├── ADR-0006-cryptographic-domains.md
│   │       └── ADR-0007-transaction-admission.md
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
│   └── threat-model/
│       └── overview.md
├── genesis/
│   └── devnet.json                         # Template only; no approved parameters
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
4. No custom consensus algorithm without a formal review and compelling evidence.
5. No mainnet, decentralization or security claims without independent validators, audits and operational evidence.

## CI

GitHub Actions runs formatting, Clippy and workspace tests on pushes to main and pull requests. A green CI run validates only the code included in that run; it is not a protocol security audit.

## Risk and token status

No token is being offered. No investment or return is promised. No production network is available. Do not use this software to custody funds or secure production assets.

## License

No license has been selected yet. All rights are reserved unless and until a license is added.
