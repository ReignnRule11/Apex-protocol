# Phase 20D — Durable Consensus Signer Safety

**Status:** Implementation submitted for exact-head CI validation.
**Base main:** `e7fd62b4646e3b022113f452175c1e00df2ad5ce`
**Release impact:** G2 and G4 remain BLOCKED. Production remains NO-GO.

## Objective

Establish the minimum durable anti-double-sign primitive before any consensus signer, validator key or engine actor is enabled. A signing coordinate must be reserved durably before the caller is allowed to invoke a signing backend.

## Implementation

- `apex-signer-safety` defines typed chain/protocol/height/round/step/signer coordinates.
- Every coordinate is encoded into a fixed-width 81-byte key with explicit big-endian numeric fields.
- `SigningIntent` holds exact caller-supplied message bytes and enforces a caller-configured non-zero size limit.
- `SigningSafetyStore::reserve` permits a first reservation, permits an exact-byte retry, and rejects different bytes at the same coordinate.
- The in-memory implementation is a test reference only.
- `apex-signer-redb` persists reservation keys and exact message bytes in one redb write transaction. It returns only after commit; a reopened store retains the reservation and rejects conflicting bytes.
- Existing or partially initialized schema metadata is checked and incompatible schema fails closed.

## Safety model and limitations

This is a durable reservation journal, not a production signer. It deliberately does not select a consensus signature algorithm, key format, validator identity registry, message codec, signing-domain tag, or engine vote semantics. Those require a reviewed protocol ADR and Malachite/CometBFT mapping.

The caller must construct canonical, fully domain-separated signing bytes before reservation. The external signer must be called only after `Reserved` or `ExistingIdentical` is returned. A process crash after reservation but before signature creation is safe for anti-double-sign: restart may reattempt only the exact same bytes. A crash after signing but before broadcast likewise cannot authorize different bytes at that coordinate.

This prototype is single-process and relies on redb's exclusive file lock. It does not implement HSM/KMS integration, key custody, encrypted key storage, backup/restore anti-rollback, key rotation, validator-set activation, engine integration, multi-process signing, or a production incident/recovery procedure. Restoring an old database backup could roll back reservations; production must prevent rollback with an independently protected monotonic anchor or external signer policy before validator keys are enabled.

Caller-supplied message limits are operational bounds, not protocol constants. This journal stores opaque bytes and does not prove that they represent a valid consensus vote.

## Tests

- First reservation and exact retry are idempotent.
- Conflicting bytes at the same coordinate are rejected.
- Distinct coordinates remain independent.
- Zero chain identity, empty payload and oversized payload are rejected.
- Fixed-width key binds signer identity and consensus coordinates.
- redb reservation survives close/reopen and rejects a conflicting post-restart message.

## Exit criteria

1. Exact-head Format, Clippy and all workspace tests pass.
2. Persistence tests pass across close/reopen.
3. Review confirms commit occurs before any external signing call can be made by a future integration.
4. Backup rollback, multi-process, crypto suite, key lifecycle and engine integration remain explicit blockers.

**Decision:** Durable anti-double-sign reservation prototype GO; production validator signing NO-GO.
