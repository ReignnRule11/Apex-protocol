# Phase 20G2C1 — Malachite-native finality bridge contract

**Status:** API audit and integration contract submitted; no runtime bridge is implemented yet.
**Base main:** `6b4abb3c08279ca91a554ee716dd0980a1b57dcd`
**Pinned engine:** Malachite v0.8.2, commit `a453b32155f4ba9f959724279b52f601bae92e09`.
**Scope:** Resolve the certificate-format boundary and specify a fail-closed path from Malachite's actual application events to APEX's verified application commit boundary.

## Objective

Establish the exact trust boundary before writing the runtime adapter. Malachite's native commit certificate and APEX's experimental Phase 20G2B certificate are different cryptographic protocols. They must not be converted by copying signature bytes or by assuming their signing preimages are interchangeable.

## Pinned upstream API findings

The audit is against the exact pinned Malachite source revision, not a moving branch:

- `malachitebft_core_types::CommitCertificate<Ctx>` contains `height`, `round`, `value_id`, and `commit_signatures`. Each commit signature carries a native validator address and the signature type selected by `Ctx::SigningScheme`.
- The signing scheme explicitly provides `encode_signature` and `encode_public_key`. These are Malachite-native encodings, not the APEX Phase 20G2A fixed-width preimage.
- `malachitebft_signing::VerifierExt::verify_commit_certificate` accepts the Malachite context, native certificate, native validator set and threshold parameters. Its documented behavior reconstructs and verifies each native precommit signature and enforces the configured voting-power threshold.
- The channel application API exposes both `AppMsg::Decided` and `AppMsg::Finalized`. `Decided` is an early decision notification. `Finalized` supplies the certificate after the finalization period and requires the application to reply with `Next::Start` or `Next::Restart`. The application must not advance height on an unverified event.
- The certificate's `value_id` identifies a Malachite value; it is not, by itself, proof that an APEX block hash or application state root has been correctly derived from the corresponding value bytes.

Upstream source reviewed:
- `code/crates/core-types/src/certificate.rs`
- `code/crates/core-types/src/signing.rs`
- `code/crates/core-types/src/context.rs`
- `code/crates/signing/src/ext.rs`
- `code/crates/app-channel/src/msgs.rs`
- `code/crates/test/app/src/app.rs`

## Required bridge sequence

The runtime adapter must implement the following order. No step may be skipped or reordered:

1. Receive Malachite `AppMsg::Finalized`; treat the event, certificate, extension data and associated value as untrusted inputs at the application boundary.
2. Resolve the exact finalized value bytes from the application-owned bounded value store. Decode them with the future canonical APEX block codec; reject malformed, oversized, non-canonical or trailing data.
3. Recompute the APEX block identity from decoded canonical block contents. Do not treat Malachite `value_id` as an APEX block hash. Require an explicit deterministic mapping between the Malachite value ID and the recomputed APEX block identity.
4. Resolve the native validator set that Malachite used for this height. Verify the native `CommitCertificate<Ctx>` with the pinned `VerifierExt::verify_commit_certificate`, native signing scheme, exact validator set and explicitly selected threshold parameters. Reject unknown/duplicate signers, invalid signatures, wrong height/round/value and insufficient power through the native verifier.
5. Independently assert APEX chain ID, protocol version, expected next height, parent block hash and validator-set transition against APEX's trusted application context. Confirm that the native validator set and APEX authenticated validator set have a one-to-one address/public-key/power mapping. Do not infer governance authorization from a locally supplied schedule.
6. Only after all checks succeed, construct the private `VerifiedFinalizedBlock` through the engine-neutral `FinalityVerifier` boundary and invoke deterministic application commit.
7. Persist the finalized block, resulting application state, consensus progress and signer safety state under a crash-consistent protocol. This atomic persistence/recovery guarantee is not implemented by this subphase.
8. Reply to Malachite with `Next::Start` only after a successful durable commit. On validation or commit failure, fail closed and use the documented restart/error path; never acknowledge a failed commit as finalized.

## Critical cryptographic incompatibility

Phase 20G2B verifies Ed25519 signatures over the APEX Phase 20G2A `APEX-CONSENSUS-MSG-V1` preimage. Malachite signs its own native vote representation through `Ctx::SigningScheme` and its `VerifierExt` implementation. These are not equivalent signing domains.

Therefore:
- Do not wrap Malachite signatures in APEX `CommitSignature` objects and pass them to `verify_commit_certificate` from Phase 20G2B.
- Do not claim that verifying the APEX preimage authenticates Malachite's native certificate.
- The first runtime bridge must use Malachite's native certificate verifier for the engine's native precommits, then separately bind the verified decision to APEX chain/version/block/application identity.
- A future unified APEX-native signing profile requires an explicit Malachite signing adapter that produces and verifies the exact ratified APEX bytes. It is not part of this subphase.

## Security invariants

- No application commit before native certificate verification and APEX identity/linkage validation.
- No height advancement on `Decided`; only the verified `Finalized` path may request `Next::Start`.
- No trust in caller-supplied value IDs, hashes, signer lists, power totals or validator-set hashes without recomputation and authenticated provenance.
- Bound value bytes, certificate cardinality, signature encodings and decode work before expensive processing.
- A certificate for a different chain, protocol version, height, round, value, parent or validator set must fail closed.
- Repeated finality notifications must be idempotent; conflicting finality for an already committed height must halt and surface evidence, not overwrite state.
- Signer reservations must be durably committed before any signing operation; backup rollback remains a hard validator activation blocker.

## Explicitly out of scope

This contract does not implement the runtime, codec, native signing provider, validator-set governance, value store, persistent atomic commit, signer anti-rollback, network transport, multi-validator process orchestration or safety/liveness proof. It does not add Malachite to the production workspace or enable production keys.

## Exit criteria for 20G2C1

1. This exact API contract is reviewed and merged with all CI green.
2. The follow-up implementation compiles against the exact pinned Malachite revision and calls its native certificate verifier.
3. Tests prove invalid native signatures, wrong height/round/value, wrong APEX chain/version/parent, validator-set mismatch and quorum failure cannot reach application commit.
4. The application acknowledges `Finalized` only after successful commit; failure/restart behavior is tested.
5. No production promotion until durable cross-subsystem recovery, process-level fault testing and independent review pass.

## Next subphase

**20G2C2 — compile-checked native certificate adapter and APEX identity binding.** Implement the generic adapter against the pinned native `CommitCertificate`, `VerifierExt` and validator-set APIs. Keep it in the isolated experiment. The following subphase must attach it to the actual `Channels` event loop and prove the application commit gate with adversarial tests.
