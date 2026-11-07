# Phase 16 — Cryptographic Account Non-Membership Proofs

**Status:** merged in PR #15; format, Clippy and workspace tests passed. Research prototype; production approval withheld.

## Objective

Prove that an AccountId is absent from the account set committed by the Phase 14 root, without changing the root format. This is an ordered-Merkle neighbor proof, not a sparse Merkle tree.

## Construction

The account map is canonically ordered by numeric AccountId. A non-membership proof contains the queried AccountId and one of four cases:

- **Empty:** no account leaves; verifier recomputes the canonical empty root.
- **BeforeFirst:** inclusion proof for leaf index zero, whose AccountId is greater than the query.
- **AfterLast:** inclusion proof for leaf index count minus one, whose AccountId is less than the query.
- **Between:** inclusion proofs for predecessor and successor, whose indices are adjacent, whose counts match, and whose AccountIds strictly bracket the query.

Each neighbor is independently verified against the same caller-supplied expected root using the Phase 15 membership verifier. The verifier checks adjacency and strict ordering. Existing-account generation returns no absence proof.

## Security properties and boundaries

- Preserves Phase 14 root bytes, domains, account codec and leaf ordering.
- Proves absence only within the account-only committed set and relative to a trusted expected root.
- Does not prove root finality, execution correctness, or absence from uncommitted/omitted state.
- Does not cover contract storage/code, validators, receipts, or other future state domains.
- Proof generation composes existing reference membership-proof generation and is not optimized for large maps.
- No sparse tree, multiproof, incremental index, state-sync protocol, cross-language implementation, fuzz campaign, or independent audit is included.

## Validation

Tests cover empty, before-first, between, after-last, existing-account refusal, wrong root/query, non-adjacent and reversed neighbors, and mixed-root/count attacks. The Phase 14 fixed root vector and Phase 15 inclusion tests remain in place.


## Merge record

PR #15: https://github.com/ReignnRule11/Apex-protocol/pull/15  
Merge commit: 1e971a60fb3534f562b74c19845cdad278ba2492  
PR CI: https://github.com/ReignnRule11/Apex-protocol/actions/runs/42380295427
