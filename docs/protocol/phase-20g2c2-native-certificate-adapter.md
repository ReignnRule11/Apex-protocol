# Phase 20G2C2 — compile-checked native certificate adapter and APEX identity binding

**Status:** Isolated adapter implementation submitted for exact-head CI; no runtime event loop or production finality is claimed.
**Base main:** `2ae89b12eedfbf078cc2ba6dad4801ee5ced063d`
**Pinned engine:** Malachite v0.8.2, commit `a453b32155f4ba9f959724279b52f601bae92e09`.

## Delivered

- Added a direct, exact-revision dependency on `malachitebft-signing` to the separate `experiments/malachite-spike` Cargo workspace only.
- Added `native_finality_bridge.rs`, a generic adapter accepting Malachite's native `CommitCertificate<Ctx>`, native context, native validator set, explicit `ThresholdParams`, expected height/round/value ID and an independently recomputed APEX block identity.
- The adapter rejects mismatched native height, round and value ID before cryptographic verification.
- An explicit `ApexValueIdBinding` contract requires the caller to prove the deterministic mapping from native value ID to canonical APEX block identity. No implicit assumption that Malachite's value ID equals an APEX block hash is made.
- Calls Malachite's `VerifierExt::verify_commit_certificate` with the supplied native context, validator set and thresholds. Native signatures are never copied into or verified as APEX Phase 20G2B signatures.
- Constructs an opaque `VerifiedNativeFinality` token only after identity validation, value-ID binding and native certificate verification all succeed.
- Added unit coverage for invalid APEX chain/protocol context and nil APEX block hash.

## Trust boundary and limitations

This is a compile-checked adapter primitive, not the complete bridge. The caller remains responsible for bounded value retrieval, strict canonical APEX block decoding, recomputation of block hash and parent linkage, authenticated validator-set provenance and one-to-one native/APEX validator address-key-power mapping.

It does not yet attach to `Channels` or process `AppMsg::Finalized`; it does not gate a real application commit, acknowledge with `Next::Start`, handle `Decided`, implement durable atomic persistence, recover consensus/application state, protect signer journals from backup rollback, or establish BFT safety/liveness. It is not included in the production Cargo workspace and does not start validators or open sockets.

## Validation

The exact-head CI workflow must compile and test the isolated workspace against the pinned Malachite revision. No local Cargo, rustfmt, test, or build result is claimed. CI success is necessary but does not prove production correctness.

## Next subphase

**20G2C3 — attach the adapter to the actual Malachite Channels event loop and test the application commit gate.** The implementation must demonstrate that `Decided` cannot advance height; invalid native certificates, value-ID binding failures, APEX chain/version/height/parent mismatches and commit failures cannot acknowledge finality; and `Finalized` advances only after the application commit succeeds. Durable cross-subsystem crash consistency remains a separate release blocker.
