# ADR-0007: Static Transaction-Admission Boundary

- **Status:** Accepted for static preflight and a limited state-aware prototype; not a complete transaction-validity decision.
- **Context:** APEX needs a deterministic boundary before stateful transaction validation and eventual EVM execution. Canonical encoding, signature algorithms, account model and fee parameters remain open.
- **Decision:** Implement only chain ID, protocol version, nonzero gas limit, caller-supplied block gas-limit bound, priority-fee/max-fee ordering and checked maximum-fee multiplication. Return the multiplication result as a fee upper bound.
- **Rationale:** These checks are deterministic and testable without selecting a signature suite, codec, account model or execution engine. Overflow and inconsistent fee caps must fail closed.
- **Consequences:** Never describe this as full transaction validation, signature verification, balance sufficiency, fee settlement or EVM execution. The block gas limit is configuration supplied by the caller, not an approved protocol constant. Zero fee remains accepted until fee policy is ratified.
- **Phase 5 extension:** A minimal read-only state adapter checks account existence, exact nonce equality and balance >= value + maximum-fee upper bound with checked arithmetic. The account ID is a u64 placeholder and sender is unauthenticated. The caller must supply a stable snapshot of the candidate parent state.
- **Deferred:** Signature authentication, canonical address, enforced snapshot/version binding, intrinsic gas, transaction ordering, canonical decoding, execution, receipts and fee settlement.
- **Implementation:** validate_transaction_admission in crates/apex-consensus; detailed boundary in transaction-admission.md.
