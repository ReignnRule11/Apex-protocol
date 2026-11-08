# ADR-0064: Signer checkpoint and independent anti-rollback contract

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related:** ADR-0055, ADR-0056, ADR-0059, ADR-0060, ADR-0061, ADR-0063
- **Phase:** 20G2D8

## Context

Phase 20G2D7 established that the current APEX signer reservation v1 journal is structurally inspectable but does not authenticate message semantics, prove complete interception of native key use, or establish freshness. It has no monotonic generation or authenticated checkpoint. A valid older database image can omit later reservations. The Malachite v0.8.2 signer-before-WAL ordering also prevents WAL absence from proving that no signature was emitted.

A recovery manifest therefore cannot treat the current signer scan, source digest, or WAL replay as authoritative signer evidence.

## Decision

Define four separate trust components:

1. the reservation journal, which stores exact signing-domain bytes;
2. a checkpoint attestor, which authenticates a statement only after validating its evidence;
3. an independent monotonic anchor, which protects accepted generation/digest state outside the signer database and recovery backup domain;
4. a recovery verifier, which reconciles all evidence while startup remains locked.

A future checkpoint must bind chain/protocol/signer/signing-domain scope, generation and predecessor digest, exact journal identity and inspection evidence, explicit coverage bounds/gap semantics, and evidence identifying the signer wrapper and its intercepted native key-use boundary.

The anchor must support atomic compare-and-advance. It rejects lower generations, conflicting digests at the same generation, and advancement from an unexpected predecessor. It must not be restorable merely by restoring signer/recovery files. Anchor enrollment, reset, migration, and trust-root rotation require separate authorization.

Coverage claims require a domain-correct APEX wrapper that durably reserves the exact bytes passed to every native key backend before invocation. The wrapper must preserve the distinction between APEX's canonical Phase 20G2A preimage and Malachite's native signing bytes. Reservation intent, backend invocation, and signature emission are distinct evidence states.

## Consequences

- Hashes identify evidence but do not authenticate it.
- Attestations authenticate statements but do not prove their inputs complete or fresh.
- Only the independent anchor protects monotonic freshness for its exact scope.
- A checkpoint cannot claim complete signer coverage until all native key-use paths are inventoried and the wrapper is tested against bypass and crash cases.
- WAL absence cannot prove absence of signing because signer invocation precedes WAL append.
- Recovery remains `RECOVERY_LOCKED` until checkpoint, anchor, signer, WAL, finality, and application evidence reconcile and a separate recovery policy authorizes continuation.
- No cryptographic suite, canonical codec, attestor key custody, anchor technology, quorum, production signer wrapper, or startup integration is selected.
- G2/G4 remain BLOCKED; production remains NO-GO.

## Rejected alternatives

- **Use a source-file hash as freshness:** rejected; hashes identify bytes but do not establish recency or provenance.
- **Use an attestation without an independent anchor:** rejected; a valid old attestation can be replayed.
- **Treat the current signer row scan as complete coverage:** rejected; it cannot prove every native key-use event was intercepted.
- **Infer no signature from missing WAL:** rejected; the audited signer-before-WAL order leaves a crash window.
- **Let checkpoint verification authorize startup:** rejected; checkpoint validity is one input to cross-store reconciliation, not a startup capability.

## Implementation gate

Before implementation, select and review the canonical encoding, attestation algorithm/key policy, independent anchor technology and trust model, scope partitioning, concurrent generation allocation, signer-wrapper completeness proof, signature-emission semantics, and recovery governance. Add adversarial tests for rollback, forked generations, same-generation digest conflicts, anchor outage/reset, wrapper bypass, and every crash boundary. Do not promote this ADR to Accepted until those decisions and tests are reviewed.
