# Phase 20G2C5 — idempotent finality commit contract

**Status:** Implementation submitted for exact-head CI; production adoption remains blocked.  
**Base main:** `7f2945c629f2eea41779f0f210038f7f8094a5c7`  
**Pinned Malachite:** v0.8.2, `a453b32155f4ba9f959724279b52f601bae92e09`.

## Objective

Close a safety gap in the Phase 20G2C3/C4 callback contract: a storage commit can succeed but its acknowledgement or returned next-height parameters can be lost. Treating every callback error as proof of non-commit and issuing `Next::Restart(current_height)` can replay an already-applied block.

## Delivered

- Added `FinalityCommitKey`, derived from the opaque verified token's complete APEX identity (chain ID, protocol version, height, parent hash and block hash).
- Made the key available only from `VerifiedNativeFinality::commit_key()`; it is not caller-selected.
- Classified commit callback failures:
  - `DefinitelyNotCommitted`: storage proves no state was committed; same-height restart is permitted.
  - `Ambiguous`: commit may have succeeded; drop the Finalized reply and stall pending durable recovery.
  - `ConflictingFinality`: another identity occupies the same chain height; drop the reply and treat as a safety halt.
- Required application implementations to atomically persist state and the commit key, and to make retries for the same key idempotent.
- Added deterministic tests proving ambiguous/conflicting outcomes do not trigger a same-height restart, while a definitely uncommitted operation may restart.

## Required application storage semantics

A future concrete adapter must implement a transaction equivalent to:

1. Validate the verified token and derive its immutable commit key.
2. Read the durable finalized-height record and commit journal in one serializable transaction.
3. If the same key is already committed, return the existing committed result without reapplying state.
4. If a different key is committed at the same height, return `ConflictingFinality` and halt.
5. If the expected parent/height does not match durable state, reject without mutation.
6. Atomically write the complete application state, finalized identity/key and recovery metadata.
7. Only after durable commit, construct and return next-height parameters.
8. On uncertain I/O outcomes, return `Ambiguous`; startup recovery must inspect the durable journal before consensus resumes.

A commit key alone does not provide atomicity. The state transition, finalized marker and replay metadata must share one atomic durability boundary or a separately proven recoverable transaction protocol.

## Test evidence

The isolated gate tests exercise the error classification and reply behavior using synthetic callbacks. They do not provide a real transactional database, prove atomicity, simulate power loss, or establish restart reconciliation. The adapter remains in the separate Malachite spike workspace.

## Explicit non-goals and blockers

- No concrete production database adapter or consensus/application atomic transaction.
- No authenticated validator-set governance or signer anti-rollback.
- No durable restart journal, backup rollback protection or crash-recovery proof.
- No live Malachite engine, network, multi-validator integration or BFT safety/liveness proof.
- No independent cryptographic/security audit.

**G2 remains BLOCKED. Overall release remains NO-GO.**

## Next

Implement a bounded canonical APEX value store and authenticated validator-set provenance behind a concrete isolated `FinalityApplication`; then integrate durable commit-journal recovery and exercise process-crash/power-loss fault injection before considering production workspace adoption.
