# Phase 20G2D7 — signer reservation evidence and rollback audit

**Status:** Audit submitted for exact-head CI and review; documentation only.  
**Base main:** `2198bb5f49834f2d144f18f85deb9e314c6d3745`.  
**Preceding gate:** Phase 20G2D6 merged; post-merge main CI run #42459801284 passed all four jobs.  
**Release impact:** G2/G4 remain BLOCKED; production remains NO-GO.

## Objective

Determine whether the current signer reservation journal can supply complete, fresh, independently verifiable evidence to a future authenticated recovery manifest. Distinguish structural inspection of a database image from proof that the image contains every signing reservation and is not a rolled-back copy.

## Audited implementation

The production-workspace `apex-signer-redb` store uses redb 4.5.0 and schema marker `apex_signer_meta_v1[0] = 1`. Its reservation table is `apex_signing_reservations_v1`:

- Key: exactly 81 bytes: 32-byte chain ID, 4-byte protocol version, 8-byte height, 4-byte round, 1-byte consensus step, and 32-byte signer ID. Numeric fields are big-endian.
- Value: exact caller-supplied message bytes, non-empty and bounded when a `SigningIntent` is constructed.
- Reservation behavior: first reservation is committed before `Reserved` is returned; exact-byte retry is idempotent; different bytes at the same coordinate fail closed.
- The current store opener uses `Database::create`, opens tables in a write transaction and commits it. It is not a forensic read-only opener.
- Phase 20G2D6's isolated reader can enumerate the current v1 layout using `ReadOnlyDatabase`, validate coordinate shape and non-empty values, enforce caller limits, and check source-byte invariance.

## Findings

1. **Structural validity is not message authentication.** The inspection reader does not decode the value as an APEX Phase 20G2A preimage or Malachite native signing request, verify a signature, or prove the value is the exact byte sequence passed to a native key backend. The reservation API accepts opaque caller-supplied bytes.
2. **Rows present are not proof of complete signer coverage.** There is no authenticated record that every signer/key-use event was intercepted by this journal. The current APEX redb store is not yet integrated as a wrapper around Malachite's native signer.
3. **No freshness or anti-rollback signal exists.** The schema has no monotonic generation, checkpoint sequence, external high-water mark, or authenticated checkpoint root. A restored older but internally valid database can omit later reservations and still pass structural inspection. A file SHA-256 identifies bytes; it does not prove that those bytes are the newest authorized state.
4. **WAL absence is not proof of absent signing.** The Phase 20G2C9 audit found that pinned Malachite v0.8.2 invokes its signer before WAL append. A crash between signing and append can leave no corresponding WAL record. Recovery must not infer that no signature was produced merely because no WAL entry exists.
5. **Signing domains must remain distinct.** APEX Phase 20G2A's canonical preimage and Malachite's native signing bytes are separate domains. A future wrapper must reserve the exact bytes consumed by the native signer and separately bind them to canonical APEX identity; it must not substitute one domain's bytes for the other.
6. **The reader's source digest is diagnostic.** It provides byte identity for the inspected file image, not authenticated provenance, an independently protected monotonic anchor, or completeness relative to an external checkpoint.

## Decision

Do not treat the Phase 20G2D6 signer scan, its report digest, a source-file hash, or WAL replay alone as an authoritative signer checkpoint. Keep the signer journal and inspection code experimental for recovery purposes; do not connect them to startup authorization.

Before manifest integration, a separate reviewed phase must define:
- an authenticated signer checkpoint with a monotonically increasing generation/high-water mark;
- an anti-rollback anchor whose trust and storage are independent of the signer database and recovery directory;
- the exact checkpoint coverage semantics (chain, protocol, signer identity, coordinate range, journal digest and generation);
- a domain-correct wrapper that reserves the exact Malachite native signing bytes durably before invoking any key backend;
- crash tests at reservation, key-use, WAL append and acknowledgement boundaries;
- fail-closed reconciliation when checkpoint, journal and WAL evidence disagree.

The checkpoint must not claim to prove a signature was or was not emitted unless the signer boundary itself provides that attestation. No specific signature suite, hardware anchor, external service, or production key custody mechanism is selected here.

## Non-goals

No code, schema, signer, WAL, manifest, cryptographic, or startup behavior is changed. This audit does not establish signer completeness, backup freshness, key custody, anti-double-sign protection across rollback, production finality, or operational durability.

## Exit criteria

- The current signer schema and open/write behavior are recorded accurately.
- The distinction between structural scan, completeness, authentication and freshness is explicit.
- The Malachite signer-before-WAL ordering and signing-domain separation are reflected in recovery constraints.
- Exact-head CI passes for the unchanged code plus documentation.
- G2/G4 remain BLOCKED; production release remains NO-GO.
