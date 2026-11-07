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

**Phase 15 — Merkle account membership proofs (review branch).** Adds generation and independent verification of account inclusion proofs while preserving the exact Phase 14 root. Non-membership and consensus root activation remain deferred. See `docs/protocol/phase-15-merkle-membership-proofs.md`.

**Phase 14 — canonical account encoding and state commitment prototype (merged; implementation CI passed).** The 38-byte account codec and SHA-256 Merkle commitment are limited to the current AccountId/nonce/balance model; this is not the complete protocol state root.

**Phase 13 — storage lifecycle and schema migration (merged; post-merge CI green).** Schema 2 migrates 24-byte account records to 32-byte records in one redb transaction, preserves StateVersion, rejects malformed legacy records, and latches the adapter after commit errors.

**Phase 12 — storage concurrency and failure-boundary engineering (merged; CI green).** Includes concurrent-reader snapshot tests, serialized competing-writer tests, pre-apply fault injection, and a lost-commit-acknowledgement simulation. Injected faults are adapter-boundary simulations, not real disk-full or device-I/O injection.

The exact redb 4.5.0 release declares Rust 1.90, so the workspace MSRV and CI toolchain are explicitly pinned to 1.90.0. redb is isolated in its own adapter crate. This is a contained evaluation, not production approval.

The redb account record is a private 32-byte little-endian storage schema (u64 nonce + u128 balance + zero-reserved u64). It is not protocol encoding. Phase 14's separate 38-byte canonical account record and account-only SHA-256 Merkle root remain a prototype. Phase 15 adds membership proofs only; it does not add cryptographic non-membership proofs or consensus root activation.

These remain research prototypes, not a functioning blockchain. There is no signature verification, canonical transaction identity, EVM execution, actual gas measurement, fee settlement, receipts, complete protocol state roots, production consensus engine, validator networking, commit-certificate verification, block production or consensus finality.

Malachite remains a consensus research candidate, not an integrated or production-approved dependency. Consensus adoption requires separate dependency, fault-injection, adversarial testing and security review.

## Repository structure

```text
Apex-protocol/
├── .github/workflows/ci.yml
├── crates/
│   ├── apex-codec/                 # Canonical account-record v1 codec
│   ├── apex-commitments/           # Account root + membership proof prototype
│   ├── apex-consensus/
│   ├── apex-state/                 # Backend-neutral StateStore + memory adapter
│   ├── apex-state-redb/            # redb persistent adapter prototype
│   ├── apex-node/                  # Placeholder CLI; not a running node
│   └── apex-types/
├── docs/
│   ├── architecture/adr/           # ADR-0001 through ADR-0018
│   ├── protocol/                   # Specifications and phase records
│   └── threat-model/
├── genesis/devnet.json
├── Cargo.toml
├── CONTRIBUTING.md
├── README.md
├── SECURITY.md
└── rust-toolchain.toml
```

The tree is a high-level summary; inspect committed files for exact crate contents.

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
