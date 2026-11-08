# ADR-0076 — Deterministic Audit of the Validator Lifecycle Reference Ledger

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related phase:** 20G2D20
- **Supersedes:** None

## Context

Phase 20G2D19 introduced an executable in-memory lifecycle state machine with per-scope append-only event histories, a current-state index, and a request-id index. Those indexes are derived from the event history. Before any persistence design is selected, the reference implementation needs a deterministic way to detect divergence among the event log and its derived indexes.

Activation and exit confirmation records also carry protocol proofs checked by an injected verifier. An audit must not treat a previously accepted proof as permanently trusted without re-running the verifier during replay.

## Decision

Add a read-only `LifecycleLedger::audit` operation that:
1. replays each scope's event history from the default state;
2. validates scope partitioning, expected revisions and unique non-zero request IDs;
3. reapplies every event through the normal transition function;
4. re-verifies protocol transition proofs through the caller-supplied verifier;
5. compares each recorded resulting state with the replayed state;
6. compares the replayed final-state map with the ledger's current-state index; and
7. compares the replayed request map with the ledger's request-id index.

Any inconsistency returns a typed error. The operation does not mutate the ledger.

## Consequences

### Positive

- Makes history/index consistency executable and testable.
- Detects altered state snapshots, missing request-index entries and state-index divergence.
- Ensures protocol proofs are re-evaluated during audit.
- Establishes a reusable conformance check before a future storage adapter is considered.

### Negative and constraints

- Audit cost is linear in the total number of retained lifecycle records.
- Audit availability depends on the supplied verifier.
- The in-memory ledger remains non-durable and single-process.
- A successful audit is not authentication, provenance, anti-rollback protection, or proof that the source history was never rewritten.

## Rejected shortcuts

- Trusting the current-state index without replay.
- Checking only record counts or final revisions.
- Treating stored proof bytes as self-authenticating.
- Introducing a serialization format, database schema, governance authority or production verifier in this phase.

## Open decisions

Canonical persistence encoding, durable storage, authenticated checkpoints, independent monotonic anchoring, verifier implementation, governance authority and production lifecycle integration remain unselected.
