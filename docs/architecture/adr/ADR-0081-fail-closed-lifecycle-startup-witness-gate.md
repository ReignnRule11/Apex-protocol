# ADR-0081 — Fail-Closed Lifecycle Startup Witness Gate

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related phase:** 20G2D25
- **Supersedes:** None

## Context

Phase 20G2D24 introduced durable local lifecycle checkpoint storage and an external rollback-witness interface. The comparison helper was not yet a startup orchestration contract. A node could otherwise proceed without verifying every configured network, and treating two absent heads as equal would silently permit an uninitialized network.

## Decision

Provide a reusable startup gate that:

1. Requires at least one configured network.
2. Rejects zero and duplicate network IDs.
3. Reads both local and independently witnessed heads for every configured network.
4. Requires both heads to exist and match exactly.
5. Fails closed on local-store errors, witness errors, uninitialized state, or divergence.
6. Never mutates state or performs automatic recovery.

The caller must run this gate before consensus, signing, networking, or lifecycle-dependent services are started.

## Consequences

- Startup orchestration has a deterministic, testable fail-closed contract.
- Genesis/bootstrap requires a separate authorized workflow.
- The current node executable is still a placeholder; this ADR does not claim the gate is wired into a running node.
- The witness implementation remains deployment-specific. It must be independently protected; colocating another database on the same host does not create an independent trust domain.
- No production anti-rollback guarantee is claimed until a real independent provider, authenticated transport, credential management, operations, and disaster-recovery policy are implemented and tested.
