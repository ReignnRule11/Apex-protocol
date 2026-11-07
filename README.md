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

**Phase 19A — revm compatibility spike (merged).** Exact revm 43.0.3 is pinned with default features disabled and std enabled explicitly. Rust 1.90.0 incompatibility was reproduced; workspace metadata, toolchain and CI were aligned to Rust 1.91.0. PR #21 and post-merge CI passed.

**Phase 19B — minimal EVM execution harness (merged).** A test-only fixture deploys a contract into revm's in-memory database, commits creation, calls it and verifies a storage value. PR #22 and post-merge CI passed.

**Phase 19C — full-width EVM state model (merged).** Adds 20-byte addresses, 256-bit words for balances/storage, bytecode references and bounded immutable snapshots. PR #23 and post-merge CI passed.

**Phase 19D — revm database adapter (merged).** Implements revm 43.0.3 Database over the immutable snapshot, preserving full-width values and failing closed on missing or malformed code. It is read-only. PR #24 and post-merge CI passed.

**Phase 19E — atomic full-width EVM state deltas (merged).** Adds parent-version-bound before/after account, code and storage updates with strict ordering checks and all-or-nothing in-memory snapshot replacement. PR #25 and post-merge CI passed.

**Phase 19F — persistent full-width EVM state prototype (merged).** PR #26 adds a separate `apex-evm-redb` crate. It stores full-width account, code, storage and block-hash records under versioned keys; one redb transaction writes the new snapshot and advances the current-version pointer. Historical versions are retained. PR CI #42388264492 and post-merge CI #42388315877 passed. This remains a single-process/single-writer prototype, with no garbage collection, canonical state encoding, state root or cross-subsystem consensus commit.

**Phase 19G — persistent-state corruption and recovery validation (merged).** PR #28 merged at `c653b073e446620b5f6ede38c7d41c7bde2c72dd`; PR CI #42389694673 and post-merge CI #42389851008 passed. Adds controlled reopen-corruption tests for incomplete metadata, unsupported schema versions and malformed current-version account records.

**Phase 19H — commit ambiguity and orphaned-version recovery tests (merged).** PR #29 merged at `5062fafeab885ec0b603bd8f9784d3d80e6ea129`; PR CI #42390234287 and post-merge CI #42390288716 passed. Adds test-only pre-commit failure, lost-acknowledgement-after-commit and orphaned-future-version tests. These validate reopen reconciliation semantics but do not simulate OS crash, power loss or device I/O faults.

**Phase 19I — revm state-diff extraction (merged).** PR #31 merged at `1682c0d81502c4d5905f9f97c466af8e71383519`; PR CI #42391544055 and post-merge CI #42391594192 passed. The adapter converts revm `EvmState` into deterministic, parent-version-bound APEX account/code/storage deltas, validates bytecode hashes and storage originals, and rejects fork-sensitive account deletion. Adds `crates/apex-evm/src/state_diff.rs`, which converts revm `EvmState` into deterministic APEX account/code/storage deltas bound to the exact parent `StateVersion`. It checks supplied bytecode hashes and collisions, validates revm original storage values against the parent, and leaves publication to the existing atomic apply boundary. Selfdestruct/account deletion fails closed until fork-specific semantics are ratified. Tests exercise constructed revm state values; they are not official execution fixtures or a complete transaction-to-state integration. The caller must pass only successful execution state; the API does not yet accept a success-gated transaction result type.

**Phase 19J — success-gated revm outcomes (PR #32; CI #42391906701 passed, review pending).** Adds `extract_successful_revm_result_delta` over revm's paired result/state value. Only `ExecutionResult::Success` is eligible for extraction; `Revert` and `Halt` are rejected before state translation. This does not authenticate the transaction or settle fees.

**Phase 19 — remaining work.** Explicit validated block context, fork selection/activation, canonical transaction authentication, fee settlement, receipts, complete state commitments, official execution fixtures and differential testing remain outstanding.

**Phase 18 — cryptographic identity and transaction authentication (in progress).** Adds ChainId256 as a 32-byte big-endian in-memory primitive and proposes EIP-2718 type 0x02 (EIP-1559) as the initial transaction profile. No numeric chain ID, canonical RLP decoder or signature verifier is implemented.

**Phase 17 — protocol completeness baseline (merged; draft).** Adds Protocol Specification v0.2, a complete state-domain model, transaction/block/consensus contracts, monetary invariants, genesis/upgrade requirements and an explicit unresolved decision register. G0 remains blocked; this is not a ratified production protocol.

**Phases 14–16 — account encoding and Merkle proofs (merged).** These are account-only research prototypes. The 38-byte account codec and SHA-256 Merkle root cover only AccountId(u64), nonce(u64) and balance(u128), not the complete EVM state model.

The workspace also contains transaction admission/planning and a redb persistence prototype for the historical narrow state model. That crate is separate from `apex-evm-redb`; neither adapter is a production database selection. There is no production BFT engine, validator networking, commit-certificate verification, block production or finality.

The workspace and CI are pinned to Rust 1.91.0, the minimum declared by the selected revm candidate graph. No EVM fork has been ratified.

## Production release status

**NO-GO — not production-ready.**

The production-readiness register defines G0–G7 and Phases 17–26: docs/operations/production-readiness.md. A green CI run validates only the code in that run; it is not protocol assurance or a security audit.

## Repository structure

    Apex-protocol/
    ├── .github/workflows/ci.yml
    ├── crates/
    │   ├── apex-codec/
    │   ├── apex-commitments/
    │   ├── apex-consensus/
    │   ├── apex-evm/                    # revm candidate, harness, full-width state, state-diff adapter and adapters
    │   ├── apex-evm-redb/               # versioned full-width EVM persistence prototype
    │   ├── apex-state/                  # legacy AccountId/u128 state prototype
    │   ├── apex-state-redb/             # persistence prototype for legacy state only
    │   ├── apex-node/                   # Placeholder CLI; not a running node
    │   └── apex-types/                  # ChainId256 primitive; legacy ChainId is prototype-only
    ├── docs/
    │   ├── architecture/adr/            # ADR-0001 through ADR-0035
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
