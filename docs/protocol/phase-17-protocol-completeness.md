# Phase 17 — Protocol Completeness Baseline

**Status:** Draft baseline committed for technical review; G0 remains BLOCKED.  
**Goal:** Turn the early architecture and isolated prototypes into a complete protocol contract before consensus-critical implementation.

## Scope delivered

- Introduces Protocol Specification v0.2 as a comprehensive working draft.
- Defines protocol identity/versioning domains and normative determinism, canonicality, resource-bound and fail-closed constraints.
- Defines complete logical state domains: EVM accounts, contract storage/code, validator registry, staking, monetary accounting, consensus parameters, governance, receipts and protocol metadata.
- Establishes the target 20-byte EVM address model and explicitly quarantines Phase 14–16 AccountId(u64) account-root prototypes from production state.
- Defines transaction admission/execution pipeline, transaction identity/replay requirements, block/receipt commitments and consensus adapter obligations.
- Defines mandatory supply-conservation invariants while leaving numerical tokenomics unapproved.
- Defines validator lifecycle, genesis requirements, governance/upgrade semantics, full data-availability requirement, optional Bitcoin anchoring boundary and failure/resource semantics.
- Adds a decision register listing all unresolved consensus-critical choices and the evidence needed to close them.

## Important architectural findings

1. **The existing AccountId(u64) model cannot be the production EVM account model.** Production EVM compatibility requires 20-byte addresses and 256-bit values. The prototype remains a historical research artifact. A new codec and state commitment version are required.
2. **The account-only root is not a complete state root.** Contract code/storage, validator/staking, receipts and monetary/governance state must be committed if they influence future validity or execution.
3. **The transaction planner is not an authenticated transaction pipeline.** Sender fields and temporary TransactionId values are not cryptographic identity.
4. **Consensus safety requires more than a BFT crate.** Engine finality, authenticated certificates, durable anti-double-sign, validator transitions and atomic coordination with application state must be specified and tested.
5. **Monetary direction is not tokenomics.** Supply cap, issuance, fees, staking and slashing values remain unapproved pending simulation and adversarial analysis.

## Explicit non-deliverables

No cryptographic algorithm is selected by assumption; no BFT engine is integrated; no EVM execution is implemented; no production account codec/state root is activated; no genesis economics are approved; no production keys, token sale, testnet or mainnet are authorized.

## Exit criteria

Phase 17 is not considered fully closed until:
- every row in the v0.2 decision register is either ratified through an ADR or explicitly deferred with a safe implementation boundary;
- canonical test vectors exist for all selected encodings and cryptographic operations;
- complete state and transition invariants are reviewed;
- economic and validator parameter simulations are published;
- protocol review identifies no unresolved consensus-critical ambiguity;
- G0 is formally changed from BLOCKED only with evidence and accountable approval.

## Repository record

The v0.2 draft is intentionally reviewable and versioned. It does not silently amend v0.1 prototype formats or promote research code to production.
