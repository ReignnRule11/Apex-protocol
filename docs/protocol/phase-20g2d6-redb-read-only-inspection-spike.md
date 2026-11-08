# Phase 20G2D6 — exact-version redb read-only inspection spike

**Status:** Implementation submitted for exact-head CI; isolated research code only.  
**Base main:** `9e2fbd74914b2ef92059fa50bcdee9377c8bc2bd`.  
**Pinned storage dependency:** redb `=4.5.0`.  
**Preceding gate:** Phase 20G2D5 merged; post-merge main CI run #42457426702 passed all four jobs.

## Objective

Exercise redb 4.5.0's read-only database handle against fixtures matching the existing APEX signer and finality table layouts. The spike must enumerate records deterministically, enforce caller-supplied source/record/payload limits, validate structural identity, and prove that the source file bytes remain unchanged during inspection.

## Implementation boundary

The new isolated module is `experiments/malachite-spike/src/redb_recovery_inspection.rs`. It is exposed only from the independent Malachite spike crate and is not added to the production Cargo workspace.

- Signer inspection opens `ReadOnlyDatabase`, validates the exact v1 metadata row, requires the expected 81-byte coordinate key, rejects zero chain/signer IDs, zero protocol version, unknown consensus-step values and empty message bytes, and bounds record count and aggregate record bytes.
- Finality inspection opens the existing journal/tip tables read-only, validates 44-byte height keys and 108-byte embedded identities, checks key-to-identity binding, payload bounds, contiguous per-chain heights, parent linkage, and agreement between enumerated journal tips and stored tip rows.
- Both reports bind the source byte length and SHA-256, record counts/bytes and explicit report-version domain into a deterministic digest. Source bytes are hashed before and after the read-only scan; any difference fails closed.
- The finality table has no explicit schema marker in the current implementation. The report records this limitation rather than claiming schema-version authentication.
- Tests construct disposable fixtures through ordinary writer transactions, close them, then inspect through the read-only API and compare source bytes. Negative tests cover malformed signer step and finality identity/parent structure, plus a record-count limit.

## Known limitations / non-claims

This is not an authoritative recovery exporter or authenticated manifest producer. The report digest is not a signature, provenance proof, independent anti-rollback anchor, or proof that a backup is the latest valid copy. The finality payload remains opaque application bytes; no canonical application-state root is derived. No cross-store consistency, WAL replay, signer watermark policy, key custody, recovery authorization or startup lock is implemented.

The byte-invariance check detects changes observable between pre/post snapshots; it does not prove immutable hardware, eliminate every concurrent rewrite race, or replace external writer exclusion. Platform-specific lock behavior still requires dedicated verification. The tests do not claim power-loss/device-failure durability.

## Exit criteria for this spike

- Exact redb 4.5.0 read-only API compiles in the isolated workspace.
- Valid signer/finality fixtures are fully traversed under explicit bounds.
- Malformed schemas/records, gaps, parent mismatches, tip mismatches and resource-limit violations fail closed.
- Source bytes remain identical for successful and rejected inspections.
- Exact-head CI passes for the APEX workspace, isolated spike and pinned upstream suite.

**G2 and G4 remain BLOCKED. Production release remains NO-GO.**
