# APEX Protocol

**APEX** is a research-stage Layer-1 blockchain infrastructure project. This repository contains protocol drafts and a small Rust foundation; it is not a functioning blockchain, public testnet, or mainnet.

## Design direction

- Independent proof-of-stake BFT consensus, with explicit safety and liveness assumptions.
- Rust implementation and deterministic state-transition rules.
- EVM-compatible execution as a planned capability, subject to a pinned revision and conformance testing.
- Full transaction-data availability in the initial design.
- Optional Bitcoin checkpoint anchoring for external commitment evidence. Anchoring is not APEX consensus and does not transfer Bitcoin security to APEX.
- Hybrid monetary-policy direction: proposed hard supply ceiling, scheduled issuance, base-fee burn and proposer priority fees. Numerical parameters remain undecided.

## Current engineering status

**Phase 1 — protocol engineering.** The repository contains a draft protocol specification, threat model, architecture decision records, and initial Rust workspace primitives. No production consensus, transaction execution, P2P networking, validator operations, or Bitcoin anchoring is implemented.

Malachite is the preferred Rust-native BFT candidate for a contained prototype, subject to mandatory stability, fault-injection, recovery, resource-bound and independent security-review gates. CometBFT remains the reference/fallback comparator. See the consensus evaluation and ADR-0004.

The apex-consensus crate currently contains only a tested voting-power supermajority arithmetic helper. It is not a BFT engine. The apex-codec crate deliberately fails closed because a canonical wire encoding has not been selected.

## Repository map

- docs/protocol/ — protocol, consensus, execution, monetary policy, anchoring and governance.
- docs/architecture/adr/ — recorded architecture decisions and open decisions.
- docs/threat-model/ — security assumptions and release gates.
- crates/ — Rust workspace foundations.
- genesis/ — placeholder configurations; not launch parameters.

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
