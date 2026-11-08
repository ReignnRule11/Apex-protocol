# Phase 20G2D3 — authenticated recovery manifest design

**Status:** Design specification submitted for review; no runtime implementation or production approval.  
**Base main:** `d7947405eb4beb93b0f1d82163387fe176e267da`.  
**Pinned upstream:** Malachite v0.8.2, commit `a453b32155f4ba9f959724279b52f601bae92e09`.  
**Preceding gates:** 20G2D1 read-only scanner and 20G2D2 quiesced snapshot acquisition are merged and post-merge CI verified.

## Objective

Specify an authenticated, anti-rollback recovery manifest that binds a preserved Malachite WAL byte snapshot to the exact software, chain configuration and independently persisted APEX recovery domains that an operator must reconcile before startup. This phase is design-only. It does not claim that a file hash authenticates provenance, that the current stores expose all required evidence, or that a manifest makes those stores atomic.

## Security properties and non-properties

The future manifest protocol must provide:

1. **Integrity:** canonical manifest bytes are covered by a cryptographic authenticator.
2. **Context binding:** the record binds the chain, protocol, engine revision, WAL format, snapshot digest and all required store tips/commitments.
3. **Freshness/anti-rollback:** an independently protected monotonic anchor or authenticated checkpoint establishes that a presented manifest is not an older valid manifest.
4. **Signer accountability:** the authenticator identifies an authorized recovery-attestation key and the policy/key epoch used.
5. **Fail-closed parsing:** unknown schema, encoding, algorithm, policy epoch, missing fields, duplicate fields, overflow and malformed digests are rejected.
6. **Reproducibility:** an independent verifier can recompute the manifest digest and verify every referenced artifact without trusting mutable filenames or timestamps.

A SHA-256 digest alone supplies integrity identification, not provenance or authorization. A signature alone does not prevent replay of an older valid manifest. A manifest does not prove that the source was quiescent, that the acquired bytes were an atomic filesystem snapshot, that compressed WAL records were fully verified, or that the signer/finality/application stores are mutually consistent. Those claims require separate evidence and checks.

## Proposed manifest envelope

The manifest is a versioned canonical record. The following is a field contract, not a selected serialization format or final schema:

| Field group | Required contents | Validation |
|---|---|---|
| Envelope | schema identifier, schema version, canonical encoding identifier, hash algorithm identifier, authenticator algorithm identifier | Exact allowlisted values; reject unknown versions/algorithms |
| Network identity | chain ID, protocol version, genesis/configuration digest | Must match independently trusted node configuration |
| Engine identity | engine name, exact upstream commit, adapter build/source revision, WAL format version | Must match the audited and deployed binaries |
| Snapshot identity | snapshot SHA-256, byte length, acquisition method/version, source logical identifier, source filesystem/file identity where available | Recompute digest/length from preserved bytes; identity is descriptive evidence, not proof of source authenticity |
| Scanner evidence | scanner version/revision, scanner result digest, entry count and explicit integrity coverage/limitations | Re-run scanner against the exact snapshot; compressed/unverified records remain explicitly unverified |
| Store evidence | signer journal schema and monotonic reservation checkpoint; finality journal schema/tip/identity; application projection height/identity/state commitment | Independently read and compare against the actual stores; no filename-only references |
| Recovery context | recovery generation, previous manifest digest, trusted anchor identifier/sequence, operator incident/change reference | Must be monotonic and anchored outside the stores being recovered |
| Attestation | recovery key identifier, key epoch/policy identifier, signature over the canonical unsigned manifest digest | Verify against an independently provisioned trust policy; never use validator signing keys as recovery-attestation keys by default |

The exact encoding, integer widths, string normalization, optional-field rules, signature suite, key custody, threshold policy and trust-root distribution remain open decisions. JSON serialization must not be treated as canonical merely because it is deterministic in one implementation. A future codec must specify byte-for-byte canonical vectors and reject alternate encodings.

## Authentication and trust model

- The recovery-attestation key is a distinct operational identity from validator keys, transaction keys, peer identities and the Malachite signer. This avoids conflating recovery authorization with consensus signing authority.
- The verifier must obtain the authorized key set, key epoch, revocation state and policy from a trust source independent of the WAL and databases under recovery.
- Key rotation must be chained by an independently authenticated policy update. A manifest cannot authorize its own signer or its own policy.
- The deployment policy must decide whether a single recovery authority, threshold authorization, or offline quorum is required. This phase selects none.
- The signed digest must domain-separate the APEX recovery-manifest protocol and bind schema version and chain identity. The exact cryptographic suite remains unselected pending review.
- Operator identity and incident references are audit metadata; they do not replace cryptographic authorization.

## Anti-rollback anchor

The manifest must include a monotonic generation/sequence and previous-manifest digest, but those fields are insufficient unless checked against a trust anchor that an attacker cannot roll back together with the recovered data directory.

