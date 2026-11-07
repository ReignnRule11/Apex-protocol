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

**Phase 3 — engine-neutral consensus boundary.** The Rust `apex-consensus` crate now contains a tested preflight validator for proposal chain ID, protocol version, sequential height and parent-hash linkage. This is deliberately narrower than proposal validation: it does not authenticate proposers, validate transactions, verify BFT certificates, execute blocks or establish finality.

Phase 2's consensus integration documentation and compatibility updates are committed. Malachite remains the preferred research candidate, but it is not integrated or approved as a production dependency. Its upstream workspace declares Rust 1.88, while APEX declares Rust 1.85; the MSRV decision remains open. See the consensus evaluation, ADR-0004 and integration-spike plan.

The `apex-consensus` crate's voting-power helper and proposal-header preflight checks are primitives only, not a BFT engine. The `apex-codec` crate deliberately fails closed because a canonical wire encoding has not been selected. No validator networking, consensus state machine, commit-certificate verification, block production or transaction execution is implemented.

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
