# Phase 20G2D5 — redb read-only recovery access audit

**Status:** Version-specific source audit; implementation spike required before this API is accepted for recovery.  
**Base main:** `7b61757ab457a821d3fedf2544b41a5cd9e25ab7`.  
**Pinned storage dependency:** redb `=4.5.0`.  
**Preceding gate:** Phase 20G2D4 is merged and post-merge main CI passed.

## Objective

Determine whether the exact redb version already pinned by APEX provides a supported way to inspect existing signer/finality database files without running the stores' mutating convenience-open paths. This phase narrows the D4 finding: a read-only database primitive exists in redb 4.5.0, but its behavior has not yet been exercised against APEX's actual schemas and recovery fixtures.

## Version-specific finding

The workspace pins redb exactly to `4.5.0`. The 4.5.0 API exposes `ReadOnlyDatabase::open(path)` and `Builder::open_read_only(path)`; read access is obtained through `ReadableDatabase::begin_read()` and read-only table handles. The upstream 4.5.0 source documents that read-only open does not modify the database. If the file requires recovery, read-only open rejects it rather than repairing it.

This is materially different from the current APEX convenience paths:
- `RedbSigningSafetyStore::open` calls `Database::create`, initializes metadata/reservation tables in a write transaction and commits.
- `FinalityJournal::open` calls `ensure_tables`, which opens a write transaction and commits table initialization.

Neither convenience path is approved for forensic inspection of the only evidence copy.

## Locking and operational constraints

1. The default redb concurrency mode is `ExclusiveWriter`. A `ReadOnlyDatabase` cannot be opened concurrently with a `Database` writer in this mode on platforms with file locks.
2. redb 4.5.0 also documents `SingleWriter` and `MultiWriter` modes that allow readers alongside a writer. APEX has not selected or configured either mode. This phase does not change concurrency mode or imply that enabling it is safe.
3. On platforms without supported file locks, callers must enforce writer exclusion operationally; the library cannot provide the same cross-process exclusion guarantee.
4. A database requiring recovery is rejected rather than repaired. That is the desired fail-closed behavior for an evidence reader, but rejection is not a diagnosis or a repair authorization.
5. A successful read-only open proves only that redb accepted the file for read access. It does not prove signer journal completeness, finality continuity, application-state correctness, provenance, or anti-rollback.

## Proposed next implementation spike

Add isolated tests against the exact pinned redb 4.5.0 API. Use fixtures created and closed by a writer, then:
- open only through `ReadOnlyDatabase`;
- enumerate signer reservation and finality journal rows using the exact existing table names and key/value encodings;
- verify schema metadata and reject unknown/partial schemas;
- enforce byte and record bounds and deterministic key ordering;
- compute a report digest over canonical, explicitly versioned report bytes;
- hash the source file before and after inspection and assert byte-for-byte non-mutation;
- verify an active exclusive writer prevents read-only open where file locks are supported;
- construct a recovery-required/corrupt fixture using controlled test-only fault injection and verify that read-only open rejects it without altering the source.

The tests must not call `RedbSigningSafetyStore::open` or `FinalityJournal::open` on the evidence source. If a disposable copy is needed for fault injection, preserve and hash the original separately.

## Explicit non-decisions

No production store API is changed. No redb concurrency mode, migration, repair path, serialization format, hash domain, state-root algorithm, signature suite, recovery trust root, anti-rollback service, WAL actor, or startup authorization is selected. The application projection still lacks a specified canonical commitment tied to a finality identity.

## Exit criteria and safety gate

The implementation spike must demonstrate read-only access and source non-mutation on exact APEX-compatible fixtures, enumerate every row or fail closed, detect schema/key/value/identity errors, and distinguish a complete report from an incomplete or rejected database. It must document platform lock limitations and retain a separate immutable evidence copy.

Even after that spike passes, signer/finality export is diagnostic evidence only until independently authenticated and anchored. Application-state commitment, WAL replay/reconciliation, independent anti-rollback and locked-startup enforcement remain open.

**G2 and G4 remain BLOCKED. Production release remains NO-GO.**
