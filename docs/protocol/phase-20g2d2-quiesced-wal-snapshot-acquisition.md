# Phase 20G2D2 — quiesced, read-only WAL snapshot acquisition

**Status:** Implementation submitted for review; isolated research workspace only.  
**Base main:** `f60d86c3880ae9f867832048ce8294e0a3ce366c`.  
**Pinned upstream:** Malachite v0.8.2, commit `a453b32155f4ba9f959724279b52f601bae92e09`.

## Objective

Add a bounded, read-only file acquisition layer before the Phase 20G2D1 byte-slice scanner. The acquisition layer must not call Malachite's mutating WAL open, replay, reset, or startup paths.

## Implementation

Adds `experiments/malachite-spike/src/wal_snapshot.rs`.

- Requires an explicit caller attestation that all WAL writers have been stopped and the source is quiescent.
- Opens the supplied path read-only and reads through the opened file descriptor.
- Requires a caller-supplied maximum byte count; rejects a file already larger than the limit and detects reads that exceed it.
- Captures file metadata before and after the read. It rejects detectable changes in length/modification time and, on Unix, device/inode/change-time identity.
- Rejects a metadata/read-length mismatch.
- Computes SHA-256 over the exact bytes captured and exposes those bytes to the existing strict preflight scanner.
- Performs no lock acquisition, write, truncate, repair, sync, or Malachite WAL API call.
- Includes tests for byte/hash fidelity and source non-mutation, explicit size-limit rejection, and empty-snapshot handling.

## Critical assurance boundary

This is **not an atomic filesystem snapshot** and the quiescence token is a caller assertion, not an enforceable lock or cryptographic proof. Metadata comparisons detect common replacement and mutation races but cannot detect every same-metadata rewrite, hostile filesystem behavior, or a writer that violates the quiescence precondition. The operator must stop all writers and independently establish that the WAL cannot change for the entire acquisition. A storage-native snapshot or stronger platform-specific locking design remains a future option.

The hash identifies the acquired byte sequence; it does not authenticate its provenance, establish an anti-rollback anchor, prove backup durability, or bind it to the signer journal, finality journal, application commitment, chain ID, protocol version, or engine revision.

## Validation and non-goals

Deterministic tests are added in the isolated spike. Exact-head CI must still validate formatting, lint, compilation, scanner integration, and tests.

This phase does not decode compressed or Malachite engine payloads, reconcile cross-store state, create an authenticated recovery manifest, authorize reset, implement a custom WAL actor, wrap the signer, start the engine, or enable validators. The snapshot must not be treated as finality evidence or a production backup.

**Exit:** bounded read-only acquisition primitive and tests submitted. Next gates: exact-head CI, then authenticated recovery-manifest design and a guarded startup/reset policy. G2/G4 remain BLOCKED; production release remains NO-GO.
