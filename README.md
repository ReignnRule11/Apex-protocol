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

**Phase 2 — consensus integration spike (interface and compatibility evaluation).** Phase 1 established the protocol drafts, threat model, architecture decisions and Rust workspace primitives. The Phase 1 CI run for commit `aacba430cd66e2b003896b1c0826b299978ad26a` passed.

Malachite remains the preferred Rust-native candidate for a contained integration prototype, but it is not integrated or approved as a production dependency. Upstream currently identifies release 0.8.2 and Rust 1.88 as its workspace minimum, while APEX currently declares Rust 1.85. The upstream README also labels the software alpha and not externally audited. This compatibility and assurance gap must be resolved before adding it to the dependency graph. See the consensus evaluation, ADR-0004 and the integration-spike plan.

The `apex-consensus` crate currently contains only a tested voting-power supermajority arithmetic helper. It is not a BFT engine. The `apex-codec` crate deliberately fails closed because a canonical wire encoding has not been selected. No validator networking, consensus state machine, commit-certificate verification or block production is implemented.

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
