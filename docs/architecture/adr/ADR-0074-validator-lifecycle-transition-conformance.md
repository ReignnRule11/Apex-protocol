# ADR-0074: Validator lifecycle transition conformance contract

- **Status:** Proposed
- **Date:** 2026-11-08
- **Phase:** 20G2D18
- **Related:** ADR-0067, ADR-0068, ADR-0070, ADR-0071, ADR-0072, ADR-0073

## Context

Phase 20G2D17 established an abstract validator lifecycle vocabulary and explicit governance decision register. The next design risk is transition ambiguity: application, eligibility, governance disposition and actual consensus membership could be collapsed into one mutable status, while retries, concurrent decisions, stale evidence or off-chain administrative actions could be mistaken for protocol effects.

The production signer and recovery path remains unproven. Lifecycle approval therefore cannot be used as a substitute for committed consensus membership or signer safety evidence.

## Proposed decision

Adopt a versioned transition-conformance contract as a prerequisite for any validator lifecycle implementation. Every transition must bind exact chain/protocol/operator/validator scope, prior state version, applicable policy/evidence revision, authorization context and—where relevant—verifiable protocol transition proof.

Implementations must preserve separate application, eligibility, governance and consensus state domains; reject stale, conflicting, replayed or unauthorized transitions; provide deterministic retry/idempotency behavior; and retain append-only audit lineage.

Administrative systems may propose and observe transitions but cannot directly mutate validator-set membership, voting power or signer enablement. A lifecycle transition becomes effective only when the deterministic protocol transition is committed and independently verifiable.

Signer enablement remains independently gated by committed active membership, trusted consensus context, exact native signing bytes, durable reservation-before-key-call, complete signing-path interception, anti-equivocation, anti-rollback and recovery. Governance exceptions cannot waive these requirements.

## Boundaries

This ADR does not select governance authority, reviewers, quorum, deadlines, eligibility thresholds, emergency powers, activation timing, height/epoch semantics, exit queues, proof format, registry, database, API, cryptography, signer backend or custody model.

It does not authorize runtime, consensus, signer, key, WAL or schema changes. It is not a complete executable validator-set state machine.

## Consequences

- Administrative status cannot be misrepresented as consensus membership.
- Stale, conflicting, replayed or incomplete transitions fail closed.
- Protocol membership changes remain deterministic and auditable.
- Lifecycle systems remain separated from operator signing authority.
- Implementation remains blocked pending ratified policy, complete protocol transition semantics and production signer/recovery evidence.

## Acceptance evidence

Change this ADR to **Accepted** only after:
- governance decisions and the complete lifecycle transition table are ratified;
- a canonical transition envelope and authoritative ordering mechanism are specified;
- implementation demonstrates deterministic concurrency, idempotency, replay and restart behavior;
- validator-set transition proofs are verified against the production consensus integration;
- production signer integration and recovery satisfy ADR-0067/0068;
- independent security review validates the administrative/consensus/signer boundaries.

**Current gate:** Proposed only. Governance decisions OPEN. G2/G4/G5 BLOCKED. Production NO-GO.
