# ADR-0080 — Durable Lifecycle Checkpoint Store and Rollback Witness

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related phase:** 20G2D24
- **Supersedes:** None

## Context

Phase 20G2D23 defines authenticated sequential checkpoints but deliberately leaves storage abstract and uses only an in-memory test store. A production integration needs durable atomic local persistence and a way to detect restoration of stale local state.

## Decision

Add a redb-backed implementation of the checkpoint store contract. Store each network's sequence and checkpoint hash as a fixed-width record, and update it using a single write transaction guarded by exact expected-head comparison. Add an external rollback-witness interface and a fail-closed equality check at startup.

## Consequences

- Checkpoint heads survive ordinary process restarts.
- A stale concurrent writer cannot replace the current head within the redb transaction boundary.
- Local database integrity can be checked explicitly.
- Local persistence is not an anti-rollback root. A separately protected monotonic witness is required to detect stale database restoration or storage replacement.
- Recovery from local/witness divergence remains blocked pending a ratified recovery policy.

## Rejected shortcuts

- Claiming a durable database alone prevents rollback.
- Silently selecting local state over an independently trusted witness.
- Automatically advancing or resetting the witness during startup.
- Treating an in-memory test witness as a production security control.