Acceptable anchor candidates for future evaluation include a separately administered append-only remote service, an offline signed checkpoint distributed through an independent channel, or hardware-backed monotonic storage. No candidate is selected here. A local file beside the WAL, signer journal or finality database is not an independent anchor.

If the anchor is unavailable, stale, inconsistent, or cannot prove that the presented generation is current, startup remains locked. Restoring an older backup requires an explicit incident procedure with independently authenticated authorization; it must not silently reset the generation.

## Creation and verification protocol

### Manifest creation

1. Enter a serialized recovery window and prevent all WAL writers and node processes from accessing the source.
2. Preserve the source using the bounded read-only acquisition path. Record the exact acquired bytes and SHA-256; retain the original source unchanged.
3. Run the strict scanner against those exact bytes and retain its complete machine-readable report. Do not upgrade a partial/compressed validation result to a clean integrity claim.
4. Independently inspect signer, finality and application stores read-only. Capture their schema versions, identities, monotonic checkpoints, tips and commitments. If any required field cannot be obtained, do not issue an authoritative manifest.
5. Build canonical manifest bytes using the reviewed schema and encoding; compute the domain-separated digest.
6. Obtain authorization from the independently governed recovery-attestation policy. Persist the manifest, signature and referenced evidence in a preservation location separate from the stores under recovery.
7. Advance the external anti-rollback anchor only after the manifest and its evidence are durably preserved and independently verifiable.

### Manifest verification

1. Load the trust policy and latest anchor independently of the recovery directory.
2. Parse strictly and reconstruct canonical bytes; reject non-canonical or unknown encodings.
3. Verify chain/configuration identity, schema and algorithm allowlists, signer authorization/key epoch, signature, generation and previous-manifest linkage.
4. Recompute snapshot hash/length from the preserved bytes and rerun the pinned scanner; compare the report digest and explicit coverage flags.
5. Independently inspect the live/restored stores and compare every recorded schema, checkpoint, finality tip and application commitment.
6. Hand verified evidence to the separate cross-subsystem reconciliation gate. A valid manifest is necessary evidence, not permission to start the engine.
7. Emit a machine-checkable verification report bound to the manifest digest and verifier build. Any mismatch leaves the node in RECOVERY_LOCKED.

## Failure and recovery rules

| Condition | Required behavior |
|---|---|
| Snapshot digest or byte length mismatch | Reject manifest; preserve all artifacts; remain locked |
| Unknown schema, codec, algorithm or policy epoch | Reject; no downgrade or best-effort parsing |
| Signature invalid or signer not authorized | Reject; record security event; remain locked |
| Manifest generation below external anchor | Reject as rollback |
| Anchor unavailable or cannot establish latest generation | Remain locked; no local fallback |
| Previous-manifest chain discontinuity | Reject pending independent operator review |
| Store tip/checkpoint differs from manifest | Do not rewrite manifest or stores; run separate reconciliation/incident procedure |
| Scanner reports partial framing, CRC failure or unverified compressed records | Preserve explicit failure/uncertainty; no clean-snapshot claim |
| Source quiescence cannot be independently established | Manifest may only be labeled diagnostic/non-authoritative; startup remains locked |
| Signature or anchor commit outcome is ambiguous | Stop; preserve evidence; reconcile the anchor before retry |
| Manifest valid but WAL/store reconciliation is incomplete | Remain RECOVERY_LOCKED; manifest is not a startup token |

## Required validation before implementation

- Review and ratify the manifest schema, canonical codec, digest domain and test vectors.
- Select the recovery key architecture, key rotation/revocation process and authorization threshold.
- Select and threat-model an anti-rollback anchor independent of the recovery data directory.
- Prove that the pinned WAL snapshot and scanner evidence can be independently reproduced.
- Define stable read-only export/checkpoint APIs for signer, finality and application stores; do not infer unavailable evidence.
- Test malformed/non-canonical inputs, signature tampering, cross-chain substitution, engine-revision mismatch, replayed manifests, anchor rollback, interrupted manifest publication and ambiguous anchor updates.
- Keep the manifest verifier and recovery tooling outside validator startup until independent security review and end-to-end reconciliation tests pass.

## Exit criteria and explicit non-goals

Exit for this design phase is a reviewed field/trust/verification contract and a list of unresolved policy decisions. It does not implement a manifest codec, signature, key store, anti-rollback service, WAL actor, reset authorization, store adapter, recovery CLI or engine startup. It does not supersede ADR-0052's active callback contract or ADR-0055's proposed cross-subsystem authority model.

**G2 and G4 remain BLOCKED. Production release remains NO-GO.** The next engineering gate is to validate whether the pinned Malachite WAL API can supply the required non-destructive evidence and define a guarded startup/reset boundary; implementation must remain blocked where the upstream API cannot meet the contract.
