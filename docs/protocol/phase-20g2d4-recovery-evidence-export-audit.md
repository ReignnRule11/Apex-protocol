# Phase 20G2D4 — recovery evidence export capability audit

**Status:** Source audit and implementation-boundary specification submitted for review; no runtime changes or production approval.  
**Base main:** `fde52bca2e28f3228b683c04a8eeb81fbe2a3c02`.  
**Pinned engine:** Malachite v0.8.2, commit `a453b32155f4ba9f959724279b52f601bae92e09`.  
**Preceding gates:** 20G2D1–20G2D3 are merged; Phase 20G2D3 post-merge main CI passed.

## Objective

Determine whether the current signer journal, finality journal and application-state surfaces can provide the independently inspectable, non-mutating evidence required by the authenticated recovery manifest. Define the boundary that must exist before an authoritative manifest can be created. This phase is an audit/specification only: it does not add readers, export APIs, codecs, cryptography, startup logic or storage migrations.

## Findings

### 1. Signer reservation journal

The engine-neutral `SigningSafetyStore` contract exposes only `reserve(&SigningIntent)`. It has no read, enumerate, checkpoint, schema-inspection or export operation. The redb implementation stores version metadata and reservation rows in private tables; the storage key binds chain ID, protocol version, height, round, step and signer ID, while the value is the exact reserved message bytes.

The current `RedbSigningSafetyStore::open` uses `Database::create`, opens/initializes tables and commits a write transaction before returning. This is not an approved forensic read-only open path. There is no public method to enumerate reservations, compute a stable journal commitment, expose a monotonic high-water mark, or distinguish a complete journal from an incomplete backup using an independent checkpoint.

**Result:** the current signer abstraction cannot supply the manifest's signer evidence. Do not open the only signer journal through this API during recovery or infer a safe watermark from filenames, file size or a partial set of records.

### 2. Finality journal

The isolated `FinalityJournal` stores a block identity plus opaque application-state payload in one row and maintains a per-chain tip. It exposes point lookup (`read_payload`) and tip lookup (`tip`), but no complete ordered iterator, read-only snapshot handle, schema report, row count, journal digest or authenticated checkpoint export.

Its `open` path calls `ensure_tables`, which creates a write transaction and commits table initialization. Consequently, the current convenience open path is not a strictly read-only forensic interface, even when the caller intends only to inspect. The current API also cannot prove that every row through the reported tip is present and valid without a full ordered audit.

**Result:** the finality journal's tip API is insufficient as authoritative manifest evidence. A future exporter must inspect every relevant record, verify key/identity/payload consistency, enforce contiguous height and parent linkage, and produce a deterministic commitment without mutating the source.

### 3. Materialized application state

The production-readiness register describes EVM state persistence as a separate prototype. The recovery design requires a chain-bound application projection height, canonical state commitment and a verifiable relationship to the finality journal. The audited recovery surfaces do not currently provide a single read-only export contract proving those values correspond.

**Result:** do not substitute a database version counter, latest block number, or opaque payload hash for a canonical application-state commitment. The exact state commitment and export boundary remain unresolved.

### 4. Malachite WAL boundary

The preceding pinned-source audit establishes that default WAL open/fetch/startup paths may truncate or reset records. The read-only scanner and bounded snapshot acquisition operate before Malachite opens the source, but they do not prevent a later default engine path from mutating the only evidence copy. The manifest must bind the preserved snapshot, not a subsequently repaired working WAL.

**Result:** no default WAL startup/reset path is approved as a recovery evidence reader. A guarded custom actor or disposable working-copy design remains unproven and must be tested against the exact pinned actor protocol.

## Evidence capability matrix

| Domain | Current observable surface | Missing authoritative evidence |
|---|---|---|
| WAL | Bounded snapshot bytes, SHA-256, framing/CRC scanner report, format/sequence when parseable | Proven quiescence, authenticated provenance, complete semantic replay, protection from later implicit reset/truncation |
| Signer | Reservation writes and exact-coordinate conflict rejection | Non-mutating open, complete enumeration, schema identity, monotonic checkpoint, deterministic journal commitment |
| Finality | Point payload read and per-chain tip read | Non-mutating open, full ordered audit, row/parent continuity proof, schema identity, deterministic journal commitment |
| Application projection | Separate state prototype and opaque finality payload | Canonical state commitment, export consistency with finality tip, read-only verification |
| Trust/anchor | No selected production mechanism | Independently provisioned signer policy, key epoch/revocation, monotonic external anti-rollback checkpoint |

## Required future export contract

Any implementation proposal must provide a recovery-only interface with these properties:

1. **Non-mutating source access:** no create, table initialization, migration, truncate, repair, compaction, sync, or write on the source evidence. If the storage library cannot guarantee this, export from a separately preserved immutable copy and label the limitation.
2. **Complete bounded enumeration:** explicit byte/record limits; deterministic ordering; reject duplicate, malformed, unknown-version and out-of-range records.
3. **Schema and identity binding:** report schema version, database identity, chain ID, protocol version, engine/source revision and export implementation revision.
4. **Signer audit:** enumerate every reservation; validate each fixed-width coordinate and exact message; detect conflicting duplicates; derive per-signer and global high-water marks without assuming that height alone is a safe ordering.
5. **Finality audit:** enumerate every record; decode identity; verify row key matches embedded identity; verify payload bounds; prove contiguous height and parent linkage; compare the recomputed tip with the stored tip.
6. **Application audit:** derive a canonical state commitment using an explicitly specified state model; bind it to the exact finality identity and projection height.
7. **Deterministic report:** canonical report bytes and digest, with explicit coverage, omissions, limitations and source-copy identity. A report digest is not provenance authentication.
8. **Independent anchoring:** authenticate the report under the separately governed recovery policy and advance an external monotonic anchor only after durable preservation and independent verification.
9. **Fail-closed behavior:** missing APIs, partial enumeration, I/O ambiguity, unknown schemas, mismatch or inability to prove completeness yields diagnostic-only evidence and leaves startup locked.

## Explicit non-decisions

This phase selects no serialization format, cryptographic suite, signer key custody/threshold, trust-root distribution, anti-rollback technology, redb read-only mode, state-root algorithm, WAL actor implementation or engine startup path. It does not claim that current stores can be safely opened read-only, and it does not authorize migrations or data repair.

## Next gate and exit criteria

Before implementation, separately verify the exact redb version's read-only/open and backup semantics, inspect every store schema and mutation path, specify canonical state commitment inputs, and prove a complete export against adversarially corrupted and stale copies. Then design a guarded engine boundary that cannot invoke default WAL reset/truncation before reconciliation.

**G2 and G4 remain BLOCKED. Production release remains NO-GO.** A green CI run validates only the code under test; it does not establish cross-store atomicity, power-loss durability, backup rollback resistance or production recovery safety.
