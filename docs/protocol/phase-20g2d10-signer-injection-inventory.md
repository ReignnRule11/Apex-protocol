# Phase 20G2D10 — concrete signer implementation and injection-path inventory

**Status:** Source inventory and integration contract; documentation only.  
**Base main:** `3e705eeda57c96288142d2d01a7a529282699eda`.  
**Pinned upstream:** Malachite v0.8.2, `a453b32155f4ba9f959724279b52f601bae92e09`.  
**Release impact:** G2/G4 remain BLOCKED; production remains NO-GO.

## Objective

Turn the Phase 20G2D9 interception questions into a concrete inventory of the pinned upstream's signer ownership, engine injection and signing call paths. This phase identifies what is proven by source and what remains application-specific or unproven. It does not implement a wrapper or claim complete key-use coverage.

## Signer ownership and injection

The pinned `app-channel` crate exposes `ConsensusContext<Ctx>` with:

- a boxed `dyn Verifier<Ctx>`;
- an optional boxed `dyn Signer<Ctx>`;
- `new_validator(address, verifier, signer)`, which supplies both;
- `new_full_node(address, verifier)`, which deliberately supplies no signer.

The public `start_engine(...)` convenience function forwards this context to `EngineBuilder::new(...).with_default_consensus(...).build()`. The builder's consensus context is the signer injection point for the default consensus actor. The APEX spike's `build_engine` adapter follows this same public path and accepts the consensus context from its caller.

The engine consensus actor stores the supplied optional signer. Its private `signer()` accessor returns the configured trait object and treats a missing signer as a bug if a signing effect is produced. The engine's effect handler invokes the configured signer for vote, proposal and vote-extension signing effects. A non-validator context has no signer and must not produce validator signing effects.

**Important ownership boundary:** Malachite does not construct or select the application's production private-key signer in this path. The application creates the concrete signer and passes it as `Box<dyn Signer<Ctx>>`. Consequently, an APEX wrapper must be installed by the application at this injection point (or at a proven lower key-backend boundary); changing only the isolated APEX adapter does not establish coverage for an application that constructs another engine path.

## Concrete implementation evidence

The pinned upstream test support crate contains `Ed25519Signer`, backed by `malachitebft_signing_ed25519::PrivateKey`. It implements `Signer<TestContext>` and `Verifier<TestContext>`. It is test support, not evidence of a production application signer or APEX key custody.

The signing crate's `Signer<Ctx>` trait has four required operations:

1. `sign_vote(vote)`;
2. `sign_proposal(proposal)`;
3. `sign_vote_extension(scope, extension)`;
4. `sign_validator_proof(public_key, peer_id)`.

The generic signing crate also supplies forwarding implementations for references, boxed trait objects and `Arc` trait objects. These forwarding implementations delegate to an inner signer; they are not independent key backends and do not themselves prove durable reservation.

The Ed25519 key crate exposes `PrivateKey::sign(&[u8])` and implements the `signature::Signer` byte-slice interface. The ECDSA signing crate provides corresponding scheme-specific `PrivateKey` implementations. These are cryptographic primitives, not APEX-aware signer implementations: the byte-slice operation has no inherent APEX chain ID, protocol version, height, round, step, validator identity or reservation journal.

## Per-operation call-path and encoding inventory

| Operation | Engine/application entry path | Byte construction evidence | Recovery classification |
|---|---|---|---|
| Vote | Consensus handler emits `Effect::SignVote`; actor dispatches to injected `Signer::sign_vote`. Replay helper may return an existing WAL signed vote instead. | The trait receives a typed vote. Concrete signer owns canonical vote bytes and scheme-specific signing. | Fresh key use on replay miss; WAL signature reuse on hit. Must record these as distinct events. |
| Proposal | Consensus handler emits `Effect::SignProposal`; actor dispatches to injected `Signer::sign_proposal`. Replay helper may return an existing WAL signed proposal instead. | The trait receives a typed proposal. Concrete signer owns canonical proposal bytes and scheme-specific signing. | Fresh key use on replay miss; WAL signature reuse on hit. |
| Vote extension | Engine builds a `VoteExtensionScope` and dispatches `sign_vote_extension(scope, extension)` to the injected signer. | Scope explicitly binds height, round, value ID and validator address. Test signer demonstrates a domain-separated envelope over precommit sign bytes and extension bytes; that test encoding is not proof of an APEX production codec. | Distinct signing domain; not interchangeable with vote or proposal reservations. |
| Validator proof | Application calls `sign_validator_proof(public_key, peer_id)` to construct proof bytes before creating `NetworkIdentity::new_validator` and the network context. | Pinned `ValidatorProof::signing_bytes` defines `PoV || u32be(public_key length) || public_key || u32be(peer ID length) || peer ID`. The signing trait requires this preimage without a domain prefix. | Separate network-identity proof operation, not an ordinary height/round consensus vote. It can occur before engine startup and must not be silently omitted from a whole-key-use inventory. |

