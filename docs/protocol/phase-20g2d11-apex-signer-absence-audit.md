# Phase 20G2D11 — APEX application signer absence and cryptographic boundary audit

**Status:** Source inventory; documentation-only phase.  
**Base main:** `305200ecc3bf42825382a1bf69bc5dadc2de5624`.  
**Pinned Malachite:** v0.8.2, `a453b32155f4ba9f959724279b52f601bae92e09`.  
**Release impact:** G2/G4 remain BLOCKED; production remains NO-GO.

## Objective

Close the downstream half of the Phase 20G2D10 inventory: determine whether the APEX workspace contains a concrete application signer, key backend, Malachite signer construction path, or production node wiring to which a reservation-enforcing signer boundary could be attached.

This phase audits the repository as it exists on the stated base commit. It does not infer an implementation from dependencies, protocol preimages, test harnesses, or upstream test utilities.

## Repository inventory

The workspace Rust source inventory contains the following relevant components:

| Component | Observed responsibility | Signer status |
|---|---|---|
| `apex-signer-safety` | Typed chain/protocol/height/round/step/signer coordinate; opaque exact-byte intent; in-memory test reference | Reservation policy only; no key or signature operation |
| `apex-signer-redb` | Durable redb reservation journal; commit-before-return behavior | Storage adapter only; no key or signature operation |
| `apex-consensus::consensus_message` | Constructs the experimental APEX domain-separated preimage | Byte construction only; module explicitly does not sign or verify |
| `apex-consensus::certificate` | Authenticated validator public keys and Ed25519 commit-certificate verification | Verification only; uses `VerifyingKey` and signature verification, not a signing key |
| `apex-consensus::application_boundary` | Engine-neutral application and finality-verifier traits | No consensus signer contract or signer implementation |
| `apex-node` | Placeholder executable | No networking, engine, validator identity, or signing activation |
| `experiments/malachite-spike` | Separate Cargo workspace pinned to Malachite v0.8.2 | Research adapter only; not a root workspace production member |
| `experiments/consensus-harness` | Four-process append/replay adversarial harness | Generic journal test harness; no cryptographic signing or validator consensus |

The root workspace includes `apex-signer-safety` and `apex-signer-redb`, but neither crate depends on a cryptographic signing library. The `apex-consensus` manifest depends on `ed25519-dalek`; the inspected certificate module uses its verification types. A dependency declaration is not evidence of a production signing implementation.

## Confirmed absence at this revision

The complete root-workspace Rust source-file inventory and the relevant manifests were inspected. No APEX-owned concrete implementation of Malachite's `Signer<Ctx>` was found. No production key backend, HSM/KMS adapter, private-key custody module, validator-proof signer, or engine `ConsensusContext`/EngineBuilder production construction path was found.

No source path connects `SigningSafetyStore::reserve` to a cryptographic key invocation. The durable reservation primitive therefore has no production caller capable of enforcing its documented commit-before-key-use rule.

The APEX preimage builder is not a signer and must not be treated as proof of native Malachite byte equivalence. The Ed25519 certificate verifier is not a signer. The isolated Malachite spike and generic process harness do not close the production integration gap.

These are repository-state findings for this exact revision, not claims that no external operator, untracked service, or future branch can contain a signer.

## Required decisions before implementation

A production signer implementation must not be guessed from the current dependency graph. The following decisions and evidence are prerequisites:

1. **Execution ownership:** identify the production validator application/runtime and the exact process that will own signing requests.
2. **Key custody/backend:** select software-held, HSM, KMS, remote signer, or another explicitly reviewed backend; define authentication, authorization, availability, backup and compromise boundaries.
3. **Signature profile:** ratify the algorithm, key encoding, public-key validation, signature encoding and validator identity derivation.
4. **Native byte construction:** trace every Malachite operation to the exact bytes passed to the backend; do not substitute the APEX Phase 20G2A preimage.
5. **Trusted context propagation:** bind native bytes to chain, protocol, signer, domain and coordinate using a non-forgeable request context; define separate vote-extension and validator-proof semantics.
6. **Reservation integration:** durably commit exact native bytes before backend invocation; reservation errors or ambiguous commits must prevent a new key call unless exact retry safety is established.
7. **Coverage and replay:** enumerate every key-use path and distinguish replayed signatures, reservation commit, backend invocation, backend result and signature hand-off.
8. **Recovery and rollback:** select and validate the independent monotonic anchor and recovery policy before enabling validator keys.
9. **Operational lifecycle:** define key enrollment, rotation, revocation, signer-set transitions, incident response, observability and disaster recovery.

## Decision

Record the absence of an APEX production signer and production injection path as an explicit implementation blocker. Do not add a speculative signer, fake backend, test signer promoted to production, or wrapper with an unproven interception claim.

The next implementation proposal must begin with a reviewed signer architecture decision covering custody/backend and execution ownership. Only after that decision may a narrowly scoped implementation phase connect the durable reservation journal to the selected native-byte signing boundary.

## Exit criteria

- Root-workspace signer-related crates and all Rust source paths are inventoried.
- Verification, preimage construction, reservation storage and actual signing are clearly distinguished.
- Malachite research dependencies are not represented as production runtime integration.
- No production signer, key backend, or signer interception is claimed.
- No runtime, key, crypto, schema, WAL, custody or startup behavior changes.
- G2/G4 remain BLOCKED; production release remains NO-GO.
