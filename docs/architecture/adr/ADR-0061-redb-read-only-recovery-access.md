# ADR-0061 — redb read-only recovery access boundary

- **Status:** Proposed for review; version-specific source audit only.
- **Date:** 2026-11-08
- **Context:** Phase 20G2D4 found that APEX signer and finality convenience-open paths commit table initialization writes and cannot be used as forensic read-only readers. The workspace pins redb exactly to 4.5.0.
- **Decision:** Evaluate redb 4.5.0 `ReadOnlyDatabase` / `Builder::open_read_only` as the only currently identified supported library-level entry point for future signer/finality evidence inspection. Do not use the existing APEX store `open` methods on the sole evidence copy.
- **Version-specific behavior:** redb 4.5.0 documents that read-only open does not modify the database and rejects a database that requires recovery rather than repairing it. Reads occur through `ReadableDatabase::begin_read` and read-only table handles.
- **Locking constraints:** The default `ExclusiveWriter` mode does not permit a read-only handle alongside a writer on platforms with file locks. `SingleWriter` and `MultiWriter` have different sharing semantics, but APEX has not selected either. Platforms without file locks require external writer exclusion.
- **Required proof:** An isolated exact-version spike must enumerate APEX-compatible signer/finality fixtures, enforce bounded deterministic traversal, reject schema and record inconsistencies, and prove source bytes are unchanged before and after inspection. Recovery-required files must be rejected without mutation.
- **Non-decisions:** No store API, concurrency mode, schema migration, report codec, cryptographic suite, state commitment, trust root, anti-rollback anchor or startup path is approved.
- **Consequences:** redb supplies a plausible read-only access primitive, narrowing but not closing the D4 gap. A successful open is not proof of journal completeness, finality validity, application-state correctness, provenance or anti-rollback. G2/G4 remain BLOCKED; release remains NO-GO.