The pinned consensus handler's `sign_or_replay_vote` and `sign_or_replay_proposal` consult the replay index before creating a signing effect. A matching record is reused; a missing record triggers fresh signing; a divergent record fails. A replay hit therefore bypasses the signer call during that replay, but it does not establish that the historical signature was reserved or that all earlier key use was covered.

## Coverage boundary and unresolved paths

The source establishes the default `ConsensusContext` injection route and the engine's use of its configured signer for consensus signing effects. It does **not** establish any of the following for a future APEX application:

- which concrete signer implementation will be selected or whether it will wrap a hardware, remote, HSM, software or threshold backend;
- whether the application creates additional signers or calls signing keys outside this EngineBuilder path;
- whether validator-proof signing, genesis tooling, key rotation, recovery utilities, tests or operational commands share the same reservation boundary;
- whether every concrete signer's typed-to-byte conversion can be intercepted without reimplementing or diverging from its canonical encoding;
- whether all signing-scheme implementations and feature combinations route through a single auditable byte operation;
- whether a trusted context channel can bind the exact backend bytes to APEX chain/protocol/signer/domain/coordinate data without caller-forgeable ambient metadata.

A source search that finds the upstream test signer is not a complete inventory of downstream application signers. The APEX repository currently has an isolated engine adapter and no production signer implementation wired into a live validator application.

## Required implementation gate

Before choosing or implementing a wrapper:

1. Select and document the actual APEX application signer(s), key backend(s), signing schemes and custody model.
2. Inventory every constructor, clone/box/arc handoff and `ConsensusContext`/EngineBuilder/start-engine invocation in the application.
3. Trace all four operations through the selected concrete implementation to the exact byte slice and final key operation.
4. Search the complete application and operational tooling for direct key use that bypasses the injected signer.
5. Define a non-forgeable, typed context handoff for chain, protocol, signer, domain and coordinate; define separate semantics for validator proofs and vote extensions.
6. Prove the exact bytes are durably reserved before the backend call, and that reservation failure prevents key invocation.
7. Model replay reuse, reservation commit, backend invocation, backend return and signature hand-off as separate states.
8. Add tests for every signer implementation and bypass path, including crash, retry, conflicting bytes, concurrency, rotation, restart and ambiguous storage outcomes.

## Decision

The pinned upstream has one clear default engine injection seam: the application-supplied `Box<dyn Signer<Ctx>>` in `ConsensusContext`. This is a necessary high-level coverage point, but it is not sufficient proof of exact native-byte reservation or complete key-use interception. Validator-proof signing is performed by the application before network identity construction, and replay can reuse WAL signatures without a fresh signer invocation.

Do not implement a production wrapper in this phase. The actual APEX application signer and custody/backend choices remain unspecified, so a complete downstream bypass inventory cannot yet be closed. Proceed only after those application-level inputs are selected and reviewed.

## Exit criteria

- Default EngineBuilder and convenience start-engine injection path are mapped.
- Engine actor signer ownership and non-validator behavior are mapped.
- Concrete upstream test signer and lower key primitives are distinguished from an APEX production signer.
- All four signing operations, validator-proof pre-engine call and vote/proposal replay reuse are classified.
- Remaining application-specific coverage gaps and implementation prerequisites are explicit.
- No runtime, schema, crypto, key custody, WAL or startup behavior changes.
- G2/G4 remain BLOCKED; production release remains NO-GO.
