# Phase 20G2D8 — signer checkpoint and independent anti-rollback contract

**Status:** Design/audit contract submitted for exact-head CI and review; documentation only.  
**Base main:** `f977a157dd7c8af1deb45c75fe5a4873803270c2`.  
**Preceding gate:** Phase 20G2D7 merged; post-merge main CI run #42466807708 passed all four jobs.  
**Release impact:** G2/G4 remain BLOCKED; production remains NO-GO.

## Objective

Define the minimum evidence contract for a future authenticated signer checkpoint and independent anti-rollback anchor. This phase does not select a cryptographic suite, key custody model, hardware service, quorum, or production integration. It must close a design gap without implying that the current v1 reservation journal already satisfies the contract.

## Trust boundaries

Treat these as separate components with separately stated guarantees:

1. **Reservation journal:** records exact signing-domain bytes at a signing coordinate. The current v1 redb journal has no authenticated generation, event sequence, checkpoint, or proof of complete key-use interception.
2. **Checkpoint attestor:** authenticates a canonical checkpoint statement only after independently verifying the journal image and the signer-boundary coverage evidence. A signature over an incomplete or stale scan does not make that scan complete.
3. **Independent monotonic anchor:** stores the accepted checkpoint generation and digest outside the signer database, recovery directory, and their common backup/restore domain. It must reject stale generations and conflicting digests at one generation.
4. **Recovery verifier:** checks the attestation, anchor state, journal evidence, identity scope, and cross-store reconciliation. It must not treat successful checkpoint verification as permission to start a validator.

A file digest is an identifier, not an authenticator. An attestation authenticates a statement, not the truth or completeness of its inputs. The anchor supplies freshness only for the exact scope and generation it protects.

## Required checkpoint statement

A future canonical checkpoint statement must bind, at minimum:

- an explicit format/domain identifier and version;
- APEX chain ID and protocol version;
- stable signer/key identity and signing-domain identifier;
- monotonically increasing generation, unique within the anchored scope;
- previous accepted checkpoint digest (except a separately authorized genesis checkpoint);
- exact journal schema/version, source digest, byte length, record count, and deterministic inspection/report digest;
- exact coverage scope and its lower/upper bounds, including how gaps and coordinate ordering are represented;
- signer-boundary evidence identifying the wrapper/build/configuration and proving which native signing requests were intercepted;
- the digest and scope of any separately retained key-use or signing-event attestation;
- checkpoint creation context and the identity/version of the attestor policy.

The canonical encoding must be deterministic, reject unknown critical fields, reject duplicate fields, enforce bounded lengths, and domain-separate every digest and attestation. No JSON canonicalization or particular binary codec is selected by this design.

## Coverage and signing semantics

A checkpoint cannot infer complete coverage from the rows currently present. Before a checkpoint can claim complete coverage:

- every native signing entry point used by the pinned Malachite integration must be inventoried;
- the APEX-owned wrapper must durably reserve the exact byte sequence passed to the native key backend before invoking that backend;
- the wrapper must bind those exact native bytes to the APEX chain/protocol/signer/height/round/step identity without substituting the distinct Phase 20G2A APEX preimage;
- bypass paths, alternate signers, recovery paths, test-only signers, and configuration changes must be explicitly excluded or instrumented;
- crash tests must cover before reservation, after reservation/before key use, during key use, after key use/before WAL append, after WAL append/before acknowledgement, and ambiguous storage outcomes.

The evidence must distinguish **intent reserved**, **key backend invoked**, and **signature emission attested**. A reservation alone proves neither that a key was invoked nor that a signature was emitted. WAL absence cannot resolve this ambiguity because Malachite v0.8.2 invokes the signer before WAL append.

## Monotonic anchor contract

The anchor is independent only if an attacker or operator able to restore the signer database and recovery directory cannot also silently roll back the anchor. Its interface must provide an atomic compare-and-advance operation over:

`(scope, generation, checkpoint_digest, previous_checkpoint_digest)`

Required behavior:

- first enrollment is an explicit, separately authorized operation;
- lower generations are rejected;
- same generation and same digest are idempotent;
- same generation with a different digest is a hard conflict;
- advancement requires the currently anchored predecessor to match;
- unavailable, stale, reset, or inconsistent anchor state fails closed;
- restoring signer files never automatically restores or rewinds the anchor;
- anchor migration, replacement, disaster recovery, and trust-root rotation require separately authorized, auditable procedures.

The anchor must bind scope strongly enough that checkpoints for different chains, protocol versions, signers, or signing domains cannot overwrite or impersonate one another. The exact scope partitioning and availability model remain open design decisions.

## Recovery verification order

1. Enter `RECOVERY_LOCKED`; disable signing, proposal/vote emission, validator service, and height advancement.
2. Preserve source evidence and establish writer exclusion before opening read-only snapshots.
3. Validate journal schema, bounds, structural integrity, and source-byte invariance.
4. Validate the checkpoint's canonical encoding, attestation, identity scope, journal digest, record count, and coverage claims.
5. Read the independent anchor and require exact generation/digest agreement; reject rollback, fork, or unknown anchor state.
6. Reconcile signer evidence with WAL, finality journal, application projection, and any signer-boundary attestations. A disagreement is a recovery failure, not a reason to repair or reset automatically.
7. Require explicit human/operator authorization under a separately reviewed recovery policy before leaving the locked state. Checkpoint validity alone never authorizes startup.

## Threats and unresolved decisions

This contract does not yet resolve: attestor key custody and compromise; anchor hardware/service and quorum; trust-root distribution; generation allocation under concurrent signers; signer identity rotation; backup restore and anchor outage; split-brain writers; proof of wrapper completeness; signature-emission attestation; canonical codec; cryptographic algorithm; revocation; or operational governance. These require explicit decisions and tests before implementation.

## Exit criteria

- Checkpoint fields, scope, coverage claims, attestor, and anchor are distinct and explicit.
- Freshness and rollback resistance depend on an independently protected monotonic anchor, not a file hash or signer-database backup.
- The signer wrapper's coverage claim is tied to exact native signing bytes and all key-use entry points.
- Intent, key invocation, and signature emission are not conflated.
- Recovery remains locked on missing, stale, conflicting, unauthenticated, or incomplete evidence.
- No runtime, schema, cryptographic, key custody, anchor, WAL, or startup behavior changes in this phase.
- G2/G4 remain BLOCKED; production release remains NO-GO.
