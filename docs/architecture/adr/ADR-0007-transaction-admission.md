# ADR-0007: Static Transaction-Admission Boundary

- **Status:** Accepted for a narrow preflight implementation; not a complete transaction-validity decision.
- **Context:** APEX needs a deterministic boundary before stateful transaction validation and eventual EVM execution. Canonical encoding, signature algorithms, account model and fee parameters remain open.
- **Decision:** Implement only chain ID, protocol version, nonzero gas limit, caller-supplied block gas-limit bound, priority-fee/max-fee ordering and checked maximum-fee multiplication. Return the multiplication result as a fee upper bound.
- **Rationale:** These checks are deterministic and testable without selecting a signature suite, codec, account model or execution engine. Overflow and inconsistent fee caps must fail closed.
- **Consequences:** Never describe this as full transaction validation, signature verification, balance sufficiency, fee settlement or EVM execution. The block gas limit is configuration supplied by the caller, not an approved protocol constant. Zero fee remains accepted until fee policy is ratified.
- **Deferred:** Sender/signature authentication, nonce and balance checks, intrinsic gas, transaction ordering, canonical decoding, execution, receipts and fee settlement.
- **Implementation:** validate_transaction_admission in crates/apex-consensus; detailed boundary in transaction-admission.md.
