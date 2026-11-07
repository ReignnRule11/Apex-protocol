# ADR-0008: State-Aware Transaction Admission Prototype

- **Status:** Accepted as a limited prototype; parent-state consistency and sender authentication remain caller responsibilities.
- **Context:** Static admission cannot reject stale nonces or obviously unaffordable transactions. These checks require a deterministic read of the candidate parent state.
- **Decision:** Add a read-only `AdmissionState` trait returning an internal `AccountState` (nonce and u128 balance) by an internal `AccountId(u64)`. The state-aware helper first runs static admission, then requires an account, exact nonce equality, and balance at least transaction value plus the maximum-fee upper bound. Multiplication and addition use checked arithmetic.
- **Rationale:** This creates a small, testable state-dependent boundary without prematurely selecting a persistent database, canonical address format, signature algorithm or EVM.
- **Trust assumptions:** The caller supplies a stable view of the exact candidate parent state. The trait does not enforce snapshot isolation, state-root binding, persistence or version consistency. The sender field is untrusted until signature verification is implemented.
- **Consequences:** The maximum fee is an affordability upper bound, not the actual fee. The check is conservative and does not calculate actual gas usage or refunds. Nonce equality is only a state comparison; it does not authenticate the transaction or advance state.
- **Deferred:** Canonical account/address type, signature verification, enforced parent-state version binding, transaction ordering, nonce advancement, intrinsic gas, cumulative block gas accounting, state mutation, fee settlement and execution.
- **Implementation:** `validate_transaction_against_state` in `crates/apex-consensus/src/lib.rs`; see `docs/protocol/transaction-admission.md`.
