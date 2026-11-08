# ADR-0063 — signer reservation evidence and rollback boundary

- **Status:** Proposed for review; audit only.
- **Date:** 2026-11-08
- **Context:** ADR-0055 through ADR-0062 define cross-subsystem recovery, WAL preflight, read-only store access and the authenticated recovery-manifest requirements. Phase 20G2D6 can structurally enumerate the current signer reservation table, but an enumerated database image is not necessarily complete, fresh or authenticated.
- **Decision:** Do not treat the signer reservation table, source-file digest or WAL replay alone as an authoritative signer checkpoint. The v1 table maps fixed-width 81-byte coordinates to opaque message bytes. It has no authenticated checkpoint, monotonic generation, independent high-water mark or anti-rollback anchor. The current APEX reservation store is not integrated as a wrapper around Malachite's native signer.
- **Critical ordering constraint:** The pinned Malachite v0.8.2 audit found signer invocation precedes WAL append. Absence of a WAL entry therefore cannot prove absence of key use. A future wrapper must durably reserve the exact native signing bytes before invoking the key backend, then separately bind those bytes to canonical APEX identity. APEX Phase 20G2A bytes and Malachite native signing bytes are distinct domains.
- **Required future evidence:** A separately protected monotonic checkpoint/generation; explicit coverage semantics; authenticated checkpoint digest; exact native-byte reservation-before-key-use proof; crash-boundary tests; and fail-closed reconciliation of signer, WAL and finality evidence.
- **Non-decisions:** No checkpoint codec, signature suite, key custody, hardware/external anchor, trust root, production signer wrapper or startup authorization is selected.
- **Consequences:** A structurally valid old backup may omit later reservations. Recovery remains locked when freshness or signer coverage cannot be established. G2/G4 remain BLOCKED; release remains NO-GO.
