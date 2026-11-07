# ADR-0009: Pure Single-Transaction Transition Planning

- **Status:** Accepted as a deterministic planning prototype; no persistent state mutation is implemented.
- **Context:** Admission checks alone do not define the account changes a valid transfer proposes. The project needs an explicit boundary between validating against a parent state and committing resulting account values.
- **Decision:** Add `plan_transaction_transition` to `apex-consensus`. It reads sender and recipient values through the read-only parent-state interface and returns a plan containing before/after account values. The sender nonce advances by one. For distinct accounts, sender balance is debited by value plus the maximum-fee upper bound and recipient balance is credited by value. For a self-transfer, value nets to zero and only the maximum-fee upper bound is deducted. Missing recipients are rejected. Nonce increment, recipient credit and arithmetic use checked operations.
- **Rationale:** A pure plan is deterministic and testable without prematurely choosing a database, persistence model, canonical address, signature algorithm, fee settlement policy or EVM implementation.
- **Atomicity requirement:** A future application/storage adapter must persist every account update in a plan atomically or persist none. The current planner does not enforce atomic writes, snapshot isolation, durable state versions, or crash recovery.
- **Fee semantics:** The maximum fee is a conservative escrow ceiling, not the actual charged fee. Fee refund, base-fee burn and proposer reward allocation are deferred until monetary and execution rules are ratified.
- **Consequences:** This is not a complete block state transition. Transaction ordering, cumulative block gas, receipts, state roots, account creation, signature verification and EVM execution remain unimplemented.
- **Implementation:** `plan_transaction_transition` in `crates/apex-consensus/src/lib.rs`; see `docs/protocol/transaction-admission.md`.
