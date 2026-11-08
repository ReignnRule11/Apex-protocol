# ADR-0062 — isolated redb read-only inspection spike

- **Status:** Proposed for review; experimental implementation only.
- **Date:** 2026-11-08
- **Context:** ADR-0061 established that redb 4.5.0 exposes a read-only open path, while existing APEX signer/finality convenience openers perform writes. Phase 20G2D6 exercises that primitive against the exact current table layouts.
- **Decision:** Add a read-only inspection module only to the independent `experiments/malachite-spike` workspace. It must not be added to the production workspace or called by normal store startup.
- **Signer scope:** Validate v1 metadata and enumerate bounded 81-byte reservation coordinates and exact non-empty message values.
- **Finality scope:** Enumerate bounded journal rows, bind height keys to embedded identities, check contiguous heights and parent linkage, and compare the resulting chain tips with the stored tip table.
- **Non-mutation check:** Hash source bytes before and after inspection and fail if length or digest changes. This is a testable guard, not a claim of immutable hardware or a substitute for writer exclusion.
- **Schema limitation:** The existing finality journal has no explicit schema-version table. The report must expose that fact; it must not infer a version marker.
- **Report boundary:** A deterministic diagnostic digest binds the source length/hash, record counts/bytes and report domain. It is not authenticated evidence and cannot authorize recovery.
- **Non-decisions:** No production exporter, canonical application-state root, signature suite, recovery trust root, monotonic anti-rollback anchor, WAL actor, cross-store reconciliation, startup gate or redb concurrency mode is selected.
- **Consequences:** The spike can prove bounded structural inspection against fixtures, but not completeness relative to an external checkpoint, provenance, freshness, backup rollback resistance, application-state correctness or production recovery safety. G2/G4 remain BLOCKED; release remains NO-GO.
