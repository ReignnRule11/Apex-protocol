# Phase 20G2D9 — Malachite native signer boundary and interception audit

**Status:** Source audit and integration contract; documentation only.  
**Base main:** `eba3a6d3cddb972ce8d7c8bd7af6acb406b7377a`.  
**Pinned upstream:** Malachite v0.8.2, `a453b32155f4ba9f959724279b52f601bae92e09`.  
**Release impact:** G2/G4 remain BLOCKED; production remains NO-GO.

## Objective

Map the pinned Malachite signing surface before designing a wrapper that can claim complete signer-use coverage. Phase 20G2D8 requires durable reservation of the exact bytes consumed by every native key backend before key invocation. A wrapper around only one high-level method is insufficient unless all other signing routes are proven unreachable or separately covered.

## Source-audited API surface

The pinned upstream `code/crates/signing/src/lib.rs` defines the async `Signer<Ctx>` trait with four signing operations:

1. `sign_vote(vote)` returns a signed vote.
2. `sign_proposal(proposal)` returns a signed proposal.
3. `sign_vote_extension(scope, extension)` returns a signed extension. Its scope includes height, round, value ID and validator address.
4. `sign_validator_proof(public_key, peer_id)` returns a validator proof.

The methods accept typed consensus objects, not the final byte slice passed to a cryptographic key. The trait documentation delegates canonical encoding and domain separation to the implementation. Therefore, a decorator at this trait boundary can observe the typed request, but cannot claim it reserved the exact native signing bytes unless it independently reproduces and proves the implementation's byte construction.

The same pinned source exposes `SigningScheme` in `code/crates/core-types/src/signing.rs`, with scheme-specific key types. The pinned Ed25519 implementation in `code/crates/signing-ed25519/src/lib.rs` exposes `PrivateKey::sign(&[u8])` and implements the lower-level `signature::Signer` interface. This lower cryptographic boundary receives message bytes, but it does not itself carry APEX chain/protocol/height/round/step metadata.

The consensus signature handler in `code/crates/core-consensus/src/handle/signature.rs` issues `Effect::SignVote` and `Effect::SignProposal`. Its replay helpers first consult the WAL replay index: when a matching recorded vote/proposal exists, the recorded signed message is reused; when absent, the helper invokes the signing effect. This replay optimization is not proof that all key use is represented in the WAL, nor does it cover vote-extension or validator-proof signing.

## Coverage implications

- **High-level trait interception:** sees all four typed method requests if every engine signer reference is routed through the wrapper. It does not automatically reveal the exact native bytes and must prove there is no alternate signer instance or direct key path.
- **Lower key-backend interception:** can observe the exact byte slice at the cryptographic operation, but must receive a trusted, non-forgeable context binding those bytes to the APEX chain/protocol/signer/coordinate and signing domain. Context cannot be inferred from arbitrary message bytes without a separately reviewed decoder.
- **Vote and proposal replay:** a replay hit may reuse an existing signature without invoking the key backend; a replay miss can invoke it. Recovery evidence must distinguish reuse from fresh key invocation and must not interpret a WAL hit as proof of complete historical reservation coverage.
- **Vote extensions:** have their own scope and domain. Their coordinate binding must include the value and validator scope and must not be collapsed into ordinary vote signing.
- **Validator proofs:** bind public-key and peer-ID material and are not ordinary height/round consensus votes. They require an explicit separate domain/scope policy or a documented exclusion from the consensus anti-double-sign checkpoint.
- **APEX vs Malachite domains:** the Phase 20G2A `APEX-CONSENSUS-MSG-V1` preimage is not a substitute for Malachite's native signing bytes. The wrapper must reserve the exact native bytes and separately bind them to canonical APEX identity.

## Required evidence before a wrapper can claim complete coverage

1. Inventory every concrete `Signer<Ctx>` implementation and every construction/injection path used by the pinned EngineBuilder/runtime.
2. Trace each of the four trait methods through concrete serialization to the exact lower-level key operation.
3. Enumerate all direct key-backend paths that can bypass the trait, including test, recovery, startup, validator-proof and alternate signing-scheme paths.
4. Establish a typed, authenticated context handoff from the high-level consensus request to the lower key backend; reject missing, duplicate, mismatched or stale context.
5. Prove that the exact bytes at the backend are durably reserved before the backend is invoked; propagate reservation failure without calling the key.
6. Define separate evidence states for replayed signature, reservation committed, backend invoked, backend result returned, and signature emission/hand-off attested.
7. Add deterministic tests for every method and bypass path, plus crashes before reservation, after reservation, immediately before key invocation, during/after key invocation, before WAL append, after WAL append and before acknowledgement.
8. Test concurrent calls at the same coordinate, same bytes, conflicting bytes, signer rotation, process restart and storage ambiguity. A failed or ambiguous reservation must fail closed.

## Decision

Do not implement or claim a production signer wrapper in this phase. The source audit establishes that the high-level trait and lower key boundary expose different information; selecting a safe interception point requires a concrete implementation/injection-path inventory and a reviewed context propagation design. No exact native signing-byte codec is inferred from the APEX preimage, and no signer checkpoint or anti-rollback guarantee is added.

## Exit criteria

- All four upstream `Signer<Ctx>` methods and the replay reuse path are recorded.
- The difference between typed request interception and exact-byte key-backend interception is explicit.
- Vote-extension and validator-proof domains are not silently conflated with ordinary consensus votes.
- Required bypass, crash, concurrency and recovery evidence is specified.
- No production workspace, key custody, cryptographic, schema, WAL or startup behavior changes.
- G2/G4 remain BLOCKED; production release remains NO-GO.
