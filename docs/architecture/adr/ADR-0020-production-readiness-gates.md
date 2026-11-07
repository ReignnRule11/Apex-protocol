# ADR-0020: Production Readiness Gates and Release Authorization

- **Status:** Accepted as release governance policy
- **Date:** 2026-11-07

## Context

APEX has accumulated research specifications and infrastructure prototypes, but it does not yet implement a functioning Layer-1 network. A green Rust CI run validates only the code in that run. It cannot establish protocol correctness, consensus safety, execution determinism, network resilience, operational recoverability or security assurance.

## Decision

1. APEX production readiness is governed by explicit gates G0–G7 in docs/operations/production-readiness.md.
2. The repository remains NO-GO for real funds, public validator operation with economic value, and mainnet until every required gate has evidence and an explicit authorization.
3. Implementation status must distinguish research prototype, integrated prototype, testnet-ready and production-approved.
4. Consensus, execution, networking, signer state and application persistence must be integrated and tested as one system; isolated crate tests cannot substitute for end-to-end evidence.
5. No custom consensus, cryptographic algorithm or protocol-critical wire format is considered approved merely because it compiles.
6. Independent review and remediation are required before production authorization.
7. The project license, contribution policy and release signing authority must be resolved before external distribution and mainnet release.

## Consequences

The production roadmap is staged and may require substantial redesign when full state and consensus semantics are specified. Release claims become evidence-based and can be blocked even when CI is green. This intentionally prioritizes safety and long-term maintainability over premature launch dates.
